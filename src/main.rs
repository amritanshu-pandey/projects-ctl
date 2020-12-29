mod cli;
mod config;
mod projects;
mod util;

fn main() {
    env_logger::init();
    let args = cli::get_cli_args();
    let cli_subcommands = args.commands;

    config::ensure_config_dir_exist(&config::canonicalise_path(&args.config_home)).expect(
        &format!("Unable to create config home: {}", &args.config_home),
    );
    config::ensure_config_file_exist("projects.yaml").expect("Unable to create empty config file");

    match cli_subcommands {
        cli::Subcommands::Add {
            repository,
            remote_url,
            remote_name,
            name,
        } => projects::add_project(
            &config::canonicalise_path(&repository),
            remote_url,
            remote_name,
            name,
        ),
        cli::Subcommands::Remove { repository } => {
            projects::remove_project(&config::canonicalise_path(&repository))
        }
        cli::Subcommands::List { wide } => {
            if wide {
                projects::list_repositories(wide);
            } else {
                projects::list_repositories(wide);
            }
        }
        cli::Subcommands::Open {
            id,
            name,
            path,
            value,
            ide,
            show,
        } => {
            if id {
                projects::open_by_id(value, ide, show);
            } else if path {
                projects::open_by_path(value, ide, show);
            } else if name {
                projects::open_by_name(value, ide, show);
            } else {
                projects::open_by_id(value, ide, show);
            }
        }
    };
}
