pub mod http {
    use crate::commands::config::Target;
    use crate::messages::{Entry, EntryDTO};
    use chrono::Utc;
    use log::*;
    use reqwest::Client;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::{sync::Arc, time::Duration};
    use tokio::sync::mpsc::Sender;
    use tokio::sync::Mutex;

    /*

    This was suppose to execute tasks, when too many tasks was running at the same time,
    it should cancel the oldest not finished task and start a new new.

    Right now it kinda defect because it will only clean up the task pool if it's it has a tasks that
    needs to be canceled.

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

    pub struct HttpRequester {
        client: Client,
        sender: Sender<EntryDTO>,
    }

    impl HttpRequester {
        pub fn new(client: Client, sender: Sender<EntryDTO>) -> HttpRequester {
            HttpRequester {
                client: client,
                sender: sender.clone(),
            }
        }

        pub async fn run(&mut self, target: Target) {
            loop {
                info!("Requesting {}", target.host);
                let entry = Entry::new(Utc::now(), 200, target.clone());
                match self.sender.send(entry.to_dto()).await {
                    Ok(_) => (),
                    Err(_) => {
                        error!("Failed to send request result");
                    }
                }
                tokio::time::delay_for(target.interval).await;
            }
        }
    }
}
