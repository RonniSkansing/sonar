pub mod http {
    use crate::commands::config::{RequestStrategy, Target};
    use crate::{
        messages::{Entry, EntryDTO, Failure, FailureDTO},
        utils::time::DurationString,
    };
    use atomic::AtomicU32;
    use chrono::Utc;
    use log::*;
    use reqwest::Client;
    use std::sync::atomic::{self, Ordering};
    use std::time::Instant;
    use tokio::sync::broadcast::Sender;

    pub struct HttpRequester {
        client: Client,
        sender: Sender<Result<EntryDTO, FailureDTO>>,
    }

    impl HttpRequester {
        pub fn new(client: Client, sender: Sender<Result<EntryDTO, FailureDTO>>) -> HttpRequester {
            HttpRequester {
                client: client,
                sender: sender.clone(),
            }
        }

        pub async fn run(&mut self, target: Target) {
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
                    if currently_running.load(Ordering::SeqCst) >= target.max_concurrent {
                        warn!("Http Requester - {} - One or more requests are not delivered in time for more concurrent requests. Skipping a tick", target.url);
                        interval.tick().await;
                        continue;
                    }
                    currently_running.fetch_add(1, Ordering::SeqCst);

                    debug!(
                        "Http Requester - Concurrent {} - GET {}",
                        currently_running.load(Ordering::SeqCst),
                        target.url
                    );

                    let client = self.client.clone();
                    let sender = self.sender.clone();
                    let target = target.clone();
                    let currently_running = currently_running.clone();
                    tokio::spawn(async move {
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
                                sender
                                    .send(Err(message.to_dto()))
                                    .expect("Failed to send request result");
                            }
                        }
                        currently_running.fetch_sub(1, Ordering::SeqCst);
                    });
                    interval.tick().await;
                },
                RequestStrategy::CancelOldest => {
                    unimplemented!("RequestStrategy::CancelOldest not implemented");
                }
            }
        }
    }
}

/*

This was suppose to execute tasks, when too many tasks was running at the same time,
it should cancel the oldest not finished task and start a new new.

Right now it kinda defect because it will only clean up the task pool if it's it has a tasks that
needs to be canceled.

use std::sync::atomic::{AtomicU32, Ordering};
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;


async fn concurrent_throttle<F, Output, Fut>(max_concurrent: u32, delay: Duration, t: F)
where
    Fut: futures::Future<Output = Output> + Sync + Send + 'static,
    Output: Sync + Send + 'static,
    F: 'static + Send + Fn() -> Fut,
{
    // main task to hold the concurrent ones
    let counter = Arc::from(AtomicU32::from(0));
    tokio::spawn(async move {
        let tasks = Arc::from(Mutex::from(vec![]));
        loop {
            let (fut, abort) = futures::future::abortable(t());
            let tasks = tasks.clone();
            let n = counter.load(Ordering::SeqCst);
            println!("{} concurrent, max is {}", n, max_concurrent);
            if n < max_concurrent {
                let mut tasks = tasks.lock().await;
                let counter = counter.clone();
                let jh = tokio::spawn(async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    let _ = tokio::spawn(fut).await;
                });
                tasks.push((jh, abort));
                tokio::time::delay_for(delay).await;
                continue;
            }
            println!("too many concurrent - removing one from tail");
            let mut tasks = tasks.lock().await;
            // TODO loop all tasks up until the first uncompleted, abort the uncompleted and remove all of them from vec
            let t = tasks.get(0).expect("could not get task at index 0");
            t.1.abort();
            tasks.remove(0);
            counter.fetch_sub(1, Ordering::SeqCst);
        }
    })
    .await
    .unwrap();
}


impl HttpRequester {
            pub async fn task(&mut self) {
        let _ = self.client;
        let sender = self.sender.clone();
        let target = self.target.clone();
        let logger = self.logger.clone();
        // let requests_in_progress = requests_in_progress.clone();
        // let requests_running = requests_running.clone();
        logger.info(format!(
            "HTTP Requester - Sending HTTP request to {}",
            target.host
        ));
        let entry = Entry::new(Utc::now(), 200);
        /*                         match sender.send(entry.to_dto()).await {
            Ok(_) => (),
            Err(_) => {
                logger.info(String::from("Failed to send result"));
            }
        } */
    }
}
*/
