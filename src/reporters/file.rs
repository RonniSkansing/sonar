use crate::messages::{Entry, EntryDTO, Failure, FailureDTO};
use crate::{config::ReportOn, utils::file::Append};
use log::*;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use tokio::sync::broadcast;

pub struct FileReporterTask {
    file: File,
    receiver: broadcast::Receiver<Result<EntryDTO, FailureDTO>>,
}

impl FileReporterTask {
    pub fn new(
        location: String,
        receiver: broadcast::Receiver<Result<EntryDTO, FailureDTO>>,
    ) -> Result<FileReporterTask, std::io::Error> {
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

        Ok(FileReporterTask {
            file: file,
            receiver,
        })
    }

    pub async fn run(&mut self) {
        loop {
            match self.receiver.recv().await {
                Ok(result) => match result {
                    Ok(dto) => {
                        let entry = Entry::from_dto(dto);
                        let log = entry.target.clone_unwrap_log();

                        match log.clone_unwrap_report_on() {
                            ReportOn::Success | ReportOn::Both => {
                                let line = format!(
                                    "{} {} {}\n",
                                    entry.time.timestamp(),
                                    entry.response_code,
                                    entry.target.url
                                );
                                match self.file.write(line.as_bytes()) {
                                    Ok(_) => (),
                                    Err(err) => {
                                        error!("failed to write to log file: {}", err.to_string());
                                        return;
                                    }
                                }
                            }
                            // TODO what is happning here?
                            ReportOn::Failure => (),
                        }
                    }
                    Err(dto) => {
                        let entry = Failure::from_dto(dto);
                        let log = entry.target.clone_unwrap_log();

                        match log.clone_unwrap_report_on() {
                            ReportOn::Both | ReportOn::Failure => {
                                let line = format!(
                                    "{} ERR {} {}\n",
                                    entry.time.timestamp(),
                                    entry.target.url,
                                    entry.reason.trim()
                                );
                                match self.file.write((line).as_bytes()) {
                                    Ok(_) => (),
                                    Err(err) => {
                                        error!("failed to write to log file: {}", err.to_string());
                                        return;
                                    }
                                }
                            }
                            ReportOn::Success => (),
                        }
                    }
                },
                Err(err) => {
                    debug!("Closing file reporter: {}", err.to_string());
                    return;
                }
            };
        }
    }
}
