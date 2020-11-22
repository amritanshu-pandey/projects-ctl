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
        cli::Subcommands::Add {
            repository,
            remote_url,
            remote_name,
        } => projects::add_project(
            &config::canonicalise_path(&repository),
            remote_url,
            remote_name,
        ),
        cli::Subcommands::Remove { repository } => {
            projects::remove_project(&config::canonicalise_path(&repository))
        }
        cli::Subcommands::List { repositories, wide } => {
            if repositories {
                if wide {
                    projects::list_repositories();
                } else {
                    projects::list_repositories();
                }
            }
        }
    };
}
