mod commands;
mod logger;
mod messages;
mod reporters;
mod requesters;
mod utils;

use clap::{App, Arg, SubCommand};
use logger::{LogLevel, Logger};
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

/* TODO : this could be cool ..
impl Into<SubCommand> for SonarCommand {
    fn into(&self) -> SubCommand {

    }
}
*/

fn main() {
    let sonar = Application {
        name: "Sonar",
        author: "",
        version: "0.0.0",
        about: "Portable Surveillance and monitoring",
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

    let verbose_arg = SonarArg {
        name: "verbose",
        short: "v",
        takes_value: &false,
        help: "Verbose",
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
        .arg(verbose_arg.into_clap())
        .arg(threads_arg.into_clap())
        .version(sonar.version)
        .author(sonar.author)
        .about(sonar.about)
        .subcommand(init_command.into_clap())
        .subcommand(run_command.into_clap());

    // TODO can I avoid a clone here?
    let mut app_help = app.clone();

    let matches = app.get_matches();

    // config debug
    if matches.is_present(debug_arg.name) {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    // setup logger
    let mut log_level = LogLevel::NORMAL;
    if matches.is_present(quiet_arg.name) {
        log_level = LogLevel::NONE;
    } else if matches.is_present(verbose_arg.name) {
        log_level = LogLevel::VERBOSE;
    }
    let logger = Logger::new(log_level);

    // setup runtime
    let mut runtime_builder = runtime::Builder::new();
    if matches.is_present(threads_arg.name) {
        let n: usize = matches
            .value_of(threads_arg.name)
            .expect("failed to get threads argument")
            .parse()
            .expect("failed to parse threads argument - invalid format");
        runtime_builder.max_threads(n);
        logger.info(format!("Thread pool set to {}", n))
    }

    let mut runtime = runtime_builder
        .thread_name("sonar-pool")
        .threaded_scheduler()
        .enable_all()
        .build()
        .expect("Failed to build runtime");

    // run command
    match matches.subcommand() {
        (name, Some(_)) if name == init_command.name => commands::init::execute(logger),
        (name, Some(_)) if name == run_command.name => {
            let l = logger.clone();
            runtime.block_on(commands::run::execute(logger));
            l.log(String::from("lol"));
        }
        (_, _) => {
            app_help
                .print_long_help()
                .expect("Failed to print error message. Sorry.");
            println!("");
        }
    }
}
