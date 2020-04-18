use crate::messages::{Entry, EntryDTO, Failure, FailureDTO};
use crate::{config::ReportOn, utils::file::Append};
use log::*;

use std::path::Path;
use tokio::fs::File;
use tokio::prelude::*;
use tokio::sync::broadcast;

pub struct FileReporterTask {
    file: File,
    receiver: broadcast::Receiver<Result<EntryDTO, FailureDTO>>,
}

impl FileReporterTask {
    pub async fn new(
        location: String,
        receiver: broadcast::Receiver<Result<EntryDTO, FailureDTO>>,
    ) -> Result<FileReporterTask, tokio::io::Error> {
        let file_path = Path::new(&location);
        let path = file_path
            .parent()
            .expect("failed to parent folder of log file");

        match tokio::fs::create_dir_all(path).await {
            Ok(_) => (),
            Err(ref e) if e.kind() == tokio::io::ErrorKind::AlreadyExists => (),
            Err(e) => panic!(e),
        }

        match tokio::fs::create_dir(path).await {
            Ok(_) => (),
            Err(ref e) if e.kind() == tokio::io::ErrorKind::AlreadyExists => (),
            Err(e) => panic!(e),
        }
        let file = File::create_append(file_path)
            .await
            .expect("failed to create or open in write mode");

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
                                    "{} {}ms {} {}\n",
                                    entry.time.timestamp(),
                                    entry.latency,
                                    entry.response_code,
                                    entry.target.url
                                );
                                match self.file.write(line.as_bytes()).await {
                                    Ok(_) => (),
                                    Err(err) => {
                                        error!("failed to write to log file: {}", err.to_string());
                                        return;
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                    Err(dto) => {
                        let entry = Failure::from_dto(dto);
                        let log = entry.target.clone_unwrap_log();

                        match log.clone_unwrap_report_on() {
                            ReportOn::Both | ReportOn::Failure => {
                                let line = format!(
                                    "{} Failed {}ms {} {}\n",
                                    entry.time.timestamp(),
                                    entry.target.url,
                                    entry.latency,
                                    entry.reason.trim()
                                );
                                match self.file.write((line).as_bytes()).await {
                                    Ok(_) => (),
                                    Err(err) => {
                                        error!("failed to write to log file: {}", err.to_string());
                                        return;
                                    }
                                }
                            }
                            _ => (),
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
