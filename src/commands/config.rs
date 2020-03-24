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
    pub file: String,
    pub report_on: ReportOn,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Target {
    pub name: String,
    pub url: String,
    // how often a request should happen
    pub interval: String,
    // if a request hits the timeout it is canceled
    pub timeout: String,
    // number of requests that can run concurrently. 2 means that up to 2 requests will be running a the same time
    pub max_concurrent: u32,
    pub log: LogFile,
    // how to handle when max_concurrent and the next interval is hit.
    pub request_strategy: RequestStrategy,
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
