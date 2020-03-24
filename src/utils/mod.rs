pub mod file {
    use std::fs::File;
    use std::fs::OpenOptions;
    use std::path::Path;

    pub trait Append {
        fn create_append<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
            OpenOptions::new()
                .truncate(false)
                .append(true)
                .create(true)
                .open(path.as_ref())
        }
    }
    impl Append for std::fs::File {}
}

pub mod time {
    use serde::{Deserialize, Serialize};
    use std::time::Duration;
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub struct DurationString {
        inner: Duration,
    }

    impl Into<Duration> for DurationString {
        fn into(self) -> Duration {
            self.inner
        }
    }

    impl From<String> for DurationString {
        fn from(duration: String) -> Self {
            let mut format: String = String::from("");
            let mut period: String = String::from("");

            for c in duration.chars() {
                if c.is_numeric() {
                    period.push(c);
                } else {
                    format.push(c);
                }
            }
            match format.as_str() {
                "ms" => DurationString {
                    inner: Duration::from_millis(
                        period
                            .parse::<u64>()
                            .expect("failed to parse time duration"),
                    ),
                },
                "s" => DurationString {
                    inner: Duration::from_secs(
                        period
                            .parse::<u64>()
                            .expect("failed to parse time duration"),
                    ),
                },
                "m" => DurationString {
                    inner: Duration::from_secs(
                        period
                            .parse::<u64>()
                            .expect("failed to parse time duration"),
                    ) * 60,
                },
                "h" => DurationString {
                    inner: Duration::from_secs(
                        period
                            .parse::<u64>()
                            .expect("failed to parse time duration"),
                    ) * 3600,
                },
                "d" => DurationString {
                    inner: Duration::from_secs(
                        period
                            .parse::<u64>()
                            .expect("failed to parse time duration"),
                    ) * 86_400,
                },
                "w" => DurationString {
                    inner: Duration::from_secs(
                        period
                            .parse::<u64>()
                            .expect("failed to parse time duration"),
                    ) * 604_800,
                },
                "y" => DurationString {
                    inner: Duration::from_secs(
                        period
                            .parse::<u64>()
                            .expect("failed to parse time duration"),
                    ) * 31_556_926,
                },
                _ => panic!("missing TimeDuration format - must be [0-9]+(ms|[smhdwy]"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    mod time {
        // use super::super::time::duration_string_to_duration;
        use std::time::Duration;
        /* #[test]
        fn test_duration_string_to_duration() {
            let hundred_millis = "100ms";
            let hundred_millis_duration =
                duration_string_to_duration(hundred_millis.to_string()).unwrap();
            assert_eq!(Duration::from_millis(100), hundred_millis_duration);
        } */
    }
}
