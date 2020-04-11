use crate::config::{
    Config, GrafanaConfig, LogFile, ReportOn, RequestStrategy, ServerConfig, ShutdownStrategy,
    Target,
};
use duration_string::DurationString;
use log::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub fn execute() {
    let server = Some(ServerConfig {
        ip: String::from("0.0.0.0"),
        port: 8080,
        health_endpoint: Some(String::from("/health")),
        prometheus_endpoint: Some(String::from("/metrics")),
        // TODO:  prometheus process metrics -> bool
    });
    let grafana_config = GrafanaConfig {
        // TODO make the path default to ./sonar-dashboard.json
        dashboard_path: "/opt/sonar/dashboards/sonar.json".to_string(),
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
            shutdown_strategy: ShutdownStrategy::Graceful,
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
            shutdown_strategy: ShutdownStrategy::Graceful,
            log: LogFile {
                file: String::from("./log/name-example.com.log"),
                report_on: ReportOn::Failure,
            },
        },
    ];

    let default_config = serde_yaml::to_string(&Config {
        server,
        grafana: Some(grafana_config),
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
