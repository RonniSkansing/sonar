use crate::config::{
    Config, LogFile, ReportOn, ReportType, ReportingConfig, RequestStrategy, ServerConfig, Target,
};
use duration_string::DurationString;
use log::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub fn execute() {
    let server = ServerConfig {
        ip: String::from("0.0.0.0"),
        port: 8080,
    };
    let grafana_reporting = ReportingConfig {
        r#type: ReportType::Grafana,
        path: Some("/opt/sonar/dashboards/sonar.json".to_string()),
    };
    let targets = vec![
        Target {
            name: String::from("example-com"),
            url: String::from("http://example.com"),
            interval: DurationString::from_string(String::from("1s"))
                .expect("could not create duration string"),
            max_concurrent: 2,
            timeout: DurationString::from_string(String::from("5s"))
                .expect("could not create duration string"),
            request_strategy: RequestStrategy::Wait,
            log: LogFile {
                file: String::from("./log/name-example.com.log"),
                report_on: ReportOn::Success,
            },
        },
        Target {
            name: String::from("www-example-com"),
            url: String::from("http://example.com"),
            interval: DurationString::from_string(String::from("2s"))
                .expect("could not create duration string"),
            max_concurrent: 2,
            timeout: DurationString::from_string(String::from("5s"))
                .expect("could not create duration string"),
            request_strategy: RequestStrategy::Wait,
            log: LogFile {
                file: String::from("./log/name-example.com.log"),
                report_on: ReportOn::Failure,
            },
        },
    ];

    let default_config = serde_yaml::to_string(&Config {
        server,
        reporting: [grafana_reporting].to_vec(),
        targets,
    })
    .expect("unexpected invalid yaml");

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
