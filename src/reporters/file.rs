use crate::messages::{Entry, EntryDTO, Failure, FailureDTO};
use crate::{config::ReportOn, utils::file::Append};
use log::*;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use tokio::sync::broadcast;
use tokio_shutdown::Syncronizer;

pub struct FileReporterTask {
    file: File,
    receiver: broadcast::Receiver<Result<EntryDTO, FailureDTO>>,
    _shutdown_sync: Syncronizer,
}

impl FileReporterTask {
    pub fn new(
        location: String,
        receiver: broadcast::Receiver<Result<EntryDTO, FailureDTO>>,
        _shutdown_sync: Syncronizer,
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
            _shutdown_sync,
        })
    }

    pub async fn run(&mut self) {
        loop {
            match self.receiver.recv().await {
                Ok(result) => match result {
                    Ok(dto) => {
                        let entry = Entry::from_dto(dto);
                        match entry.target.log.report_on {
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
                            ReportOn::Failure => (),
                        }
                    }
                    Err(dto) => {
                        let entry = Failure::from_dto(dto);
                        match entry.target.log.report_on {
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
