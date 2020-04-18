use crate::config::{grafana::to_grafana_dashboard_json, Config, Target, TargetDefault};
use crate::messages::{EntryDTO, FailureDTO};
use crate::reporters::file::FileReporterTask;
use crate::utils::{
    file::{read_to_string, to_absolute_pair},
    prometheus as util_prometheus,
};
use crate::{requesters::http::HttpRequestTask, server::SonarServer};
use broadcast::RecvError;
use futures::future::{AbortHandle, Abortable};
use log::*;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use prometheus::{Counter, Histogram, HistogramOpts, Opts, Registry};
use reqwest::Client;
use std::error::Error;
use std::{collections::HashMap, path::PathBuf, time::Duration};
use tokio::fs::File;
use tokio::{
    prelude::*,
    sync::{broadcast, broadcast::channel, oneshot},
    time::delay_for,
};

pub struct Executor {
    http_client: Client,
    server_kill_sender: Option<oneshot::Sender<()>>,
    graceful_shutdown_complete_receiver: Option<oneshot::Receiver<()>>,
    server_running: bool,
    prometheus_registry: Option<Registry>,
    requester_abort_controllers: Option<Vec<AbortHandle>>,
    reporter_abort_controllers: Option<Vec<AbortHandle>>,
}

impl Executor {
    pub async fn watch_config_and_handle<'a>(
        config_file_path: PathBuf,
        client: Client,
    ) -> Result<(), Box<dyn Error>> {
        let (abs_path_to_config_file, abs_path_to_config_folder) =
            to_absolute_pair(config_file_path.clone()).await;

        let (tx, rx) = std::sync::mpsc::channel::<DebouncedEvent>();
        let mut config_watcher = watcher(tx, std::time::Duration::from_millis(100))
            .expect("failed to create config watcher");

        config_watcher
            .watch(
                abs_path_to_config_folder.clone(),
                RecursiveMode::NonRecursive,
            )
            .expect("failed to watch config root folder");

        // handle initial start run
        let mut executor = Executor::new(client);
        executor.handle(abs_path_to_config_file.clone()).await;
        debug!(
            "watching for config changes in {}",
            abs_path_to_config_folder
                .to_str()
                .expect("failed to stringify abs config folder path")
        );

        // handle when changes are made to the config file
        loop {
            match rx.recv() {
                Ok(event) => match event {
                    DebouncedEvent::Create(path) | DebouncedEvent::Write(path) => {
                        if !path.eq(&abs_path_to_config_file) {
                            continue;
                        }
                        executor.handle(abs_path_to_config_file.clone()).await;
                    }
                    DebouncedEvent::Remove(path) => {
                        if !path.eq(&abs_path_to_config_file) {
                            executor.stop_all().await;
                            continue;
                        }
                        println!("Not implemtented: handle deleted config");
                    }
                    _ => (),
                },
                Err(err) => panic!("Failed to listen to config changes: {}", err.to_string()),
            }
        }
    }

    pub fn new(http_client: Client) -> Self {
        Self {
            http_client,
            server_kill_sender: None,
            graceful_shutdown_complete_receiver: None,
            server_running: false,
            prometheus_registry: None,
            requester_abort_controllers: None,
            reporter_abort_controllers: None,
        }
    }

    pub async fn handle(&mut self, config_path: PathBuf) {
        let config_str = match read_to_string(config_path.to_string_lossy().to_string().as_str())
            .await
        {
            Ok(v) => v,
            Err(err) => {
                error!("config file missing - run 'sonar init' to create one and check that the path is correct. You can add the file without stopping the program. For more debug - enable '-d' debug flag for more info. ");
                debug!("error: {}", err);
                return;
            }
        };
        match serde_yaml::from_str::<Config>(&config_str) {
            Err(err) => {
                error!("invalid config - Please fix: {}", err);
                return;
            }
            Ok(config) => {
                info!("config loaded");

                self.handle_grafana_dashboard(config.clone()).await;
                let request_data_receivers = self.handle_requesters(config.targets.clone()).await;
                if config.server.is_some() {
                    match config.server.clone() {
                        Some(server_config) => {
                            if server_config.prometheus_endpoint.is_some() {
                                self.handle_prometheus_exporter(
                                    config.targets.clone(),
                                    config.targets_defaults.clone(),
                                    request_data_receivers,
                                );
                            }
                        }
                        _ => (),
                    }
                }
                self.handle_server(config.clone()).await;
            }
        }
    }

    async fn stop(&mut self, abort_controllers: Vec<AbortHandle>) {
        abort_controllers.iter().for_each(|a| a.abort());
    }

    async fn stop_all(&mut self) {
        self.stop_requesters().await;
        self.stop_reporters().await;
    }

    async fn stop_reporters(&mut self) {
        if self.reporter_abort_controllers.is_some() {
            let controllers = self
                .requester_abort_controllers
                .take()
                .expect("failed to take requester forceful shutdown handlers");
            self.stop(controllers).await;
        }
    }

    async fn stop_requesters(&mut self) {
        if self.requester_abort_controllers.is_some() {
            let controllers = self
                .requester_abort_controllers
                .take()
                .expect("failed to take requester forceful shutdown handlers");
            self.stop(controllers).await;
        }
    }

    async fn handle_requesters(
        &mut self,
        targets: Vec<Target>,
    ) -> Vec<broadcast::Receiver<Result<EntryDTO, FailureDTO>>> {
        // stop/clean out old requesters
        self.stop_reporters().await;
        self.stop_requesters().await;

        let mut requester_abort_handles = Vec::new();
        let mut reporter_abort_handles = Vec::new();
        let mut request_result_rx = Vec::new();
        for target in targets {
            let (broadcast_tx, _broadcast_rx) = channel::<Result<EntryDTO, FailureDTO>>(1);
            request_result_rx.push(_broadcast_rx);

            // reporters
            if target.log.is_some() {
                let log = target.clone_unwrap_log();
                let mut file_reporter = FileReporterTask::new(log.file, broadcast_tx.subscribe())
                    .await
                    .expect("failed to create flat file reporter");

                let (abort_handle, abort_registration) = AbortHandle::new_pair();
                reporter_abort_handles.push(abort_handle);
                tokio::spawn(Abortable::new(
                    async move {
                        file_reporter.run().await;
                    },
                    abort_registration,
                ));
            }
            // requesters
            let requester = HttpRequestTask::new(self.http_client.clone(), broadcast_tx);
            let (abort_handle, abort_registration) = AbortHandle::new_pair();
            requester_abort_handles.push(abort_handle);
            tokio::spawn(Abortable::new(
                async move {
                    requester.run(target).await;
                },
                abort_registration,
            ));
        }
        self.requester_abort_controllers = Some(requester_abort_handles);

        request_result_rx
    }

    fn handle_prometheus_exporter(
        &mut self,
        targets: Vec<Target>,
        target_defaults: Option<TargetDefault>,
        receivers: Vec<broadcast::Receiver<Result<EntryDTO, FailureDTO>>>,
    ) {
        let mut timers: HashMap<String, Histogram> = HashMap::new();
        let mut counters: HashMap<String, Counter> = HashMap::new();
        let registry = Registry::new();

        for target in &targets {
            let counter_success_name =
                util_prometheus::counter_success_name(target.clone_unwrap_name());
            let counter_success = Counter::with_opts(Opts::new(
                counter_success_name.clone(),
                String::from("Number of successful requests"),
            ))
            .expect("failed to create success counter");
            counters.insert(counter_success_name.clone(), counter_success.clone());

            let timer_name = util_prometheus::timer_name(target.clone_unwrap_name());
            let prometheus_response_time_bucket =
                if target.prometheus_response_time_bucket.is_none() {
                    if target_defaults.is_none() {
                        TargetDefault::default_prometheus_response_time_bucket()
                    } else {
                        target_defaults
                            .clone()
                            .unwrap()
                            .prometheus_response_time_bucket
                    }
                } else {
                    target.prometheus_response_time_bucket.clone().unwrap()
                };
            let request_time_opts =
                HistogramOpts::new(timer_name.clone(), String::from("latency in ms"))
                    .buckets(prometheus_response_time_bucket);
            let request_time =
                Histogram::with_opts(request_time_opts).expect("unable to create timer");

            timers.insert(timer_name.clone(), request_time.clone());

            registry
                .register(Box::new(request_time))
                .expect("unable to register timer");
            registry
                .register(Box::new(counter_success))
                .expect("unable to register timer");
        }
        self.prometheus_registry = Some(registry);

        // TODO optimize this, make a map of receivers and what target they are connected to.
        for mut r in receivers {
            let timers = timers.clone();
            let counters = counters.clone();
            tokio::spawn(async move {
                loop {
                    debug!("started prometheus metrics receiver");
                    match r.recv().await {
                        Ok(m) => match m {
                            Ok(r) => {
                                counters
                                    .get(&util_prometheus::counter_success_name(
                                        r.target.clone_unwrap_name(),
                                    ))
                                    .expect("could not find success counter by key")
                                    .inc();
                                timers
                                    .get(&util_prometheus::timer_name(r.target.clone_unwrap_name()))
                                    .expect("could not find timer by key")
                                    .observe(r.latency as f64);
                            }
                            Err(err) => {
                                timers
                                    .get(&util_prometheus::timer_name(
                                        err.target.clone_unwrap_name(),
                                    ))
                                    .expect("could not find timer by name")
                                    .observe(err.latency as f64);
                            }
                        },
                        Err(err) => {
                            match err {
                                RecvError::Closed => {
                                    debug!("stopped prometheus metrics receiver: {}", err);
                                    break;
                                }
                                RecvError::Lagged(n) => {
                                    warn!(
                                        "prometheus metrics receiver is lagging behind with: {}",
                                        n
                                    );
                                }
                            };
                        }
                    }
                    delay_for(Duration::from_millis(500)).await;
                }
            });
        }
    }

    async fn handle_grafana_dashboard(&self, config: Config) {
        if !config.grafana.is_some() {
            return;
        }
        let grafana_config = match config.clone().grafana {
            Some(p) => p,
            None => {
                error!("missing grafana config");
                return;
            }
        };
        info!(
            "outputting grafana dashboard json to {}",
            grafana_config.dashboard_json_output_path
        );
        let mut file = match File::create(grafana_config.dashboard_json_output_path).await {
            Ok(f) => f,
            Err(err) => {
                error!("failed to create grafana dashboard file: {}", err);
                return;
            }
        };
        match file
            .write_all(to_grafana_dashboard_json(&config).as_bytes())
            .await
        {
            Ok(_) => (),
            Err(err) => {
                error!("failed to write grafana dashboard file: {}", err);
                return;
            }
        }
    }

    async fn start_server(&mut self, config: Config) {
        let config = config.server.expect("failed to unwrap server config");
        let mut server = SonarServer::new(config, self.prometheus_registry.take());
        let (server_kill_sender, graceful_shutdown_complete_receiver) = server.start();
        self.server_kill_sender = Some(server_kill_sender);
        self.graceful_shutdown_complete_receiver = Some(graceful_shutdown_complete_receiver);
    }

    async fn stop_server_gracefully(&mut self) {
        let kill_signal = self.server_kill_sender.take();
        let graceful_shutdown_complete_receiver = self.graceful_shutdown_complete_receiver.take();
        if kill_signal.is_some() {
            info!("waiting for graceful server shutdown");
            kill_signal
                .expect("failed to get server kill signal sender")
                .send(())
                .expect("failed to send kill signal to server");
            graceful_shutdown_complete_receiver
                .expect("could not get graceful shutdown complete receiver")
                .await
                .expect("failed to get signal that server stopped");
        }
    }

    async fn handle_server(&mut self, config: Config) {
        if self.server_running {
            self.stop_server_gracefully().await;
            self.server_running = false;
        }
        if !config.server.is_some() {
            return;
        }
        self.start_server(config).await;
        self.server_running = true;
    }
}
