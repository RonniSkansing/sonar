use super::config::{
    Report, ReportFormat, ReportType, RequestStrategy, SecondMilliDuration, Target, TargetType,
};
use log::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::time::Duration;

pub fn execute() {
    let targets = vec![
        Target {
            name: String::from("name-example.com"),
            host: String::from("example.com"),
            r#type: TargetType::HTTP,
            interval: SecondMilliDuration::from(Duration::from_secs(1)),
            max_concurrent: 2,
            timeout: SecondMilliDuration::from(Duration::from_secs(5)),
            request_strategy: RequestStrategy::Wait,
            report: Report {
                r#type: ReportType::FILE,
                format: ReportFormat::FLAT,
                location: String::from("./log/name-example.com.log"),
            },
        },
        Target {
            name: String::from("name2-www.example.com"),
            host: String::from("www.example.com"),

            r#type: TargetType::HTTP,
            interval: SecondMilliDuration::from(Duration::from_secs(1)),
            max_concurrent: 2,
            timeout: SecondMilliDuration::from(Duration::from_secs(5)),
            request_strategy: RequestStrategy::Wait,
            report: Report {
                r#type: ReportType::FILE,
                format: ReportFormat::FLAT,
                location: String::from("./log/name-example.com.log"),
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
