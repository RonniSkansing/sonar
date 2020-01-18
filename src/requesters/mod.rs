pub mod http {
    use crate::messages::Entry;
    use crate::Logger;
    use std::sync::mpsc::Sender;
    use std::thread;
    use std::time::Duration;

    pub struct HttpRequester {
        target: String,
        interval: Duration,
        sender: Sender<Entry>,
        logger: Logger,
    }

    impl HttpRequester {
        pub fn new(
            target: String,
            interval: Duration,
            sender: Sender<Entry>,
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
                self.logger
                    .info(format!("Sending HTTP request to {}", self.target));

                match self.sender.send(Entry { timestamp_ms: 1234 }) {
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
