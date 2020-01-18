use serde::{Deserialize, Serialize};
use std::time::Duration;

use strum_macros::Display;

#[derive(Debug, PartialEq, Serialize, Deserialize, Display)]
pub enum TargetType {
    HTTP,
    HTTPS,
    UDP,
    TCP,
    IMCP,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Display)]
pub enum ReportType {
    FILE,
    HTTP,
    HTTPS,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Display)]
pub enum ReportFormat {
    FLAT,
    JSON,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub r#type: ReportType,
    pub format: ReportFormat,
    pub location: String,
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
