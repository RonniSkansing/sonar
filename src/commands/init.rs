use super::config::{Report, ReportFormat, ReportType, Target, TargetType};
use crate::Logger;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub fn execute(logger: Logger) {
    let targets = vec![
        Target {
            name: String::from("name-example.com"),
            host: String::from("example.com"),
            r#type: TargetType::HTTP,
            interval: std::time::Duration::from_secs(1),
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
            interval: std::time::Duration::from_secs(1),
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
            reason.description()
        ),
        Ok(file) => file,
    };
    match file.write_all(default_config.as_bytes()) {
        Err(why) => panic!("failed to write config {}: {}", display, why.description()),
        Ok(_) => logger.log(String::from(
            "sample sonar.yaml created - Run 'sonar run' to begin monitoring",
        )),
    }
}
