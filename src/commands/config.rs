use serde::{Deserialize, Serialize};
use std::time::Duration;
use strum_macros::Display;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SecondMilliDuration {
    pub seconds: u64,
    pub millis: u64,
}

impl Into<Duration> for SecondMilliDuration {
    fn into(self) -> Duration {
        let seconds_in_millis = self.seconds * 1000;
        return Duration::from_millis(seconds_in_millis + self.millis);
    }
}

impl From<Duration> for SecondMilliDuration {
    fn from(d: Duration) -> SecondMilliDuration {
        let duration_millis = d.as_millis();
        let seconds = (duration_millis / 1000) as u64;
        let millis = (duration_millis % 1000) as u64;

        SecondMilliDuration { seconds, millis }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Display)]
pub enum ReportType {
    FILE,
    HTTP,
    HTTPS,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Display)]
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
    pub url: String,
    // how often a request should happen
    pub interval: SecondMilliDuration,
    // if a request hits the timeout it is canceled
    pub timeout: SecondMilliDuration,
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

#[cfg(test)]
mod tests {
    mod config_duration {
        use super::super::*;

        #[test]
        fn test_into_duration() {
            let cd: Duration = SecondMilliDuration {
                seconds: 4,
                millis: 1200,
            }
            .into();
            assert_eq!(5200, cd.as_millis());
        }

        #[test]
        fn test_from_duration() {
            let d = Duration::from_millis(4210);
            let i = SecondMilliDuration::from(d);
            assert_eq!((4, 210), (i.seconds, i.millis));
        }
    }
}
