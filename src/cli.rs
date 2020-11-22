use structopt::StructOpt;

#[derive(Debug, PartialEq, StructOpt)]
pub struct Cli {
    #[structopt(
        long = "config-home",
        default_value = "~/.config/projects",
        env = "PROJECTS_CLI_CONFIG_HOME"
    )]
    pub config_home: String,
    #[structopt(subcommand)]
    pub commands: Subcommands,
}

#[derive(Debug, PartialEq, StructOpt)]
#[structopt(about = "Manage git repos")]
pub enum Subcommands {
    Add {
        #[structopt(long)]
        repository: String,
    },
    Remove {
        #[structopt(long)]
        repository: String,
    },
    List {
        #[structopt(long)]
        repositories: bool,
        #[structopt(long)]
        wide: bool,
    },
}

pub fn get_cli_args() -> Cli {
    let cli_args = Cli::from_args();
    cli_args
}

pub fn get_config_home() -> String {
    get_cli_args().config_home
}
