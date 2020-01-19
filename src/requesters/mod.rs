pub mod http {
    use crate::messages::{Entry, EntryDTO};
    use crate::Logger;
    use chrono::Utc;
    use std::sync::mpsc::Sender;
    use std::thread;
    use std::time::Duration;

    pub struct HttpRequester {
        target: String,
        interval: Duration,
        sender: Sender<EntryDTO>,
        logger: Logger,
    }

    impl HttpRequester {
        pub fn new(
            target: String,
            interval: Duration,
            sender: Sender<EntryDTO>,
            logger: Logger,
        ) -> HttpRequester {
            HttpRequester {
                target: target,
                interval: interval,
                sender: sender,
                logger: logger,
            }
        }

        pub fn run(&self) {
            loop {
                self.logger.info(format!(
                    "HTTP Requester - Sending HTTP request to {}",
                    self.target
                ));

                let entry = Entry::new(Utc::now(), 200);

                match self.sender.send(entry.to_dto()) {
                    Ok(_) => (),
                    Err(_) => {
                        self.logger.info(String::from(
                            "Receiver of message unreachable. Closing Requester",
                        ));
                        break;
                    }
                }
                thread::sleep(self.interval);
            }
        }
    }
}
