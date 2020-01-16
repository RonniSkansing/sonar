// TODO Rename to init-command.rs
use super::config::{Report, ReportFormat, ReportType, Target, TargetType};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub fn execute() {
    let targets = vec![
        Target {
            name: String::from("name-example.com"),
            host: String::from("example.com"),
            r#type: TargetType::HTTPS,
            interval: std::time::Duration::from_secs(5),
            report: Report {
                r#type: ReportType::FILE,
                format: ReportFormat::FLAT,
            },
        },
        Target {
            name: String::from("name-example.com"),
            host: String::from("example.com"),
            r#type: TargetType::HTTPS,
            interval: std::time::Duration::from_secs(5),
            report: Report {
                r#type: ReportType::FILE,
                format: ReportFormat::FLAT,
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
        Ok(_) => println!("sample sonar.yaml created. Run 'sonar run' to start"),
    }
}
