use crate::messages::{Entry, EntryDTO};
use crate::utils::file::Append;
use crate::Logger;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::mpsc::Receiver;

pub struct FileReporter {
    file: File,
    receiver: Receiver<EntryDTO>,
    logger: Logger,
}

impl FileReporter {
    pub fn new(
        location: String,
        receiver: Receiver<EntryDTO>,
        logger: Logger,
    ) -> Result<FileReporter, std::io::Error> {
        let file_path = Path::new(&location);
        let path = file_path
            .parent()
            .expect("failed to parent folder of log file");

        match std::fs::create_dir(path) {
            Ok(_) => (),
            Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(e) => panic!(e),
        }
        let file = File::create_append(file_path).expect("failed to create or open in write mode");

        Ok(FileReporter {
            file: file,
            receiver: receiver,
            logger: logger,
        })
    }

    pub fn listen(&mut self) {
        loop {
            match self.receiver.recv() {
                Ok(dto) => {
                    let entry = Entry::from_dto(dto);
                    let line = format!("{} {}\n", entry.time.timestamp(), entry.response_code);

                    // TODO extract as trait and implement with macro for all reporters
                    self.logger.info(format!("HTTP Reporter - {}", line));

                    match self.file.write(line.as_bytes()) {
                        Ok(_) => (),
                        Err(err) => {
                            self.logger.error(format!(
                                "failed to write to log file: {}",
                                err.description()
                            ));
                            break;
                        }
                    }
                }
                Err(err) => {
                    self.logger.error(format!(
                        "failed to read from requester: {}",
                        err.description()
                    ));
                    break;
                }
            }
        }

        self.logger.info(String::from("Stopping FileReporter"));
    }
}
