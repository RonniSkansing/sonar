use crate::config::{
    Config as SonarConfig, GrafanaConfig, LogFile, ReportOn, ServerConfig, Target, TargetDefault,
};
use duration_string::DurationString;
use log::*;
use std::path::{Path, PathBuf};
use tokio::prelude::*;

const DEFAULT_CONFIG_PATH: &str = "./sonar.yaml";
const DEFAULT_SERVER_IP: &str = "0.0.0.0";
const DEFAULT_SERVER_PORT: u16 = 8080;
const DEFAULT_SERVER_HEALTH_ENDPOINT: &str = "/health";
const DEFAULT_SERVER_PROMETHEUS_ENDPOINT: &str = "/metrics";
const DEFAULT_MAX_CONCURRENT: u32 = 1;
const DEFAULT_GRAFANA_JSON_PATH: &str = "/opt/sonar/dashboards/sonar.json";
const DEFAULT_INTERVAL: &str = "1m";
const DEFAULT_TIMEOUT: &str = "5s";

pub enum Size {
    Minimal,
    Maximal,
}

pub struct Config {
    pub force: bool,
    pub size: Size,
    pub from_file: Option<PathBuf>,
}

pub struct InitCommand {
    pub config: Config,
}

impl InitCommand {
    pub async fn create(&self) {
        let config: Result<SonarConfig, std::io::Error> = if self.config.from_file.is_some() {
            let file_path = PathBuf::from(self.config.from_file.clone().unwrap());
            let mut targets = Vec::new();

            match tokio::fs::read_to_string(file_path).await {
                Ok(file) => match self.config.size {
                    Size::Minimal => {
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

                        Ok(SonarConfig {
                            server: None,
                            grafana: None,
                            targets_defaults: None,
                            targets,
                        })
                    }
                    Size::Maximal => {
                        let server = ServerConfig {
                            ip: String::from(DEFAULT_SERVER_IP),
                            port: DEFAULT_SERVER_PORT,
                            health_endpoint: Some(String::from(DEFAULT_SERVER_HEALTH_ENDPOINT)),
                            prometheus_endpoint: Some(String::from(
                                DEFAULT_SERVER_PROMETHEUS_ENDPOINT,
                            )),
                        };
                        let grafana = GrafanaConfig {
                            dashboard_json_output_path: DEFAULT_GRAFANA_JSON_PATH.to_string(),
                        };
                        for line in file.split('\n') {
                            if line == "" {
                                continue;
                            }
                            let url = line.to_string();
                            let name = Target::normalize_name(&url);
                            let interval =
                                DurationString::from_string(DEFAULT_INTERVAL.to_string())
                                    .expect("failed to create interval");
                            let timeout = DurationString::from_string(DEFAULT_TIMEOUT.to_string())
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
                                    prometheus_response_time_bucket: Some(vec![
                                        100.0, 250.0, 500.0, 1000.0,
                                    ]),
                                }
                                .hydrate(),
                            );
                        }
                        Ok(SonarConfig {
                            server: Some(server),
                            grafana: Some(grafana),
                            targets_defaults: Some(TargetDefault::default()),
                            targets,
                        })
                    }
                },
                Err(err) => Err(err),
            }
        } else {
            match self.config.size {
                Size::Minimal => Ok(SonarConfig {
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
                }),
                Size::Maximal => {
                    let server = ServerConfig {
                        ip: String::from(DEFAULT_SERVER_IP),
                        port: DEFAULT_SERVER_PORT,
                        health_endpoint: Some(String::from(DEFAULT_SERVER_HEALTH_ENDPOINT)),
                        prometheus_endpoint: Some(String::from(DEFAULT_SERVER_PROMETHEUS_ENDPOINT)),
                    };
                    let grafana = GrafanaConfig {
                        dashboard_json_output_path: DEFAULT_GRAFANA_JSON_PATH.to_string(),
                    };
                    let interval = DurationString::from_string(DEFAULT_INTERVAL.to_string())
                        .expect("failed to create interval");
                    let timeout = DurationString::from_string(DEFAULT_TIMEOUT.to_string())
                        .expect("failed to create timeout");
                    let log = LogFile {
                        file: "./log/https-example-com.log".to_string(),
                        report_on: Some(ReportOn::Success),
                    };
                    let url = "https://example.com".to_string();

                    Ok(SonarConfig {
                        server: Some(server),
                        grafana: Some(grafana),
                        targets_defaults: Some(TargetDefault::default()),
                        targets: vec![Target {
                            name: Some(Target::normalize_name(&url)),
                            url,
                            interval: Some(interval),
                            timeout: Some(timeout),
                            max_concurrent: Some(DEFAULT_MAX_CONCURRENT),
                            log: Some(log),
                            prometheus_response_time_bucket: Some(vec![
                                100.0, 250.0, 500.0, 1000.0,
                            ]),
                        }],
                    })
                }
            }
        };
        match config {
            Ok(c) => {
                let config = serde_yaml::to_string(&c).expect("invalid yaml");
                // TODO implement force
                self.write(config.as_bytes()).await;
            }
            Err(err) => {
                error!("failed to create config: {}", err);
            }
        }
    }

    async fn write(&self, config: &[u8]) {
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
}
