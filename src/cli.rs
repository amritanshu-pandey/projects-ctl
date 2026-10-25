use crate::config::canonicalise_path;
use std::env;
use structopt::StructOpt;

#[derive(Debug, PartialEq, StructOpt)]
pub struct Cli {
    #[structopt(
        long = "config-home",
        default_value = "~/.config/projects",
        env = "PROJECTS_CTL_CONFIG_HOME"
    )]
    pub config_home: String,
    #[structopt(subcommand)]
    pub commands: Subcommands,
}

#[derive(Debug, PartialEq, StructOpt)]
#[structopt(about = "Manage git repos")]
pub enum Subcommands {
    Add {
        repository: String,
        #[structopt(long)]
        remote_url: Option<String>,
        #[structopt(long, default_value = "origin")]
        remote_name: String,
        #[structopt(long)]
        name: Option<String>,
    },
    Remove {
        repository: String,
    },
    List {
        #[structopt(long)]
        wide: bool,
    },
    Open {
        #[structopt(
            short,
            long,
            help = "Find project by name (default: try ID, then name, then path)"
        )]
        name: bool,
        #[structopt(short, long, help = "Find project by ID")]
        id: bool,
        #[structopt(short, long, help = "Find project by path")]
        path: bool,
        value: String,
        #[structopt(long, env = "PROJECTS_CTL_IDE_PATH", default_value = "code")]
        ide: String,
        #[structopt(short, long, help = "Print path on stdout")]
        show: bool,
    },
}

pub fn get_cli_args() -> Cli {
    // Keep honouring the old variable name used before the PROJECTS_CTL_ prefix
    if env::var_os("PROJECTS_CTL_CONFIG_HOME").is_none() {
        if let Some(value) = env::var_os("PROJECTS_CLI_CONFIG_HOME") {
            env::set_var("PROJECTS_CTL_CONFIG_HOME", value);
        }
    }
    let cli_args = Cli::from_args();
    cli_args
}

pub fn get_config_home() -> String {
    canonicalise_path(&get_cli_args().config_home)
}
