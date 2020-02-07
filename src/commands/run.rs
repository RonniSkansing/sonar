use super::config::{ReportFormat, ReportType, Target, TargetType};
use crate::reporters::file::FileReporter;
use crate::requesters::http::HttpRequester;
use crate::Logger;
use reqwest::Client;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::mpsc::channel;
use tokio::spawn;
use tokio::task::JoinHandle;

pub async fn execute<'a>(logger: Logger, client: &Client) {
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

    let mut tasks: Vec<JoinHandle<_>> = vec![];
    for target in config {
        let (sender, recv) = channel();
        let reporter_location = target.report.location.clone();

        match target.report.format {
            ReportFormat::FLAT => match target.report.r#type {
                ReportType::FILE => {
                    // reporter
                    let logger = logger.clone();
                    tasks.push(spawn(async move {
                        logger.info(format!("Starting flat file reporter {}", reporter_location));
                        FileReporter::new(reporter_location, recv, logger)
                            .expect("failed to create flat file reporter")
                            .listen();
                    }));
                }
                ReportType::HTTP => unimplemented!(),
                ReportType::HTTPS => unimplemented!(),
            },
            ReportFormat::JSON => match target.report.r#type {
                ReportType::FILE => unimplemented!(),
                ReportType::HTTP => unimplemented!(),
                ReportType::HTTPS => unimplemented!(),
            },
        };

        match target.r#type {
            TargetType::HTTP => {
                // TODO implement max concurrent requests and timeout limit
                logger.info(format!(
                    "Starting HTTP requester for {} {}",
                    target.name, target.host
                ));
                // let client = client.clone();
                let requester = HttpRequester::new(&client /*, &sender, logger.clone()*/);
                tasks.push(spawn(async move {
                    requester.run(/*target.host.clone()*/);
                    loop {
                        /*
                        let sender = sender.clone();
                        let host = target.host.clone();
                        let interval = target.interval.clone();
                        let logger = logger.clone();
                        let host = host.clone();
                        let client = client.clone();
                        */
                        // tokio::time::delay_for(target.interval).await;
                    }
                }));
            }
            TargetType::HTTPS => unimplemented!(),
            TargetType::TCP => unimplemented!(),
            TargetType::UDP => unimplemented!(),
            TargetType::IMCP => unimplemented!(),
        }
    }

    for t in tasks {
        t.await.expect("failed to listen to task to completion");
    }
}

fn read_to_string(file: &str) -> Result<String, std::io::Error> {
    let path = Path::new(file);
    let mut f = File::open(path)?;
    let mut c = String::new();
    f.read_to_string(&mut c)?;
    Ok(c)
}
