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

    let result = match cli_subcommands {
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
            projects::list_repositories(wide);
            Ok(())
        }
        cli::Subcommands::Open {
            id,
            name,
            path,
            value,
            ide,
            show,
        } => {
            let lookup = if id {
                projects::Lookup::Id
            } else if path {
                projects::Lookup::Path
            } else if name {
                projects::Lookup::Name
            } else {
                projects::Lookup::Auto
            };
            projects::open_project(lookup, &value, &ide, show)
        }
    };

    if let Err(message) = result {
        eprintln!("{} ❌", message);
        std::process::exit(1);
    }
}
