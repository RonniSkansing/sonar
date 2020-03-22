use super::config::Target;
use crate::messages::{EntryDTO, FailureDTO};
use crate::reporters::file::FileReporter;
use crate::requesters::http::HttpRequester;
use log::*;
use reqwest::Client;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use tokio::spawn;
use tokio::sync::mpsc::channel;
use tokio::task::JoinHandle;

pub async fn execute<'a>(client: Client) -> Result<(), Box<dyn Error>> {
    let config_str = read_to_string("./sonar.yaml")?;
    let config: Vec<Target> = serde_yaml::from_str(&config_str)?;

    let mut tasks: Vec<JoinHandle<_>> = vec![];
    for target in config {
        let (sender, recv) = channel::<Result<EntryDTO, FailureDTO>>(100);
        let reporter_location = target.report.location.clone();

        tasks.push(spawn(async move {
            debug!("Starting flat file reporter {}", reporter_location);
            FileReporter::new(reporter_location, recv)
                .expect("failed to create flat file reporter")
                .listen()
                .await;
        }));

        let mut requester = HttpRequester::new(client.clone(), sender);

        let target = target.clone();
        tasks.push(spawn(async move {
            requester.run(target).await;
        }));
    }

    for t in tasks {
        t.await.expect("failed to listen to task to completion");
    }

    Ok(())
}

fn read_to_string(file: &str) -> Result<String, std::io::Error> {
    let path = Path::new(file);
    let mut f = File::open(path)?;
    let mut c = String::new();
    f.read_to_string(&mut c)?;
    Ok(c)
}
