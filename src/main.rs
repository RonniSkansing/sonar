//!### Internal Event Flow
//!
//!- Sonar started
//!
//!  - Event: Config created/changed/deleted
//!    - Job: Create new grafana dashboard
//!      - 3.Party: Grafana reads new dashboard
//!    - Job: Change Grafana Metrics exporter
//!    - Job: Stop / Start / Change Requesters
//!
//!- Event: Requester completed/failed request
//!  - Job: Logger writes to log file
//!  - Job: Grafana metrics added to exporter client
mod cli;
mod command;
mod config;
mod messages;
mod server;
mod tasks;
mod utils;

use clap::{App, Arg, Shell, SubCommand};
use log::*;
use reqwest::Client;
use simplelog::*;
use std::path::PathBuf;
use tokio::runtime;

const DEFAULT_CONFIG_PATH: &str = "./sonar.yaml";

fn main() {
    let app = App::new(cli::APP_NAME)
        .arg(
            Arg::with_name(cli::DEBUG_ARG_NAME)
                .help(cli::DEBUG_ARG_HELP)
                .short(cli::DEBUG_ARG_SHORT)
                .long(cli::DEBUG_ARG_LONG)
                .takes_value(cli::DEBUG_ARG_TAKES_VALUE),
        )
        .arg(
            Arg::with_name(cli::QUIET_ARG_NAME)
                .help(cli::QUIET_ARG_HELP)
                .short(cli::QUIET_ARG_SHORT)
                .long(cli::QUIET_ARG_LONG)
                .takes_value(cli::QUIET_ARG_TAKES_VALUE),
        )
        .version(cli::APP_VERSION)
        .author(cli::APP_AUTHOR)
        .about(cli::APP_ABOUT)
        .subcommand(
            SubCommand::with_name(command::init::NAME)
                .about(command::init::ABOUT)
                .arg(
                    Arg::with_name(command::init::MAXIMUM_ARG_NAME)
                        .help(command::init::MAXIMUM_ARG_HELP)
                        .short(command::init::MAXIMUM_ARG_SHORT)
                        .long(command::init::MAXIMUM_ARG_LONG)
                        .takes_value(command::init::MAXIMUM_ARG_TAKES_VALUE),
                )
                .arg(
                    Arg::with_name(command::init::FROM_ARG_NAME)
                        .help(command::init::FROM_ARG_HELP)
                        .short(command::init::FROM_ARG_SHORT)
                        .long(command::init::FROM_ARG_LONG)
                        .takes_value(command::init::FROM_ARG_TAKES_VALUE),
                )
                .arg(
                    Arg::with_name(command::init::OVERWRITE_ARG_NAME)
                        .help(command::init::OVERWRITE_ARG_HELP)
                        .short(command::init::OVERWRITE_ARG_SHORT)
                        .long(command::init::OVERWRITE_ARG_LONG)
                        .takes_value(command::init::OVERWRITE_ARG_TAKES_VALUE),
                ),
        )
        .subcommand(
            SubCommand::with_name(command::run::NAME)
                .about(command::run::ABOUT)
                .arg(
                    Arg::with_name(command::run::CONFIG_ARG_NAME)
                        .help(command::run::CONFIG_ARG_HELP)
                        .short(command::run::CONFIG_ARG_SHORT)
                        .long(command::run::CONFIG_ARG_LONG)
                        .takes_value(command::run::CONFIG_ARG_TAKES_VALUE),
                )
                .arg(
                    Arg::with_name(command::run::THREAD_ARG_NAME)
                        .help(command::run::THREAD_ARG_HELP)
                        .short(command::run::THREAD_ARG_SHORT)
                        .long(command::run::THREAD_ARG_LONG)
                        .takes_value(command::run::THREAD_ARG_TAKES_VALUE),
                ),
        )
        .subcommand(
            SubCommand::with_name(command::autocomplete::NAME)
                .about(command::autocomplete::ABOUT)
                .arg(
                    Arg::with_name(command::autocomplete::SHELL_ARG_NAME)
                        .short(command::autocomplete::SHELL_ARG_SHORT)
                        .long(command::autocomplete::SHELL_ARG_LONG)
                        .required(command::autocomplete::SHELL_ARG_REQUIRED)
                        .possible_values(&command::autocomplete::SHELL_POSSIBLE_VALUES)
                        .help(command::autocomplete::SHELL_ARG_HELP),
                ),
        );

    let mut app_clone = app.clone();
    let matches = app.get_matches();

    // config debug
    let is_debug = matches.is_present(cli::DEBUG_ARG_NAME);
    if is_debug {
        std::env::set_var("RUST_BACKTRACE", "full");
    }

    // setup logger
    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![];
    let config = ConfigBuilder::new()
        .add_filter_ignore_str("mio")
        .add_filter_ignore_str("hyper")
        .add_filter_ignore_str("reqwest")
        .add_filter_ignore_str("want")
        .build();
    if !matches.is_present(cli::QUIET_ARG_NAME) {
        let filter: LevelFilter = if is_debug {
            LevelFilter::Trace
        } else {
            LevelFilter::Info
        };

        match TermLogger::new(filter, config.clone(), TerminalMode::Mixed) {
            Some(logger) => loggers.push(logger),
            None => loggers.push(SimpleLogger::new(filter, config.clone())),
        }
    }

    let _ = CombinedLogger::init(loggers).expect("failed to setup logger");

    let mut runtime_builder = runtime::Builder::new();
    // run command
    match matches.subcommand() {
        (name, Some(matches)) if name == command::init::NAME => {
            let from_file = match matches.args.get(command::run::NAME) {
                Some(args) => Some(PathBuf::from(
                    args.vals[0]
                        .clone()
                        .into_string()
                        .expect("failed to get path to file with domain names"),
                )),
                None => None,
            };
            let size = matches
                .args
                .get(command::init::MAXIMUM_ARG_NAME)
                .map(|_| command::init::Size::Maximal)
                .unwrap_or(command::init::Size::Minimal);
            let overwrite = matches
                .args
                .get(command::init::OVERWRITE_ARG_NAME)
                .map(|_| true)
                .unwrap_or(false);

            runtime_builder
                .build()
                .expect("failed to create runtime")
                .block_on(
                    command::init::Command {
                        config: command::init::Config {
                            overwrite,
                            size,
                            from_file,
                        },
                    }
                    .execute(),
                );
        }
        (name, Some(matches)) if name == command::run::NAME => {
            // setup runtime
            let threads_arg_match = matches.args.get(command::run::THREAD_ARG_NAME);
            if threads_arg_match.is_some() {
                let v = threads_arg_match.unwrap();
                let n: usize = v.vals[0]
                    .clone()
                    .into_string()
                    .expect("failed to take osstring into string")
                    .parse()
                    .expect("failed to parse thread number to usize");
                runtime_builder.max_threads(n);
                debug!("Thread pool set to {}", n);
            }

            let config_path = match matches.args.get(command::run::CONFIG_ARG_NAME) {
                Some(arg) => PathBuf::from(
                    arg.vals[0]
                        .clone()
                        .into_string()
                        .expect("failed to get config path"),
                ),
                None => DEFAULT_CONFIG_PATH.into(),
            };
            runtime_builder
                .build()
                .expect("failed to create runtime")
                .block_on(command::run::Command::exercute(config_path, Client::new()))
                .expect("failed to block on run super	");
        }
        (command::autocomplete::NAME, Some(sub_matches)) => {
            let shell: Shell = sub_matches
                .value_of(command::autocomplete::SHELL_ARG_NAME)
                .expect("unable to get shell name")
                .parse()
                .expect("unable to parse SHELL");
            command::autocomplete::Command::execute(app_clone, shell);
        }
        (_, _) => {
            app_clone
                .print_long_help()
                .expect("failed to print error message. Sorry.");
            println!("");
        }
    }
}
