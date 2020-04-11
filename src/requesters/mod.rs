pub mod http {
    use crate::config::{RequestStrategy, Target};
    use crate::messages::{Entry, EntryDTO, Failure, FailureDTO};
    use atomic::AtomicU32;
    use chrono::Utc;
    use duration_string::DurationString;
    use futures::future::{AbortHandle, Abortable};
    use log::*;
    use reqwest::Client;
    use std::sync::atomic::{self, Ordering};
    use std::time::Instant;
    use tokio::sync::broadcast;
    use tokio_shutdown::Syncronizer;

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
            let mut interval =
                tokio::time::interval(DurationString::from(target.interval.clone()).into());
            interval.tick().await;
            let currently_running = std::sync::Arc::from(AtomicU32::new(0));
            match target.request_strategy {
                RequestStrategy::Wait => loop {
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

                    debug!(
                        "Concurrent: {}\tGET {} to Http Requester",
                        currently_running.load(Ordering::SeqCst),
                        target.url
                    );

                    let client = self.client.clone();
                    let sender = self.broadcaster.clone();
                    let target = target.clone();
                    let currently_running = currently_running.clone();

                    let req = client
                        .get(&target.url)
                        .timeout(DurationString::from(target.timeout.clone()).into());

                    let latency = Instant::now();
                    match req.send().await {
                        Ok(res) => {
                            let latency_millis = latency.elapsed().as_millis();
                            let message = Entry::new(
                                Utc::now(),
                                latency_millis,
                                res.status().as_u16(),
                                target.clone(),
                            );
                            sender
                                .send(Ok(message.to_dto()))
                                .expect("Failed to send request result");
                        }
                        Err(err) => {
                            let latency_millis = latency.elapsed().as_millis();

                            let message = Failure::new(
                                Utc::now(),
                                latency_millis,
                                err.to_string(),
                                target.clone(),
                            );
                            match sender.send(Err(message.to_dto())) {
                                Ok(_) => {}
                                Err(err) => {
                                    debug!("Failed to send request result due to: {:?}", err);
                                }
                            }
                        }
                    }
                    currently_running.fetch_sub(1, Ordering::SeqCst);

                    interval.tick().await;
                },
                RequestStrategy::CancelOldest => {
                    unimplemented!("RequestStrategy::CancelOldest not implemented");
                }
            }
        }
    }
}
