pub mod http {
    use crate::config::{RequestStrategy, Target};
    use crate::messages::{Entry, EntryDTO, Failure, FailureDTO};
    use crate::utils::tokio_shutdown::Syncronizer;
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
        shutdown_sync: Syncronizer,
    }

    impl HttpRequestTask {
        pub fn new(
            client: Client,
            broadcaster: broadcast::Sender<Result<EntryDTO, FailureDTO>>,
            shutdown_notifer: Syncronizer,
        ) -> Self {
            Self {
                client: client,
                broadcaster: broadcaster,
                shutdown_sync: shutdown_notifer,
            }
        }

        pub async fn run(mut self, target: Target) {
            debug!(
                "Starting requester for {} with strategy {}",
                target.url, target.request_strategy
            );
            println!(
                "Ticking with interval of: {}s",
                target.interval.clone().to_string()
            );
            let mut interval =
                tokio::time::interval(DurationString::from(target.interval.clone()).into());
            interval.tick().await;
            let currently_running = std::sync::Arc::from(AtomicU32::new(0));
            match target.request_strategy {
                RequestStrategy::Wait => loop {
                    // TODO bug, this only starts to hear about a graceful shutdown
                    // on next request iteration.. imaging there was 200s between,
                    // it would take forever
                    if self.shutdown_sync.should_stop() {
                        info!("graceful shutdown of requests to {}", target.url);
                        self.shutdown_sync.done().await;
                        break;
                    }

                    if currently_running.load(Ordering::SeqCst) >= target.max_concurrent {
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
                        .timeout(DurationString::from(target.timeout.clone()).into());

                    let latency = Instant::now();
                    let task = async move {
                        debug!("Sending GET {}", target.url.clone());
                        match req.send().await {
                            Ok(res) => {
                                let latency_millis = latency.elapsed().as_millis();
                                info!("200\t{}ms\t{}", latency_millis, target.url.clone());
                                let message = Entry::new(
                                    Utc::now(),
                                    latency_millis,
                                    res.status().as_u16(),
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
                },
                RequestStrategy::CancelOldest => {
                    unimplemented!("RequestStrategy::CancelOldest not implemented");
                }
            }
        }
    }
}
