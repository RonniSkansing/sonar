use crate::config::{grafana::to_prometheus_grafana, Config};
use crate::messages::{Command, EntryDTO, FailureDTO};
use crate::reporters::file::FileReporter;
use crate::{requesters::http::HttpRequester, server::SonarServer};
use log::*;
use reqwest::Client;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use tokio::spawn;
use tokio::sync::{broadcast::channel, oneshot};
use tokio::task::JoinHandle;

pub struct Executor {
    server_kill_sender: Option<oneshot::Sender<()>>,
    graceful_shutdown_complete_receiver: Option<oneshot::Receiver<()>>,
    server_running: bool,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            server_kill_sender: None,
            graceful_shutdown_complete_receiver: None,
            server_running: false,
        }
    }

    pub async fn handle(&mut self, config_path: PathBuf, config_reloading: bool) {
        let config_str = match read_to_string(config_path.to_string_lossy().to_string().as_str()) {
            Ok(v) => v,
            Err(err) => {
                error!("failed to read config file: {}", err);
                return;
            }
        };
        match serde_yaml::from_str::<Config>(&config_str) {
            Err(err) => {
                error!("Invalid config - Please fix: {}", err);
                return;
            }
            Ok(config) => {
                if !config_reloading {
                    info!("Config loaded");
                } else {
                    info!("Config reloaded");
                }
                self.handle_grafana_dashboard(config.clone()).await;
                self.handle_server(config.clone()).await;
            }
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
            grafana_config.dashboard_path
        );
        let mut file = match File::create(grafana_config.dashboard_path) {
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
        let mut server = SonarServer::new(config, Vec::new());
        let (server_kill_sender, graceful_shutdown_complete_receiver) = server.start(Vec::new());
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
    let abs_config_path = std::fs::canonicalize(config_file_path)
        .expect("could not create absolute path from config_path");
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

    // command dispatcher // TODO fix channel capacity
    // let (sender, _) = tokio::sync::broadcast::channel::<Command<_>>(100);
    let mut executor = Executor::new();
    executor.handle(abs_config_path.clone(), false).await;
    loop {
        match rx.recv() {
            Ok(event) => match event {
                DebouncedEvent::Create(path) | DebouncedEvent::Write(path) => {
                    if !is_config_file(&path, &abs_config_path) {
                        continue;
                    }
                    executor.handle(abs_config_path.clone(), true).await;
                }
                DebouncedEvent::Remove(path) => {
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
    /*
       hotwatch
           .watch(watch_root, move |event: Event| match event {
               Event::Create(path) | Event::Write(path) => {
                   if !is_config_file(&path, &abs_config_path) {
                       return;
                   }
                   let config_str =
                       read_to_string(abs_config_path.to_string_lossy().to_string().as_str())
                           .expect("failed to read config file");
                   match serde_yaml::from_str::<Config>(&config_str) {
                       Err(err) => {
                           info!("Invalid config - Please fix: {}", err);
                           return;
                       }
                       Ok(config) => {
                           info!("Config loaded");

                           // let mut grafana_dashboard_consumer_setup = false;
                           let sender = sender.clone();
                           println!("The End");
                           return;
                           tokio::spawn(async move {
                               if config.grafana.is_some() {
                                   // TODO extract grafana_dashboard_consumer
                                   let mut receiver = sender.subscribe();
                                   //if !grafana_dashboard_consumer_setup {
                                   tokio::spawn(async move {
                                       // grafana_dashboard_consumer_setup = true;
                                       for command in receiver.recv().await {
                                           println!("Got command to do! {:?}", command);
                                       }
                                       /*
                                       // TODO extract dashboard consumer
                                                                       let path = config
                                           .clone()
                                           .grafana
                                           .expect("failed to unwrap grafana config")
                                           .dashboard_path
                                           .clone();
                                       debug!("Writting grafana dashboard file to: {}", path.clone());
                                       File::create(path)
                                           .expect("failed to create grafana dashboard.json file")
                                           .write_all(to_prometheus_grafana(&config).as_bytes())
                                           .expect("failed to write dashboard json to file");
                                           */
                                   });
                                   //}
                                   let command = Command::new_grafana_update(config);
                                   sender
                                       .send(command)
                                       .expect("failed to broadcast update grafana command");
                               }
                           });
                       }
                   }
                   /*
                   let config: Config = serde_yaml::from_str(&config_str)
                       .expect("failed to create yaml from config");
                    */

                   //let config_str = read_to_string(;
                   // if config.grafana.is_some() {}
                   println!("Config file was changed");
                   // TODO
               }
               Event::Remove(path) => {
                   if is_config_file(&path, &abs_config_path) {
                       println!("Config file was deleted! oh noes");
                       // TODO
                   }
               }
               _ => (),
           })
           .expect("Failed to config watch file!");
    */
    tokio::time::delay_for(std::time::Duration::from_secs(160)).await;
    println!("STOPPING!");
    return Ok(());

    /*
    let config_str = read_to_string(config_path.as_str())?;
    let config: Config = serde_yaml::from_str(&config_str)?;
    let server_config = config.server.clone();
    let config_targets = config.targets.clone();
    let mut tasks: Vec<JoinHandle<_>> = vec![];
    let mut receivers = vec![];

    if config.grafana.is_some() {
        let path = config
            .clone()
            .grafana
            .expect("failed to unwrap grafana config")
            .dashboard_path
            .clone();
        debug!("Writting grafana dashboard file to: {}", path.clone());
        File::create(path)
            .expect("failed to create grafana dashboard.json file")
            .write_all(to_prometheus_grafana(&config).as_bytes())
            .expect("failed to write dashboard json to file");
    }

    // TODO send a start signal to all requesters when everything is ready so we do not loose requests
    for target in config.targets {
        // TODO set the capacity to be number of concurrent requests?
        let (sender, recv) = channel::<Result<EntryDTO, FailureDTO>>(100);
        receivers.push(sender.subscribe());
        let reporter_location = target.log.file.clone();

        tasks.push(spawn(async move {
            if reporter_location == "" {
                debug!("Skipping file reporter {}", reporter_location);
            } else {
                debug!("Starting file reporter {}", reporter_location);
                FileReporter::new(reporter_location, recv)
                    .expect("failed to create flat file reporter")
                    .listen()
                    .await;
            }
        }));

        let mut requester = HttpRequester::new(client.clone(), sender);
        let target = target.clone();
        tasks.push(spawn(async move {
            requester.run(target).await;
        }));
    }

    tasks.push(spawn(async move {
        let server = SonarServer::new(server_config, config_targets);
        server.start(receivers).await;
    }));

    for t in tasks.drain(..) {
        t.await.expect("failed to listen to task to completion");
    }

    Ok(())
    */
}

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
