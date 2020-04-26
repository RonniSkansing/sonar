use crate::utils::factory;
use duration_string::DurationString;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

pub mod grafana;

const DEFAULT_MAX_CONCURRENT: u32 = 1;
const DEFAULT_INTERVAL: &str = "1m";
const DEFAULT_TIMEOUT: &str = "5s";

const DEFAULT_SERVER_IP: &str = "0.0.0.0";
const DEFAULT_SERVER_PORT: u16 = 8080;
const DEFAULT_SERVER_HEALTH_ENDPOINT: &str = "/health";
const DEFAULT_SERVER_PROMETHEUS_ENDPOINT: &str = "/metrics";
const DEFAULT_GRAFANA_JSON_PATH: &str = "/opt/sonar/dashboards/sonar.json";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Display)]
pub enum ReportOn {
    Success,
    Failure,
    Both,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogFile {
    pub file: String,
    #[serde(
        default = "LogFile::some_default_report_on",
        skip_serializing_if = "Option::is_none"
    )]
    pub report_on: Option<ReportOn>,
}

impl LogFile {
    pub fn clone_unwrap_report_on(&self) -> ReportOn {
        self.report_on.clone().expect("failed to get report_on")
    }

    pub fn some_default_report_on() -> Option<ReportOn> {
        Some(ReportOn::Both)
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
    #[serde(default = "factory::none", skip_serializing_if = "Option::is_none")]
    pub prometheus_response_time_bucket: Option<Vec<f64>>,

    #[serde(default = "factory::none", skip_serializing_if = "Option::is_none")]
    pub web_hook: Option<String>,
}

impl Target {
    fn some_default_timeout() -> Option<DurationString> {
        Some(
            DurationString::from_string(String::from(DEFAULT_TIMEOUT))
                .expect("failed to create from duration string"),
        )
    }

    fn some_default_interval() -> Option<DurationString> {
        Some(
            DurationString::from_string(String::from(DEFAULT_INTERVAL))
                .expect("failed to create from duration string"),
        )
    }

    fn some_default_max_concurrent() -> Option<u32> {
        Some(DEFAULT_MAX_CONCURRENT)
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
            prometheus_response_time_bucket: self.prometheus_response_time_bucket,
            log: self.log,
            web_hook: self.web_hook,
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

    pub fn clone_unwrap_web_hook(&self) -> String {
        self.web_hook.clone().expect("failed to get web hook")
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

impl<'a> ServerConfig {
    pub fn new<T: Into<String>, P: Into<u16>>(ip: T, port: P) -> Self {
        ServerConfig {
            ip: ip.into(),
            port: port.into(),
            health_endpoint: None,
            prometheus_endpoint: None,
        }
    }

    pub fn health_endpoint<T: Into<String>>(&'a mut self, path: T) -> &'a mut Self {
        self.health_endpoint = Some(path.into());
        self
    }

    pub fn prometheus_endpoint<T: Into<String>>(&'a mut self, path: T) -> &'a mut Self {
        self.prometheus_endpoint = Some(path.into());
        self
    }
}

//
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GrafanaConfig {
    pub dashboard_json_output_path: String,
}

impl GrafanaConfig {
    pub fn new<T: Into<String>>(output_file_path: T) -> Self {
        Self {
            dashboard_json_output_path: output_file_path.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TargetDefault {
    pub prometheus_response_time_bucket: Vec<f64>,
}

impl TargetDefault {
    pub fn default() -> Self {
        Self {
            prometheus_response_time_bucket: Self::default_prometheus_response_time_bucket(),
        }
    }

    pub fn default_prometheus_response_time_bucket() -> Vec<f64> {
        vec![50.0, 100.0, 150.0, 200.0, 250.0, 300.0, 350.0, 400.0, 500.0]
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WebHook {
    url: String,
    #[serde(
        default = "WebHook::some_default_timeout",
        skip_serializing_if = "Option::is_none"
    )]
    timeout: Option<DurationString>,
}

impl WebHook {
    pub fn some_default_timeout() -> Option<DurationString> {
        Some(DurationString::from_string("3s".to_string()).unwrap())
    }

    pub fn new<T: Into<String>>(url: T) -> Self {
        Self {
            url: url.into(),
            timeout: None,
        }
    }

    pub fn set_timeout(&mut self, d: Option<DurationString>) -> &mut Self {
        self.timeout = d;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<ServerConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grafana: Option<GrafanaConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub targets_defaults: Option<TargetDefault>,
    pub targets: Vec<Target>,
}

impl Config {
    pub fn create_with_minimal_fields() -> Self {
        Self {
            server: None,
            grafana: None,
            targets_defaults: None,
            targets: vec![Target {
                name: None,
                url: String::from("http://example.com"),
                interval: None,
                max_concurrent: None,
                timeout: None,
                log: None,
                prometheus_response_time_bucket: None,
                web_hook: None,
            }
            .hydrate()],
        }
    }

    pub fn create_with_maximum_fields() -> Self {
        let server = ServerConfig::new(DEFAULT_SERVER_IP, DEFAULT_SERVER_PORT)
            .health_endpoint(DEFAULT_SERVER_HEALTH_ENDPOINT)
            .prometheus_endpoint(DEFAULT_SERVER_PROMETHEUS_ENDPOINT)
            .to_owned();
        let grafana = GrafanaConfig::new(DEFAULT_GRAFANA_JSON_PATH);
        let interval = DurationString::from_string(DEFAULT_INTERVAL.to_string())
            .expect("failed to create interval");
        let timeout = DurationString::from_string(DEFAULT_TIMEOUT.to_string())
            .expect("failed to create timeout");
        let log = LogFile {
            file: "./log/https-example-com.log".to_string(),
            report_on: Some(ReportOn::Success),
        };
        let url = "https://example.com".to_string();

        Self {
            server: Some(server),
            grafana: Some(grafana),
            targets_defaults: Some(TargetDefault::default()),
            targets: vec![Target {
                name: Some(Target::normalize_name(&url)),
                url,
                interval: Some(interval),
                timeout: Some(timeout),
                max_concurrent: Some(DEFAULT_MAX_CONCURRENT),
                log: Some(log),
                prometheus_response_time_bucket: Some(vec![100.0, 250.0, 500.0, 1000.0]),
                web_hook: Some("http://localhost:12345".to_string()),
            }],
        }
    }

    // takes a string of newline seperated domains to create a minimal config from
    pub fn create_with_minimal_fields_with_urls(urls: String) -> Self {
        let targets = urls
            .split('\n')
            .into_iter()
            .filter(|l| l == &"")
            .map(|l| {
                Target {
                    name: None,
                    url: l.to_string(),
                    interval: None,
                    max_concurrent: None,
                    timeout: None,
                    log: None,
                    prometheus_response_time_bucket: None,
                    web_hook: None,
                }
                .hydrate()
            })
            .collect();

        Self {
            server: None,
            grafana: None,
            targets_defaults: None,
            targets,
        }
    }

    // takes a string of newline seperated domains to create a minimal config from
    pub fn create_with_maximum_fields_with_urls(urls: String) -> Self {
        let server = ServerConfig::new(DEFAULT_SERVER_IP, DEFAULT_SERVER_PORT)
            .health_endpoint(DEFAULT_SERVER_HEALTH_ENDPOINT)
            .prometheus_endpoint(DEFAULT_SERVER_PROMETHEUS_ENDPOINT)
            .to_owned();
        let grafana = GrafanaConfig::new(DEFAULT_GRAFANA_JSON_PATH);
        let targets = urls
            .split('\n')
            .into_iter()
            .filter(|l| l == &"")
            .map(|l| {
                let url = l.to_string();
                let name = Target::normalize_name(&url);
                let interval = DurationString::from_string(DEFAULT_INTERVAL.to_string())
                    .expect("failed to create interval");
                let timeout = DurationString::from_string(DEFAULT_TIMEOUT.to_string())
                    .expect("failed to create timeout");
                let log = LogFile {
                    file: format!("./log/{}.log", name),
                    report_on: Some(ReportOn::Success),
                };

                Target {
                    name: Some(name),
                    url,
                    interval: Some(interval),
                    max_concurrent: Some(1),
                    timeout: Some(timeout),
                    log: Some(log),
                    prometheus_response_time_bucket: Some(vec![100.0, 250.0, 500.0, 1000.0]),
                    web_hook: Some("https://localhost:12345".to_string()),
                }
                .hydrate()
            })
            .collect();
        Self {
            server: Some(server),
            grafana: Some(grafana),
            targets_defaults: Some(TargetDefault::default()),
            targets,
        }
    }
}
