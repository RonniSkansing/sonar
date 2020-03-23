use crate::commands::config::{ServerConfig, Target};
use crate::messages::{EntryDTO, FailureDTO};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Response, Server, StatusCode};
use log::*;
use prometheus::{Counter, Encoder, Opts, Registry, TextEncoder};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::net::{Ipv4Addr, Ipv6Addr};
use tokio::sync::broadcast::Receiver;

pub struct SonarServer {
    config: ServerConfig,
    targets: Vec<Target>,
    registry: Registry,
}

fn replace_name_success(s: String) -> String {
    (s + "_success").replace('-', "_").replace('.', "_")
}

fn replace_name_failure(s: String) -> String {
    (s + "_failure").replace('-', "_").replace('.', "_")
}

impl SonarServer {
    pub fn new(config: ServerConfig, targets: Vec<Target>) -> SonarServer {
        SonarServer {
            config,
            targets,
            registry: Registry::new(),
        }
    }

    fn setup_prometheus(&self, receivers: Vec<Receiver<Result<EntryDTO, FailureDTO>>>) {
        let mut counters: HashMap<String, Counter> = HashMap::new();

        for target in &self.targets {
            let success_name = replace_name_success(target.name.clone());
            let failure_name = replace_name_failure(target.name.clone());
            let success_counter_opts = Opts::new(success_name.clone(), success_name.clone());
            let failure_counter_opts = Opts::new(failure_name.clone(), success_name.clone());
            let success_counter = Counter::with_opts(success_counter_opts).unwrap();
            let failure_counter = Counter::with_opts(failure_counter_opts).unwrap();
            self.registry
                .register(Box::new(success_counter.clone()))
                .unwrap();
            self.registry
                .register(Box::new(failure_counter.clone()))
                .unwrap();
            counters.insert(success_name.clone(), success_counter);
            counters.insert(failure_name.clone(), failure_counter);
        }

        for mut r in receivers {
            let counters = counters.clone();
            tokio::spawn(async move {
                loop {
                    match r.recv().await {
                        Ok(m) => match m {
                            Ok(r) => {
                                counters
                                    .get(&replace_name_success(r.target.name))
                                    .expect("could not find counter by name")
                                    .inc();
                            }
                            Err(err) => {
                                counters
                                    .get(&replace_name_failure(err.target.name))
                                    .expect("could not find counter by name")
                                    .inc();
                            }
                        },
                        Err(err) => {
                            error!("Failed to read message: {}", err);
                        }
                    }
                }
            });
        }
    }

    pub async fn start(&self, receivers: Vec<Receiver<Result<EntryDTO, FailureDTO>>>) {
        self.setup_prometheus(receivers);
        let is_ip_v4 = self.config.ip.contains(".");

        let addr = if is_ip_v4 {
            let ip = self
                .config
                .ip
                .parse::<Ipv4Addr>()
                .expect("Not a valid ip v4");
            SocketAddr::from((ip, self.config.port))
        } else {
            let ip = self
                .config
                .ip
                .parse::<Ipv6Addr>()
                .expect("Not a valid ip v6");
            SocketAddr::from((ip, self.config.port))
        };

        let registry = self.registry.clone();
        let make_service = make_service_fn(move |_| {
            let registry = registry.clone();
            async move {
                Ok::<_, Error>(service_fn(move |req| {
                    let registry = registry.clone();
                    async move {
                        let metric_families = registry.gather();
                        let mut response = Response::new(Body::empty());
                        if req.uri().path() == "/metrics" {
                            let mut buffer = vec![];
                            let encoder = TextEncoder::new();
                            encoder.encode(&metric_families, &mut buffer).unwrap();
                            *response.body_mut() = Body::from(buffer);
                        } else {
                            *response.status_mut() = StatusCode::NOT_FOUND;
                        }
                        Ok::<_, Error>(response)
                    }
                }))
            }
        });

        //self.server = Some(Server::bind(&addr).serve(make_service));
        let server = Server::bind(&addr).serve(make_service);

        info!("Listening on http://{}/metrics", addr);

        if let Err(e) = server.await {
            error!("server error: {}", e);
        }
    }
}
