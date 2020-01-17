use super::config::{ReportFormat, ReportType, Target, TargetType};
use crate::reports::file::FileReporter;
use crate::Logger;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;

pub fn execute(logger: Logger) {
    // read and parse
    let config_str = match read_to_string("./sonar.yaml") {
        // TODO Reconsider the use of panic for missing file
        Err(e) => panic!("failed to read sonar.yaml: {}", e.description()),
        Ok(s) => s,
    };
    let config: Vec<Target> = match serde_yaml::from_str(&config_str) {
        // TODO Reconsider the use of panic for missing file
        Err(e) => panic!("failed to parse config: {}", e.description()),
        Ok(conf) => conf,
    };

    let mut threads = vec![];
    let mut thread_number = 0;

    // handle target requesting
    for target in config {
        let (_, recv) = channel();
        let reporter_location = target.report.location.clone();

        match target.report.format {
            ReportFormat::FLAT => match target.report.r#type {
                ReportType::FILE => {
                    // TODO extract thread logger
                    thread_number += 1;
                    let logger = logger.clone();
                    threads.push(thread::spawn(move || {
                        logger.info(format!(
                            "Starting flat file report handler at {}",
                            reporter_location
                        ));
                        // TODO Improve error handling if FileReporter can not be constructed
                        FileReporter::new(reporter_location, recv).unwrap();
                    }));
                }
                ReportType::HTTP => unimplemented!(),
                ReportType::HTTPS => unimplemented!(),
                // _ => panic!("Unknown report type: {}", &r.r#type),
            },
            ReportFormat::JSON => match target.report.r#type {
                ReportType::FILE => unimplemented!(),
                ReportType::HTTP => unimplemented!(),
                ReportType::HTTPS => unimplemented!(),
                // _ => panic!("Unknown report type: {}", &r.r#type),
            },
        };

        let logger = logger.clone();
        threads.push(thread::spawn(move || {
            logger.log(format!(
                "# Started thread {} with target {}\n",
                thread_number, target.name
            ));

            match target.r#type {
                TargetType::HTTP => unimplemented!(),
                TargetType::HTTPS => loop {
                    logger.info(format!(
                        "[{}] Sending {} request to {}",
                        thread_number, target.r#type, target.host
                    ));
                    thread::sleep(target.interval);
                },
                TargetType::TCP => unimplemented!(),
                TargetType::UDP => unimplemented!(),
                TargetType::IMCP => unimplemented!(),
            }
        }));
    }

    logger.info(format!("# Running with {} threads\n", thread_number));

    for thread in threads {
        // wait for all threads to finish
        let _ = thread.join();
    }
}

fn read_to_string(file: &str) -> Result<String, std::io::Error> {
    let path = Path::new(file);
    let mut f = File::open(path)?;
    let mut c = String::new();
    f.read_to_string(&mut c)?;
    Ok(c)
}
