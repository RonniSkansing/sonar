use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum TargetType {
    HTTP,
    HTTPS,
    UDP,
    TCP,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ReportType {
    FILE,
    HTTP,
    HTTPS,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ReportFormat {
    FLAT,
    JSON,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub r#type: ReportType,
    pub format: ReportFormat,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Target {
    pub name: String,
    pub host: String,
    pub r#type: TargetType,
    pub interval: Duration,
    pub report: Report,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub targets: Vec<Target>,
}
