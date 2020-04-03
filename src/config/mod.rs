use duration_string::DurationString;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

// The strategy determins how requesting will be processed when the requester is asked
// to do more requests concurrently then the max.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Display)]
pub enum RequestStrategy {
    // do not send a new request before one of the current requests are done
    Wait,
    // cancel the longest living unfinished request and start a new one immediatly
    CancelOldest,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Display)]
pub enum ReportOn {
    Success,
    Failure,
    Both,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogFile {
    #[serde(default = "LogFile::default_file")]
    pub file: String,
    #[serde(default = "LogFile::default_report_on")]
    pub report_on: ReportOn,
}

impl LogFile {
    fn default() -> Self {
        LogFile {
            file: Self::default_file(),
            report_on: Self::default_report_on(),
        }
    }
    // default to an empty string which equals no logging
    fn default_file() -> String {
        String::from("")
    }

    fn default_report_on() -> ReportOn {
        ReportOn::Failure
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Target {
    pub name: String,
    pub url: String,
    // how often a request should happen
    #[serde(default = "Target::default_interval")]
    pub interval: DurationString,
    // if a request hits the timeout it is canceled
    #[serde(default = "Target::default_timeout")]
    pub timeout: DurationString,
    // number of requests that can run concurrently. 2 means that up to 2 requests will be running a the same time
    #[serde(default = "Target::default_concurrent")]
    pub max_concurrent: u32,
    #[serde(default = "LogFile::default")]
    pub log: LogFile,
    // how to handle when max_concurrent and the next interval is hit.
    #[serde(default = "Target::default_strategy")]
    pub request_strategy: RequestStrategy,
}

impl Target {
    fn default_strategy() -> RequestStrategy {
        RequestStrategy::Wait
    }

    fn default_timeout() -> DurationString {
        DurationString::from_string(String::from("5s"))
            .expect("failed to create from duration string")
    }

    fn default_interval() -> DurationString {
        DurationString::from_string(String::from("1m"))
            .expect("failed to create from duration string")
    }

    fn default_concurrent() -> u32 {
        1
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub ip: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub targets: Vec<Target>,
}
