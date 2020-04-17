use crate::utils::factory;
use duration_string::DurationString;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

pub mod grafana;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Display)]
pub enum ReportOn {
    Success,
    Failure,
    Both,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogFile {
    pub file: String,

    #[serde(default = "factory::none", skip_serializing_if = "Option::is_none")]
    pub report_on: Option<ReportOn>,
}

impl LogFile {
    pub fn clone_unwrap_report_on(&self) -> ReportOn {
        self.report_on.clone().expect("failed to get report_on")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Target {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub url: String,
    // how often a request should happen
    #[serde(
        default = "Target::some_default_interval",
        skip_serializing_if = "Option::is_none"
    )]
    pub interval: Option<DurationString>,
    // if a request hits the timeout it is canceled
    #[serde(
        default = "Target::some_default_timeout",
        skip_serializing_if = "Option::is_none"
    )]
    pub timeout: Option<DurationString>,
    // number of requests that can run concurrently. 2 means that up to 2 requests will be running a the same time
    #[serde(
        default = "Target::some_default_max_concurrent",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_concurrent: Option<u32>,
    #[serde(default = "factory::none", skip_serializing_if = "Option::is_none")]
    pub log: Option<LogFile>,
}

impl Target {
    fn some_default_timeout() -> Option<DurationString> {
        Some(
            DurationString::from_string(String::from("5s"))
                .expect("failed to create from duration string"),
        )
    }

    fn some_default_interval() -> Option<DurationString> {
        Some(
            DurationString::from_string(String::from("1m"))
                .expect("failed to create from duration string"),
        )
    }

    fn some_default_max_concurrent() -> Option<u32> {
        Some(1)
    }

    pub fn normalize_name(name: &String) -> String {
        name.replace("://", "-")
            .chars()
            .map(|c| if c.is_ascii_alphabetic() { c } else { '_' })
            .collect()
    }

    pub fn hydrate(self) -> Self {
        let name = if self.name.is_none() {
            Self::normalize_name(&self.url)
        } else {
            self.name.unwrap()
        };

        Self {
            url: self.url,
            interval: self.interval,
            name: Some(name),
            timeout: self.timeout,
            max_concurrent: self.max_concurrent,
            log: self.log,
        }
    }

    pub fn clone_unwrap_name(&self) -> String {
        self.name.clone().expect("failed to get name")
    }

    pub fn clone_unwrap_interval(&self) -> DurationString {
        self.interval.clone().expect("failed to get timeout")
    }

    pub fn clone_unwrap_timeout(&self) -> DurationString {
        self.timeout.clone().expect("failed to get timeout")
    }

    pub fn unwrap_max_concurrent(&self) -> u32 {
        self.max_concurrent.expect("failed ot get max_concurrent")
    }

    pub fn clone_unwrap_log(&self) -> LogFile {
        self.log.clone().expect("failed to get log")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub ip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prometheus_endpoint: Option<String>,
}

//
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GrafanaConfig {
    pub dashboard_json_output_path: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<ServerConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grafana: Option<GrafanaConfig>,
    pub targets: Vec<Target>,
}
