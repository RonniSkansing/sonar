pub mod http {
    use crate::messages::{Entry, EntryDTO};
    use crate::Logger;
    use chrono::Utc;
    use reqwest::Client;
    use std::sync::mpsc::Sender;

    pub struct HttpRequester {
        client: Client,
        //sender: &'a Sender<EntryDTO>,
        //logger: Logger,
    }

    impl HttpRequester {
        pub fn new(
            client: Client,
            //sender: &'a Sender<EntryDTO>,
            //logger: Logger,
        ) -> HttpRequester {
            HttpRequester {
                client: client,
                //sender: sender.clone(),
                //logger: logger,
            }
        }

        pub fn run(&self /*, target: String*/) {
            /*
            self.logger.info(format!(
                "HTTP Requester - Sending HTTP request to {}",
                target
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
            */
        }
    }
}
