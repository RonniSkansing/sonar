use crate::messages::Entry;
use crate::utils::file::Append;
use crate::Logger;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::mpsc::Receiver;

pub struct FileReporter {
    file: File,
    receiver: Receiver<Entry>,
    logger: Logger,
}

impl FileReporter {
    pub fn new(
        location: String,
        receiver: Receiver<Entry>,
        logger: Logger,
    ) -> Result<FileReporter, std::io::Error> {
        let file_path = Path::new(&location);
        let path = file_path
            .parent()
            .expect("failed to parent folder of log file");

        if !path.is_dir() {
            std::fs::create_dir_all(path).expect("failed to create path to log file");
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
                Ok(entry) => {
                    self.logger
                        .info(format!("FileReporter got message: {:?}", entry));

                    // TODO Consider better handling of write errors
                    let line = format!("{} CODE DATA\n", entry.timestamp_ms);
                    self.file
                        .write(line.as_bytes())
                        .expect("failed to write to log file");
                }
                // TODO Consider some kind of logging to know why the channel was closed (other than normal hang up)
                Err(_err) => break,
            }
        }

        self.logger.info(String::from("Stopping FileReporter"));
    }
}
