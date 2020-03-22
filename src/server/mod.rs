/* use crate::commands::config::ServerConfig;
use crate::messages::{EntryDTO, FailureDTO};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Response, Server, StatusCode};
use log::*;
use std::net::SocketAddr;
use std::net::{Ipv4Addr, Ipv6Addr};
use tokio::sync::mpsc::Receiver;

pub struct SonarServer {
    config: ServerConfig,
    receivers: Vec<Receiver<Result<EntryDTO, FailureDTO>>>,
}

impl SonarServer {
    pub fn new(
        config: ServerConfig,
        receivers: Vec<Receiver<Result<EntryDTO, FailureDTO>>>,
    ) -> SonarServer {
        SonarServer { config, receivers }
    }

    pub async fn start(&self) {
        info!(
            "Starting webserver on {}:{}",
            self.config.ip, self.config.port
        );
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

        // The closure inside `make_service_fn` is run for each connection,
        // creating a 'service' to handle requests for that specific connection.
        // The closure inside `make_service_fn` is run for each connection,
        // creating a 'service' to handle requests for that specific connection.
        let make_service = make_service_fn(move |_| {
            // While the state was moved into the make_service closure,
            // we need to clone it here because this closure is called
            // once for every connection.
            //
            // Each connection could send multiple requests, so
            // the `Service` needs a clone to handle later requests.

            async move {
                // This is the `Service` that will handle the connection.
                // `service_fn` is a helper to convert a function that
                // returns a Response into a `Service`.
                Ok::<_, _>(service_fn(move |req| {
                    // Get the current count, and also increment by 1, in a single
                    // atomic operation.
                    async move {
                        let mut response = Response::new(Body::empty());
                        if req.uri().path() == "/metrics" {
                            *response.body_mut() = Body::from("Metrics");
                        } else {
                            *response.status_mut() = StatusCode::NOT_FOUND;
                        }
                        Ok(response)
                    }
                }))
            }
        });
    }
}
 */
