use crate::config::{Config, GrafanaConfig, ServerConfig, Target};
use log::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

const DEFAULT_CONFIG_PATH: &str = "./sonar.yaml";

pub fn execute() {
    let default_config = serde_yaml::to_string(&Config {
        server: None,
        grafana: None,
        targets: vec![Target {
            name: None,
            url: String::from("http://example.com"),
            interval: None,
            max_concurrent: None,
            timeout: None,
            log: None,
        }
        .hydrate()],
    })
    .expect("unexpected invalid yaml");

    let path = Path::new(DEFAULT_CONFIG_PATH);
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

// TODO full example
/*
let server = Some(ServerConfig {
    ip: String::from("0.0.0.0"),
    port: 8080,
    health_endpoint: Some(String::from("/health")),
    prometheus_endpoint: Some(String::from("/metrics")),
    // TODO:  prometheus process metrics -> bool
});
let grafana_config = GrafanaConfig {
    // TODO make the path default to ./sonar-dashboard.json
    dashboard_json_output_path: "/opt/sonar/dashboards/sonar.json".to_string(),
};
*/
