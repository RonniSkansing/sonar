// TODO - Implement a init
// TODO - Read the project file or bail
// TODO - Implement proxy via. HTTP_PROXY / HTTPS_PROXY

use clap::{App, SubCommand};
mod commands;

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
        return SubCommand::with_name(self.name)
            .help(self.help)
            .about(self.about);
    }
}

struct SonarArg<'a> {
    name: &'a str,
    short: &'a str,
    takes_value: &'a bool,
    help: &'a str,
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

    let mut app = App::new(sonar.name)
        .version(sonar.version)
        .author(sonar.author)
        .about(sonar.about)
        .subcommand(init_command.into_clap())
        .subcommand(run_command.into_clap());

    // TODO - can I avoid the clone?
    match app.clone().get_matches().subcommand() {
        (name, Some(_)) if name == init_command.name => {
            commands::init::execute();
            // init(); // TODO : this put it into a endless loop
            // println!("called {} {:?}", name);
        }
        (_, Some(_)) => (),
        (&_, None) => (),
    }
}

/* use clap::{App, Arg, SubCommand};
use reqwest::{StatusCode};
use chrono::{Utc, SecondsFormat};

const SINGLE_TARGET_COMMAND: &str = "single-target";
const SINGLE_TARGET_ARG_DOMAIN: &str = "domain";
const SINGLE_TARGET_ARG_DELAY: &str = "delay";

const SINGLE_TARGET_ARG_DELAY_DEFAULT: &str = "1000";

struct MyArg<'a> {
    name: &'a str,
    short: &'a str,
    takes_value: bool,
    help: &'a str // The last field in a struct can have a dynamic size (wtf) try removing the &'a
}

trait Command<'a> {
    const NAME: &'a str;
    const ABOUT: &'a str;

    fn args() -> Vec<MyArg<'a>>;
}

struct SingleRequestcommand {}
impl<'a> Command<'a> for SingleRequestcommand {
    const NAME: &'static str = "single-request";
    const ABOUT: &'static str = "fire a single request";

    fn args() -> Vec<MyArg<'a>> {
        vec!(
            MyArg{
                name: SINGLE_TARGET_ARG_DOMAIN,
                short: "d",
                takes_value: true,
                help: "the domain to ping agaist"
            },
            MyArg{
                name: SINGLE_TARGET_ARG_DOMAIN,
                short: "t",
                takes_value: true,
                help: "the repeat request delay in ms"
            }
        )
    }
}

// TODO Implement verbosity flag
// TODO Implement latency

fn main() {
    //let single_target_arg_delay_help: &str = &format!("the repeat request delay in ms default to {}", SINGLE_TARGET_ARG_DELAY_DEFAULT);

    let app = App::new("Sonar")
        .version("0.1")
        .author("--")
        .about("")
    ;

    [SingleRequestcommand{}].iter().for_each(|c| {
        SubCommand.with_name()
    });

        /*
        .subcommand(
            SubCommand::with_name(SINGLE_TARGET_COMMAND)
                .about("starts requesting against a single target")
                .arg(
                    Arg::with_name(SINGLE_TARGET_ARG_DOMAIN)
                        .short("d")
                        .takes_value(true)
                        .help("the domain to ping agaist"),
                )
                .arg(
                    Arg::with_name(SINGLE_TARGET_ARG_DELAY)
                        .short("t")
                        .takes_value(true)
                        .help(single_target_arg_delay_help)
                )
        );
        */

    match app.get_matches().subcommand() {
        (SINGLE_TARGET_COMMAND, Some(args)) => {
            single_target(args);
        }
        // TODO - What does this cover?
        (_, Some(_)) => panic!("Some ?"),
        // TODO - What is the meaning of &_ ?
        (&_, None) => panic!("None ? "),
    }
}

fn single_target(args: &clap::ArgMatches) {
    let target = match args.value_of(SINGLE_TARGET_ARG_DOMAIN) {
        Some(v) => v,
        None => panic!("Missing domain argument. Supply with -d or --domain")
    };
    let timeout: u32 = args.value_of(SINGLE_TARGET_ARG_DELAY).unwrap_or(SINGLE_TARGET_ARG_DELAY_DEFAULT).parse().unwrap_or_default();
    // TODO - Validate targ(et

    loop {
        match reqwest::get(target) {
            Ok(mut res) => {
                match res.status() {
                    StatusCode::OK => {
                        println!("{} 200 {}", Utc::now().to_rfc3339_opts(SecondsFormat::Millis, false), target);
                    },
                    _ => {
                        println!("Not ok status code");
                    }
                }
            },
            Err(err) => println!("FAILED: {}", err)
        }
        std::thread::sleep_ms(timeout)
    }
}
 */
