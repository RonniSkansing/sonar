use crate::config::{grafana::to_prometheus_grafana, Config, Target};
use crate::messages::{EntryDTO, FailureDTO};
use crate::reporters::file::FileReporterTask;
use crate::utils::prometheus as util_prometheus;
use crate::utils::tokio_shutdown::{self, to_abortable_with_registration, AbortController};
use crate::{requesters::http::HttpRequestTask, server::SonarServer};
use log::*;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use prometheus::{Counter, Histogram, HistogramOpts, Opts, Registry};
use reqwest::Client;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::sync::broadcast;
use tokio::sync::{broadcast::channel, oneshot};

pub struct Executor {
    http_client: Client,
    server_kill_sender: Option<oneshot::Sender<()>>,
    graceful_shutdown_complete_receiver: Option<oneshot::Receiver<()>>,
    server_running: bool,
    prometheus_registry: Option<Registry>,
    requester_abort_controllers: Option<Vec<AbortController>>,
    reporter_abort_controllers: Option<Vec<AbortController>>,
}

impl Executor {
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
        let config_str = match read_to_string(config_path.to_string_lossy().to_string().as_str()) {
            Ok(v) => v,
            Err(err) => {
                error!("Config file missing - run 'sonar init' to create one and check that the path is correct. You can add the file without stopping the program. For more debug - enable '-d' debug flag for more info. ");
                debug!("Error: {}", err);
                return;
            }
        };
        match serde_yaml::from_str::<Config>(&config_str) {
            Err(err) => {
                error!("Invalid config - Please fix: {}", err);
                return;
            }
            Ok(config) => {
                info!("Config loaded");

                self.handle_grafana_dashboard(config.clone()).await;
                let request_data_receivers = self.handle_requesters(config.targets.clone()).await;
                if config.server.is_some() {
                    match config.server.clone() {
                        Some(server_config) => {
                            if server_config.prometheus_endpoint.is_some() {
                                self.handle_prometheus_exporter(
                                    config.targets.clone(),
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

    async fn stop_all(&mut self, abort_controllers: Vec<AbortController>) {
        let mut join_handles = Vec::new();
        for c in abort_controllers {
            join_handles.push(tokio::spawn(async {
                let _ = c.shutdown_gracefully().await;
            }));
        }

        for jh in join_handles.drain(..) {
            let _ = jh.await;
        }
    }

    async fn stop_reporters(&mut self) {
        if self.reporter_abort_controllers.is_some() {
            let controllers = self
                .requester_abort_controllers
                .take()
                .expect("failed to take requester forceful shutdown handlers");
            self.stop_all(controllers).await;
        }
    }

    async fn stop_requesters(&mut self) {
        if self.requester_abort_controllers.is_some() {
            let controllers = self
                .requester_abort_controllers
                .take()
                .expect("failed to take requester forceful shutdown handlers");
            self.stop_all(controllers).await;
        }
    }

    async fn handle_requesters(
        &mut self,
        targets: Vec<Target>,
    ) -> Vec<broadcast::Receiver<Result<EntryDTO, FailureDTO>>> {
        // stop/clean out old requesters
        self.stop_reporters().await;
        self.stop_requesters().await;

        let mut requester_abort_controllers = Vec::new();
        let mut reporter_abort_controllers = Vec::new();
        let mut request_result_rx = Vec::new();
        for target in targets {
            // TODO set the capacity to be number of concurrent requests?
            let (broadcast_tx, _broadcast_rx) = channel::<Result<EntryDTO, FailureDTO>>(1);
            request_result_rx.push(_broadcast_rx);

            println!("{:?}", target);
            // reporters
            if target.log.is_some() {
                let log = target.clone_unwrap_log();
                if log.file.is_some() {
                    let (abort_controller, _, syncronizer) = tokio_shutdown::new();
                    let mut file_reporter = FileReporterTask::new(
                        log.clone_unwrap_file(),
                        broadcast_tx.subscribe(),
                        syncronizer,
                    )
                    .expect("failed to create flat file reporter");

                    reporter_abort_controllers.push(abort_controller);
                    tokio::spawn(async move {
                        file_reporter.run().await;
                    });
                }
            }

            // requesters
            let (abort_controller, abort_reg, syncronizer) = tokio_shutdown::new();
            let requester =
                HttpRequestTask::new(self.http_client.clone(), broadcast_tx, syncronizer);
            requester_abort_controllers.push(abort_controller);
            tokio::spawn(to_abortable_with_registration(abort_reg, async move {
                requester.run(target).await;
            }));
        }

        self.requester_abort_controllers = Some(requester_abort_controllers);

        request_result_rx
    }

    fn handle_prometheus_exporter(
        &mut self,
        targets: Vec<Target>,
        receivers: Vec<broadcast::Receiver<Result<EntryDTO, FailureDTO>>>,
    ) {
        // TODO implement
        // let _process_data = process_collector::ProcessCollector::for_self();
        // registry
        //     .register(Box::new(process_data))
        //     .expect("failed to register process info to registry");

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
            let request_time_opts =
                HistogramOpts::new(timer_name.clone(), String::from("latency in ms"))
                    // TODO replace with bucket in target
                    .buckets(vec![
                        1.0, 10.0, 50.0, 100.0, 200.0, 400.0, 600.0, 800.0, 1000.0, 1200.0, 1400.0,
                        1600.0, 1800.0, 2000.0,
                    ]);
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
                            debug!("failed to read message: {}", err);
                        }
                    }
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
                error!("Missing grafana config");
                return;
            }
        };
        info!(
            "Writting grafana dashboard to {}",
            grafana_config.dashboard_json_output_path
        );
        let mut file = match File::create(grafana_config.dashboard_json_output_path) {
            Ok(f) => f,
            Err(err) => {
                error!("Failed to create grafana dashboard file: {}", err);
                return;
            }
        };
        match file.write_all(to_prometheus_grafana(&config).as_bytes()) {
            Ok(_) => (),
            Err(err) => {
                error!("Failed to create grafana dashboard file: {}", err);
                return;
            }
        }
    }

    async fn start_server(&mut self, config: Config) {
        // start server
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
            info!("Waiting for graceful server shutdown");
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

pub async fn execute<'a>(config_file_path: PathBuf, client: Client) -> Result<(), Box<dyn Error>> {
    let config_file_name = config_file_path
        .file_name()
        .expect("failed to get filename of config");
    let mut abs_config_path = std::fs::canonicalize(
        config_file_path
            .parent()
            .expect("could not get parent path of config file"),
    )
    .expect("could not create absolute path from config_path");
    abs_config_path.push(config_file_name);

    let watch_root = if abs_config_path.is_dir() {
        abs_config_path.as_path()
    } else {
        abs_config_path
            .parent()
            .expect("could not unwrap parent path to config_path")
    };
    debug!(
        "Watching for changes in {}",
        watch_root
            .to_str()
            .expect("could not unwrap watch_roo to str")
    );

    let (tx, rx) = std::sync::mpsc::channel::<DebouncedEvent>();
    let mut config_watcher =
        watcher(tx, std::time::Duration::from_secs(1)).expect("failed to create config watcher");

    config_watcher
        .watch(watch_root, RecursiveMode::NonRecursive)
        .expect("failed to watch config root folder");

    // handle initial start run
    let mut executor = Executor::new(client);
    executor.handle(abs_config_path.clone()).await;
    // handle when changes are made to the config file
    loop {
        match rx.recv() {
            Ok(event) => match event {
                DebouncedEvent::Create(path) | DebouncedEvent::Write(path) => {
                    if !is_config_file(&path, &abs_config_path) {
                        continue;
                    }
                    executor.handle(abs_config_path.clone()).await;
                }
                DebouncedEvent::Remove(path) => {
                    // TODO
                    if !is_config_file(&path, &abs_config_path) {
                        continue;
                    }
                    println!("Not implemtented: handle deleted config");
                }
                _ => (), // debug!("got unknown event"),
            },
            Err(err) => panic!("Failed to listen to config changes: {}", err.to_string()),
        }
    }
}

// todo seems bloaty
fn is_config_file(path: &PathBuf, config_path: &PathBuf) -> bool {
    path.eq(config_path)
}

fn read_to_string(file: &str) -> Result<String, std::io::Error> {
    let path = Path::new(file);
    let mut f = File::open(path)?;
    let mut c = String::new();
    f.read_to_string(&mut c)?;
    Ok(c)
}
