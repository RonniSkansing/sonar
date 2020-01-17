pub mod factory {
    pub fn none<T>() -> Option<T> {
        None
    }
}

pub mod file {
    use async_trait::async_trait;
    use std::path::{Path, PathBuf};
    use tokio::fs::File;
    use tokio::fs::OpenOptions;
    use tokio::prelude::*;

    // Returns a tuple of the absolute path to the file and absolute path of the parent folder
    pub async fn to_absolute_pair<'a>(file_path: PathBuf) -> (PathBuf, PathBuf) {
        let config_file_name = file_path
            .file_name()
            .expect("failed to get filename to into absolute path");
        let mut abs_file_path = tokio::fs::canonicalize(
            file_path
                .parent()
                .expect("failed to get parent path of file"),
        )
        .await
        .expect("failed to create absolute path from file path");
        abs_file_path.push(config_file_name);

        return if abs_file_path.is_dir() {
            (abs_file_path.clone(), abs_file_path)
        } else {
            (
                abs_file_path.clone(),
                PathBuf::from(
                    abs_file_path
                        .parent()
                        .expect("failed to unwrap parent path from absolute file path"),
                ),
            )
        };
    }

    pub async fn read_to_string(file: &str) -> Result<String, tokio::io::Error> {
        let path = Path::new(file);
        let mut f = File::open(path).await?;
        let mut c = String::new();
        f.read_to_string(&mut c).await?;
        Ok(c)
    }

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
