pub mod http {
    use crate::commands::config::Target;
    use crate::messages::{Entry, EntryDTO};
    use crate::Logger;
    use chrono::Utc;
    use reqwest::Client;
    use tokio::sync::mpsc::Sender;

    pub struct HttpRequester {
        client: Client,
        sender: Sender<EntryDTO>,
        logger: Logger,
    }

    impl HttpRequester {
        pub fn new(client: Client, sender: Sender<EntryDTO>, logger: Logger) -> HttpRequester {
            HttpRequester {
                client: client,
                sender: sender.clone(),
                logger: logger,
            }
        }

        pub async fn run(&mut self, target: Target) {
            let _ = self.client;
            let mut tasks = vec![]; // std::vec::Vec::with_capacity(target.max_concurrent)
            loop {
                {
                    let mut sender = self.sender.clone();
                    let target = target.clone();
                    let logger = self.logger.clone();
                    tasks.push(tokio::spawn(async move {
                        logger.info(format!(
                            "HTTP Requester - Sending HTTP request to {}",
                            target.host
                        ));
                        let entry = Entry::new(Utc::now(), 200);
                        match sender.send(entry.to_dto()).await {
                            Ok(_) => (),
                            Err(_) => {
                                logger.info(String::from("Failed to send result"));
                            }
                        }
                    }));
                }
                tokio::time::delay_for(target.interval).await;
                for t in tasks.drain(0..) {
                    let _ = t.await;
                }
            }
        }
    }
}
