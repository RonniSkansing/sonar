use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum TargetType {
    HTTP,
    HTTPS,
    UDP,
    TCP,
    IMCP,
}

// TODO Implement macro for implementing display on a enum
impl fmt::Display for TargetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TargetType::HTTP => write!(f, "HTTP"),
            TargetType::HTTPS => write!(f, "HTTPS"),
            TargetType::UDP => write!(f, "UDP"),
            TargetType::TCP => write!(f, "TCP"),
            TargetType::IMCP => write!(f, "IMCP"),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ReportType {
    FILE,
    HTTP,
    HTTPS,
}

// TODO Implement macro for implementing display on a enum
impl fmt::Display for ReportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ReportType::FILE => write!(f, "FILE"),
            ReportType::HTTP => write!(f, "HTTP"),
            ReportType::HTTPS => write!(f, "HTTPS"),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ReportFormat {
    FLAT,
    JSON,
}

// TODO Implement macro for implementing display on a enum
impl fmt::Display for ReportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ReportFormat::FLAT => write!(f, "FLAT"),
            ReportFormat::JSON => write!(f, "JSON"),
        }
    }
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
