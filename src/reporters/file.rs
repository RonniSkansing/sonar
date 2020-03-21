use crate::messages::{Entry, EntryDTO, Failure, FailureDTO};
use crate::utils::file::Append;
use log::*;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use tokio::sync::mpsc::Receiver;

pub struct FileReporter {
    file: File,
    receiver: Receiver<Result<EntryDTO, FailureDTO>>,
}

impl FileReporter {
    pub fn new(
        location: String,
        receiver: Receiver<Result<EntryDTO, FailureDTO>>,
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
            receiver,
        })
    }

    pub async fn listen(&mut self) {
        loop {
            match self.receiver.recv().await {
                Some(result) => match result {
                    Ok(dto) => {
                        let entry = Entry::from_dto(dto);
                        let line = format!(
                            "{} {} {}",
                            entry.time.timestamp(),
                            entry.response_code,
                            entry.target.url
                        );

                        info!("File Reporter - {}", line);

                        match self.file.write((line + "\n").as_bytes()) {
                            Ok(_) => (),
                            Err(err) => {
                                error!("failed to write to log file: {}", err.to_string());
                                break;
                            }
                        }
                    }
                    Err(dto) => {
                        let entry = Failure::from_dto(dto);
                        let line = format!(
                            "{} {} {}",
                            entry.time.timestamp(),
                            entry.target.url,
                            entry.reason
                        );

                        info!("File Reporter - {}", line);

                        match self.file.write((line + "\n").as_bytes()) {
                            Ok(_) => (),
                            Err(err) => {
                                error!("failed to write to log file: {}", err.to_string());
                                break;
                            }
                        }
                    }
                },
                None => {
                    error!("failed to read - connection was broken");
                    break;
                }
            }
        }

        info!("stopping FileReporter");
    }
}
