use crate::commands::config::{ServerConfig, Target};
use crate::messages::{EntryDTO, FailureDTO};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Response, Server, StatusCode};
use log::*;
use prometheus::{Counter, Encoder, Histogram, HistogramOpts, Opts, Registry, TextEncoder};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::net::{Ipv4Addr, Ipv6Addr};
use tokio::sync::broadcast::Receiver;

pub struct SonarServer {
    config: ServerConfig,
    targets: Vec<Target>,
    registry: Registry,
}

fn prometheus_normalize_name(s: String) -> String {
    s.replace('-', "_").replace('.', "_")
}

fn counter_success_name(s: String) -> String {
    prometheus_normalize_name(s + "_success")
}

fn timer_name(name: String) -> String {
    prometheus_normalize_name(name + "_time_ms")
}

impl SonarServer {
    pub fn new(config: ServerConfig, targets: Vec<Target>) -> SonarServer {
        SonarServer {
            config,
            targets,
            registry: Registry::new(),
        }
    }

    // TODO move this out of the webserver
    fn setup_prometheus(&self, receivers: Vec<Receiver<Result<EntryDTO, FailureDTO>>>) {
        let mut timers: HashMap<String, Histogram> = HashMap::new();
        let mut counters: HashMap<String, Counter> = HashMap::new();

        for target in &self.targets {
            let counter_success_name = counter_success_name(target.name.clone());
            let counter_success = Counter::with_opts(Opts::new(
                counter_success_name.clone(),
                String::from("Number of successful requests"),
            ))
            .expect("failed to create success counter");
            counters.insert(counter_success_name.clone(), counter_success.clone());

            let timer_name = timer_name(target.name.clone());
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

            self.registry
                .register(Box::new(request_time))
                .expect("unable to register timer");
            self.registry
                .register(Box::new(counter_success))
                .expect("unable to register timer");
        }

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
                                    .get(&counter_success_name(r.target.name.clone()))
                                    .expect("could not find success counter by key")
                                    .inc();
                                timers
                                    .get(&timer_name(r.target.name.clone()))
                                    .expect("could not find timer by key")
                                    .observe(r.latency as f64);
                            }
                            Err(err) => {
                                timers
                                    .get(&timer_name(err.target.name.clone()))
                                    .expect("could not find timer by name")
                                    .observe(err.latency as f64);
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
                            encoder
                                .encode(&metric_families, &mut buffer)
                                .expect("unable to put metrics in buffer");
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
