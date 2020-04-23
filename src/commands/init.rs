use crate::config::Config as SonarConfig;
use log::*;
use std::path::{Path, PathBuf};
use tokio::{fs::read_to_string, prelude::*};

pub enum Size {
    Minimal,
    Maximal,
}

pub struct Config {
    pub force: bool,
    pub size: Size,
    pub from_file: Option<PathBuf>,
}

pub struct Command {
    pub config: Config,
}

impl Command {
    pub async fn execute(&self) {
        let config: Result<SonarConfig, std::io::Error> = if self.config.from_file.is_some() {
            let file_path = PathBuf::from(self.config.from_file.clone().unwrap());
            match read_to_string(file_path).await {
                Ok(s) => match self.config.size {
                    Size::Minimal => Ok(SonarConfig::create_with_minimal_fields_with_urls(s)),
                    Size::Maximal => Ok(SonarConfig::create_with_maximum_fields_with_urls(s)),
                },
                Err(err) => Err(err),
            }
        } else {
            match self.config.size {
                Size::Minimal => Ok(SonarConfig::create_with_minimal_fields()),
                Size::Maximal => Ok(SonarConfig::create_with_maximum_fields()),
            }
        };
        match config {
            Ok(c) => {
                let config = serde_yaml::to_string(&c).expect("invalid yaml");
                if self.config_exists().await && !self.config.force {
                    error!("config already exists. Aborting");
                    return;
                }
                self.write(config.as_bytes()).await;
            }
            Err(err) => {
                error!("failed to create config: {}", err);
            }
        }
    }

    async fn config_exists(&self) -> bool {
        if let Ok(_) = tokio::fs::File::open(crate::DEFAULT_CONFIG_PATH).await {
            true
        } else {
            false
        }
    }

    async fn write(&self, config: &[u8]) {
        let path = Path::new(crate::DEFAULT_CONFIG_PATH);
        let display = path.display();

        let mut file = match tokio::fs::File::create(path).await {
            Ok(file) => file,
            Err(reason) => panic!(
                "failed to create config {}: {}",
                display,
                reason.to_string()
            ),
        };
        match file.write_all(config).await {
            Ok(_) => info!("sample sonar.yaml created - Run 'sonar run' to begin monitoring"),
            Err(err) => error!("failed to create config: {}", err.to_string()),
        }
    }
}
