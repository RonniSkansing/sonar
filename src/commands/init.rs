use super::config::{Report, ReportOn, RequestStrategy, SecondMilliDuration, Target};
use log::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::time::Duration;

pub fn execute() {
    let targets = vec![
        Target {
            name: String::from("name-example.com"),
            url: String::from("http://example.com"),
            interval: SecondMilliDuration::from(Duration::from_secs(1)),
            max_concurrent: 2,
            timeout: SecondMilliDuration::from(Duration::from_secs(5)),
            request_strategy: RequestStrategy::Wait,
            report: Report {
                location: String::from("./log/name-example.com.log"),
                report_on: ReportOn::Failure,
            },
        },
        Target {
            name: String::from("name2-www.example.com"),
            url: String::from("www.example.com"),
            interval: SecondMilliDuration::from(Duration::from_secs(1)),
            max_concurrent: 2,
            timeout: SecondMilliDuration::from(Duration::from_secs(5)),
            request_strategy: RequestStrategy::Wait,
            report: Report {
                location: String::from("./log/name-example.com.log"),
                report_on: ReportOn::Failure,
            },
        },
    ];
    let default_config = serde_yaml::to_string(&targets).expect("unexpected invalid yaml");

    let config_file_name = "./sonar.yaml";
    let path = Path::new(config_file_name);
    let display = path.display();

    let mut file = match File::create(path) {
        Err(reason) => panic!(
            "failed to create config {}: {}",
            display,
            reason.to_string()
        ),
        Ok(file) => file,
    };
    match file.write_all(default_config.as_bytes()) {
        Err(why) => panic!("failed to write config {}: {}", display, why.to_string()),
        Ok(_) => info!("sample sonar.yaml created - Run 'sonar run' to begin monitoring"),
    }
}
