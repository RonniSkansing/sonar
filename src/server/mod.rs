use crate::config::ServerConfig;
use crate::messages::{EntryDTO, FailureDTO};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Response, Server, StatusCode};
use log::*;
use prometheus::{Encoder, Registry, TextEncoder};
use std::net::SocketAddr;
use std::net::{Ipv4Addr, Ipv6Addr};
use tokio::sync::{broadcast::Receiver, oneshot};

pub struct SonarServer {
    config: ServerConfig,
    registry: Option<Registry>,
}

impl SonarServer {
    pub fn new(config: ServerConfig, registry: Option<Registry>) -> SonarServer {
        SonarServer { config, registry }
    }

    // Returns a pair of (shutdown_signal_sender, graceful_shutdown_complete_sender)
    pub fn start(
        &mut self,
        receivers: Vec<Receiver<Result<EntryDTO, FailureDTO>>>,
    ) -> (oneshot::Sender<()>, oneshot::Receiver<()>) {
        // self.setup_prometheus(receivers);
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

        let server_config = self.config.clone();
        let registry = self.registry.clone();
        let make_service = make_service_fn(move |_| {
            let server_config = server_config.clone();
            let registry = registry.clone();
            async move {
                Ok::<_, Error>(service_fn(move |req| {
                    let registry = registry.clone();
                    let server_config = server_config.clone();
                    async move {
                        let mut response = Response::new(Body::empty());

                        if server_config.health_endpoint.is_some() {
                            if req.uri().path()
                                == server_config
                                    .health_endpoint
                                    .expect("failed to unwrap health endpoint path")
                            {
                                debug!("Handling health request");
                                return Ok::<_, Error>(response);
                            }
                        }
                        if server_config.prometheus_endpoint.is_some() {
                            if req.uri().path()
                                == server_config
                                    .prometheus_endpoint
                                    .expect("failed to unwrap prometheus metric endpoint path")
                            {
                                debug!("Handling metric request");
                                let registry =
                                    registry.expect("failed to get registry prometheus export");

                                let metric_families = registry.gather();
                                let mut buffer = vec![];
                                let encoder = TextEncoder::new();
                                encoder
                                    .encode(&metric_families, &mut buffer)
                                    .expect("unable to put metrics in buffer");
                                *response.body_mut() = Body::from(buffer);
                                return Ok::<_, Error>(response);
                            }
                        }

                        *response.status_mut() = StatusCode::NOT_FOUND;
                        Ok::<_, Error>(response)
                    }
                }))
            }
        });

        let bind = Server::bind(&addr);
        let server = bind.serve(make_service);
        let (kill_signal_tx, kill_signal_rx) = oneshot::channel::<()>();
        let (shutdown_complete_tx, shutdown_complete_rx) = oneshot::channel::<()>();
        let server = server.with_graceful_shutdown(async {
            let _ = kill_signal_rx.await;
            debug!("Server got shutdown signal")
        });
        info!("Listening on http://{}/", addr);
        if self.config.health_endpoint.is_some() {
            info!(
                "Health check endpoint at {}",
                self.config
                    .health_endpoint
                    .clone()
                    .expect("failed to get health check endpoint path")
            );
        }
        if self.config.prometheus_endpoint.is_some() {
            info!(
                "Metrics endpoint at {}",
                self.config
                    .prometheus_endpoint
                    .clone()
                    .expect("failed to get metrics endpoint path")
            );
        }
        tokio::spawn(async {
            if let Err(e) = server.await {
                error!("server error: {}", e);
            }
            info!("Server stopped");
            shutdown_complete_tx
                .send(())
                .expect("failed to send graceful shutdown complete");
        });

        return (kill_signal_tx, shutdown_complete_rx);
    }
}
