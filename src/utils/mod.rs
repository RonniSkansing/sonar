pub mod factory {
    pub fn none<T>() -> Option<T> {
        None
    }
}

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
