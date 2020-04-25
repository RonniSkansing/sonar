use crate::cli;
use clap::{App, Shell};
use std::io;

pub const NAME: &str = "autocomplete";
pub const ABOUT: &str = "Generate autocomplete";

pub const SHELL_ARG_SHORT: &str = "s";
pub const SHELL_ARG_LONG: &str = "shell";
pub const SHELL_ARG_REQUIRED: bool = false;
pub const SHELL_ARG_NAME: &str = "shell";
pub const SHELL_ARG_HELP: &str = "Shell name to generate autocomplete for";
pub const SHELL_POSSIBLE_VALUES: [&str; 3] = ["bash", "fish", "zsh"];

pub struct Command {}

impl Command {
    pub fn execute(mut app: App, shell: Shell) {
        app.gen_completions_to(cli::APP_NAME, shell, &mut io::stdout());
    }
}
