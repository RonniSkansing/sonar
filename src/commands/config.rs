use serde::{Deserialize, Serialize};
use std::time::Duration;

use strum_macros::Display;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Display)]
pub enum TargetType {
    HTTP,
    HTTPS,
    UDP,
    TCP,
    IMCP,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Display)]
pub enum ReportType {
    FILE,
    HTTP,
    HTTPS,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Display)]
pub enum ReportFormat {
    FLAT,
    JSON,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub r#type: ReportType,
    pub format: ReportFormat,
    pub location: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Target {
    pub name: String,
    pub host: String,
    pub r#type: TargetType,
    pub interval: Duration,
    pub max_concurrent: u32,
    pub report: Report,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub targets: Vec<Target>,
}
