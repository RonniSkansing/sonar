pub mod http {
    use crate::messages::{Entry, EntryDTO};
    use crate::Logger;
    use chrono::Utc;
    use std::sync::mpsc::Sender;

    pub struct HttpRequester {
        target: String,
        sender: Sender<EntryDTO>,
        logger: Logger,
    }

    impl HttpRequester {
        pub fn new(target: String, sender: Sender<EntryDTO>, logger: Logger) -> HttpRequester {
            HttpRequester {
                target: target,
                sender: sender,
                logger: logger,
            }
        }

        pub fn run(&self) {
            self.logger.info(format!(
                "HTTP Requester - Sending HTTP request to {}",
                self.target
            ));
            let entry = Entry::new(Utc::now(), 200);
            match self.sender.send(entry.to_dto()) {
                Ok(_) => (),
                Err(_) => {
                    self.logger.info(String::from(
                        "Failed to send result - receiving task of message unreachable.",
                    ));
                }
            }
        }
    }
}
