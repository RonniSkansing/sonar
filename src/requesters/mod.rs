pub mod http {
    use crate::{
        config::Target,
        messages::{Entry, EntryDTO, Failure, FailureDTO},
    };
    use atomic::AtomicU32;
    use chrono::Utc;
    use duration_string::DurationString;
    use log::*;
    use reqwest::Client;
    use std::sync::atomic::{self, Ordering};
    use std::time::Instant;
    use tokio::sync::broadcast;

    pub struct HttpRequestTask {
        client: Client,
        broadcaster: broadcast::Sender<Result<EntryDTO, FailureDTO>>,
    }

    impl HttpRequestTask {
        pub fn new(
            client: Client,
            broadcaster: broadcast::Sender<Result<EntryDTO, FailureDTO>>,
        ) -> Self {
            Self {
                client: client,
                broadcaster: broadcaster,
            }
        }

        pub async fn run(self, target: Target) {
            debug!("Starting requester for {}", target.url);
            let mut interval =
                tokio::time::interval(DurationString::from(target.clone_unwrap_interval()).into());
            interval.tick().await;
            let currently_running = std::sync::Arc::from(AtomicU32::new(0));

            loop {
                if currently_running.load(Ordering::SeqCst) >= target.unwrap_max_concurrent() {
                    warn!("HTTP GET - {} - Responses are not delivered in time for more concurrent requests. Skipping a request", target.url);
                    interval.tick().await;
                    continue;
                }
                currently_running.fetch_add(1, Ordering::SeqCst);

                let client = self.client.clone();
                let sender = self.broadcaster.clone();
                let target = target.clone();
                let currently_running = currently_running.clone();

                let req = client
                    .get(&target.url)
                    .timeout(DurationString::from(target.clone_unwrap_timeout()).into());

                let latency = Instant::now();
                let task = async move {
                    debug!("Sending GET {}", target.url.clone());
                    match req.send().await {
                        Ok(res) => {
                            let latency_millis = latency.elapsed().as_millis();
                            let response_code = res.status().as_u16();
                            info!(
                                "{}\t{}ms\t{}",
                                response_code,
                                latency_millis,
                                target.url.clone()
                            );
                            let message = Entry::new(
                                Utc::now(),
                                latency_millis,
                                response_code,
                                target.clone(),
                            );
                            let _ = sender.send(Ok(message.to_dto()));
                        }
                        Err(err) => {
                            let latency_millis = latency.elapsed().as_millis();

                            let message = Failure::new(
                                Utc::now(),
                                latency_millis,
                                err.to_string(),
                                target.clone(),
                            );
                            info!(
                                "Request failure\t{}\t{}",
                                target.url.clone(),
                                err.to_string()
                            );
                            let _ = sender.send(Err(message.to_dto()));
                        }
                    }
                    currently_running.fetch_sub(1, Ordering::SeqCst);
                };
                tokio::spawn(task);

                interval.tick().await;
            }
        }
    }
}
