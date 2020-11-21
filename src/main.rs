use std::process::Command;
mod cli;
mod config;
mod projects;
mod util;

fn main() {
    env_logger::init();
    let args = cli::get_cli_args();
    let cli_subcommands = args.commands;

    config::ensure_config_dir_exist(&args.config_home).expect(&format!(
        "Unable to create config home: {}",
        &args.config_home
    ));
    config::ensure_config_file_exist("projects.yaml").expect("Unable to create empty config file");

    match cli_subcommands {
        cli::Subcommands::Add { repository } => config::add_project(&repository),
        cli::Subcommands::Remove { repository } => config::remove_project(&repository),
    };
}
