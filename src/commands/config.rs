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

// The strategy determins how requesting will be processed when the requester is asked
// to do more requests concurrently then the max.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Display)]
pub enum RequestStrategy {
    // do not send a new request before one of the current requests are done
    Wait,
    // cancel the longest living unfinished request and start a new one immediatly
    CancelOldest,
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
    // how often a request should happen
    pub interval: Duration,
    // if a request hits the timeout it is canceled
    pub timeout: Duration,
    // number of requests that can run concurrently. 2 means that up to 2 requests will be running a the same time
    pub max_concurrent: u32,
    pub report: Report,
    // how to handle when max_concurrent and the next interval is hit.
    pub request_strategy: RequestStrategy,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub targets: Vec<Target>,
}
