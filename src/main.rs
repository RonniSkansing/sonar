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

mod commands;
mod config;
mod messages;
mod reporters;
mod requesters;
mod server;
mod utils;

use clap::{App, Arg, Shell, SubCommand};
use log::*;
use reqwest::Client;
use simplelog::*;
use std::path::PathBuf;
use tokio::runtime;

struct Application<'a> {
    name: &'a str,
    author: &'a str,
    version: &'a str,
    about: &'a str,
}

struct SonarCommand<'a> {
    name: &'a str,
    about: &'a str,
    help: &'a str,
}

impl SonarCommand<'_> {
    fn into_clap(&self) -> App {
        SubCommand::with_name(self.name)
            .help(self.help)
            .about(self.about)
    }
}

struct SonarArg<'a> {
    name: &'a str,
    short: &'a str,
    takes_value: &'a bool,
    help: &'a str,
}

impl SonarArg<'_> {
    fn into_clap(&self) -> Arg {
        Arg::with_name(self.name)
            .help(self.help)
            .short(self.short)
            .takes_value(*self.takes_value)
    }
}

fn main() {
    let sonar = Application {
        name: "Sonar",
        author: "",
        version: "0.0.0",
        about: "Portable monitoring",
    };

    let debug_arg = SonarArg {
        name: "debug",
        short: "d",
        takes_value: &false,
        help: "Add a backtrace (if build with symbols)",
    };

    let threads_arg = SonarArg {
        name: "threads",
        short: "t",
        takes_value: &true,
        help: "Max number of threads. The default value is the number of cores available to the system.",
    };

    let config_arg = SonarArg {
        name: "config",
        short: "c",
        takes_value: &true,
        help: "Config file. Example -> sonar run -c ./foo/bar/baz.yaml",
    };

    let quiet_arg = SonarArg {
        name: "quiet",
        short: "q",
        takes_value: &false,
        help: "Quiet",
    };

    let init_command = SonarCommand {
        name: "init",
        about: "Inits a new project in the current directory",
        help: "Inits a new project in the current directory",
    };

    let run_command = SonarCommand {
        name: "run",
        about: "runs the project",
        help: "runs the project",
    };

    let app = App::new(sonar.name)
        .arg(debug_arg.into_clap())
        .arg(quiet_arg.into_clap())
        .version(sonar.version)
        .author(sonar.author)
        .about(sonar.about)
        .subcommand(init_command.into_clap())
        .subcommand(
            run_command
                .into_clap()
                .arg(threads_arg.into_clap())
                .arg(config_arg.into_clap()),
        )
        .subcommand(
            SubCommand::with_name("autocomplete")
                .about("Generates completion scripts for your shell")
                .arg(
                    Arg::with_name("SHELL")
                        .required(true)
                        .possible_values(&["bash", "fish", "zsh"])
                        .help("The shell to generate the script for"),
                ),
        );

    let mut app_clone = app.clone();

    let matches = app.get_matches();

    // config debug
    let is_debug = matches.is_present(debug_arg.name);
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
    if !matches.is_present(quiet_arg.name) {
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

    let _ = CombinedLogger::init(loggers).expect("Failed to setup logger");

    // run command
    match matches.subcommand() {
        // TODO add arg --grafana -g to add the grafana dashboard path, default to ./sonar.json
        (name, Some(_)) if name == init_command.name => commands::init::execute(),

        // TODO use Some(submatches) instead of manually pulling out of subcommand_matches
        (name, Some(_)) if name == run_command.name => {
            let subcommand_matches = matches
                .subcommand_matches(run_command.name)
                .expect("Could not get run command arguments");

            // setup runtime
            let mut runtime_builder = runtime::Builder::new();
            let threads_arg_match = subcommand_matches.args.get(threads_arg.name);
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
            let mut runtime = runtime_builder
                .thread_name("sonar-pool")
                .threaded_scheduler()
                .enable_all()
                .build()
                .expect("Failed to build runtime");

            let default_config_path_match = subcommand_matches.args.get(config_arg.name);
            let default_config_path = if default_config_path_match.is_some() {
                let v = default_config_path_match.expect("failed to get default_config_path match");
                v.vals[0]
                    .clone()
                    .into_string()
                    .expect("failed to take cconfig_path osstring into string")
            } else {
                "./sonar.yaml".to_string()
            };
            debug!("config path: {}", default_config_path);

            runtime
                .block_on(commands::run::execute(
                    PathBuf::from(default_config_path.clone()),
                    Client::new(),
                ))
                .unwrap_or_else(|e| {
                    error!("Failed to run {}", e.to_string());
                });
        }
        ("autocomplete", Some(sub_matches)) => {
            let shell: Shell = sub_matches
                .value_of("SHELL")
                .expect("unable to get value of SHELL")
                .parse()
                .expect("unable to match SHELL");
            app_clone.gen_completions_to("sonar", shell, &mut std::io::stdout());
        }
        (_, _) => {
            app_clone
                .print_long_help()
                .expect("Failed to print error message. Sorry.");
            println!("");
        }
    }
}
