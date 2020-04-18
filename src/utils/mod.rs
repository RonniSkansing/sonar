pub mod factory {
    pub fn none<T>() -> Option<T> {
        None
    }
}

pub mod file {
    use async_trait::async_trait;
    use std::path::Path;
    use tokio::fs::File;
    use tokio::fs::OpenOptions;

    #[async_trait(?Send)]
    pub trait Append {
        async fn create_append<P: AsRef<Path>>(path: P) -> tokio::io::Result<File> {
            OpenOptions::new()
                .truncate(false)
                .append(true)
                .create(true)
                .open(path.as_ref())
                .await
        }
    }
    impl Append for tokio::fs::File {}
}

pub mod prometheus {
    pub fn normalize_name(s: String) -> String {
        s.replace('-', "_").replace('.', "_")
    }

    pub fn counter_success_name(s: String) -> String {
        normalize_name(s + "_success")
    }

    pub fn timer_name(name: String) -> String {
        normalize_name(name + "_time_ms")
    }
}
