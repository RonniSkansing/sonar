use super::config::{ReportFormat, ReportType, Target, TargetType};
use crate::reports::file::FileReporter;
use crate::requesters::http::HttpRequester;
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

    for target in config {
        let (sender, recv) = channel();
        let reporter_location = target.report.location.clone();

        match target.report.format {
            ReportFormat::FLAT => match target.report.r#type {
                ReportType::FILE => {
                    // reporter
                    // TODO extract thread logger
                    thread_number += 1;
                    let logger = logger.clone();
                    threads.push(thread::spawn(move || {
                        logger.info(format!(
                            "Thread [{}] Starting flat file report handler at {}",
                            thread_number, reporter_location
                        ));
                        FileReporter::new(reporter_location, recv, logger)
                            .expect("failed to create file reporter")
                            .listen();
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
        let name = target.name.clone();
        let host = target.host.clone();
        let interval = target.interval.clone();
        match target.r#type {
            TargetType::HTTP => {
                // requester
                thread_number += 1;
                let logger = logger.clone();
                threads.push(thread::spawn(move || {
                    logger.info(format!(
                        "Thread [{}] Starting http requester for {} {}",
                        thread_number, name, host
                    ));
                    HttpRequester::new(host, interval, sender, logger).run();
                }));
            }
            TargetType::HTTPS => unimplemented!(),
            TargetType::TCP => unimplemented!(),
            TargetType::UDP => unimplemented!(),
            TargetType::IMCP => unimplemented!(),
        }
    }

    logger.info(format!("# Running with {} threads\n", thread_number));

    for thread in threads {
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
