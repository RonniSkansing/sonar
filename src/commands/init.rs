use crate::config::{
    Config, GrafanaConfig, LogFile, ReportOn, ServerConfig, Target, TargetDefault,
};
use duration_string::DurationString;
use log::*;
use std::path::{Path, PathBuf};
use tokio::prelude::*;

const DEFAULT_CONFIG_PATH: &str = "./sonar.yaml";

pub async fn minimal_config() {
    let config = serde_yaml::to_string(&Config {
        server: None,
        grafana: None,
        targets_defaults: None,
        targets: vec![Target {
            name: None,
            url: String::from("http://example.com"),
            interval: None,
            max_concurrent: None,
            timeout: None,
            log: None,
            prometheus_response_time_bucket: None,
        }
        .hydrate()],
    })
    .expect("unexpected invalid yaml");

    write(config.as_bytes()).await;
}

pub async fn maximal_config() {
    let server = ServerConfig {
        ip: String::from("0.0.0.0"),
        port: 8080,
        health_endpoint: Some(String::from("/health")),
        prometheus_endpoint: Some(String::from("/metrics")),
    };
    let grafana = GrafanaConfig {
        dashboard_json_output_path: "/opt/sonar/dashboards/sonar.json".to_string(),
    };
    let interval =
        DurationString::from_string("10s".to_string()).expect("failed to create interval");
    let timeout = DurationString::from_string("5s".to_string()).expect("failed to create timeout");
    let log = LogFile {
        file: "./log/https-example-com.log".to_string(),
        report_on: Some(ReportOn::Success),
    };
    let url = "https://example.com".to_string();

    let config = serde_yaml::to_string(&Config {
        server: Some(server),
        grafana: Some(grafana),
        targets_defaults: Some(TargetDefault::default()),
        targets: vec![Target {
            name: Some(Target::normalize_name(&url)),
            url,
            interval: Some(interval),
            timeout: Some(timeout),
            max_concurrent: Some(2),
            log: Some(log),
            prometheus_response_time_bucket: Some(vec![100.0, 250.0, 500.0, 1000.0]),
        }],
    })
    .expect("unexpected invalid yaml");

    write(config.as_bytes()).await;
}

pub async fn from_file_with_minimal_config(file_path: PathBuf) {
    match tokio::fs::read_to_string(file_path).await {
        Ok(file) => {
            let mut targets = Vec::new();
            for line in file.split('\n') {
                if line == "" {
                    continue;
                }
                targets.push(
                    Target {
                        name: None,
                        url: line.to_string(),
                        interval: None,
                        max_concurrent: None,
                        timeout: None,
                        log: None,
                        prometheus_response_time_bucket: None,
                    }
                    .hydrate(),
                );
            }

            let config = serde_yaml::to_string(&Config {
                server: None,
                grafana: None,
                targets_defaults: None,
                targets,
            })
            .expect("unexpected invalid yaml");

            write(config.as_bytes()).await;
        }
        Err(err) => error!("failed to read file: {}", err),
    }
}

pub async fn from_file_with_complete_config(file_path: PathBuf) {
    let server = ServerConfig {
        ip: String::from("0.0.0.0"),
        port: 8080,
        health_endpoint: Some(String::from("/health")),
        prometheus_endpoint: Some(String::from("/metrics")),
    };
    let grafana = GrafanaConfig {
        dashboard_json_output_path: "/opt/sonar/dashboards/sonar.json".to_string(),
    };
    match tokio::fs::read_to_string(file_path).await {
        Ok(file) => {
            let mut targets = Vec::new();
            for line in file.split('\n') {
                if line == "" {
                    continue;
                }
                let url = line.to_string();
                let name = Target::normalize_name(&url);
                let interval = DurationString::from_string("10s".to_string())
                    .expect("failed to create interval");
                let timeout = DurationString::from_string("5s".to_string())
                    .expect("failed to create timeout");
                let log = LogFile {
                    file: format!("./log/{}.log", name),
                    report_on: Some(ReportOn::Success),
                };

                targets.push(
                    Target {
                        name: Some(name),
                        url,
                        interval: Some(interval),
                        max_concurrent: Some(1),
                        timeout: Some(timeout),
                        log: Some(log),
                        prometheus_response_time_bucket: Some(vec![100.0, 250.0, 500.0, 1000.0]),
                    }
                    .hydrate(),
                );
            }
            let config = serde_yaml::to_string(&Config {
                server: Some(server),
                grafana: Some(grafana),
                targets_defaults: Some(TargetDefault::default()),
                targets,
            })
            .expect("unexpected invalid yaml");

            write(config.as_bytes()).await;
        }
        Err(err) => error!("failed to read file: {}", err),
    }
}

async fn write(config: &[u8]) {
    let path = Path::new(DEFAULT_CONFIG_PATH);
    let display = path.display();

    let mut file = match tokio::fs::File::create(path).await {
        Ok(file) => file,
        Err(reason) => panic!(
            "failed to create config {}: {}",
            display,
            reason.to_string()
        ),
    };
    match file.write_all(config).await {
        Ok(_) => info!("sample sonar.yaml created - Run 'sonar run' to begin monitoring"),
        Err(err) => error!("failed to create config: {}", err.to_string()),
    }
}
