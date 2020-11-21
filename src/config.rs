extern crate shellexpand;
use crate::cli::get_config_home;
use crate::projects::Projects;
use crate::util;
use log;
use std::fs;
use std::path::Path;

pub fn ensure_config_dir_exist(directory: &str) -> std::io::Result<()> {
    let expanded_dir: &str = &shellexpand::tilde(directory);
    match fs::metadata(expanded_dir) {
        Ok(..) => {
            log::debug!("Config home directory exist: {}", expanded_dir);
        }
        Err(..) => {
            log::debug!("Config home directory doesn't exist: {}", expanded_dir);
            println!("Creating config 🏠: {}", expanded_dir);
            fs::create_dir_all(expanded_dir)?;
        }
    }
    Ok(())
}

pub fn ensure_config_file_exist(filename: &str) -> std::io::Result<()> {
    let config_file_home = get_config_home();
    let config_file_pathbuf = Path::new(&config_file_home);
    let config_file_path = config_file_pathbuf.join(filename);

    let config_file = match config_file_path.to_str() {
        Some(filename) => filename,
        None => panic!("Unable to get project config file path!"),
    };

    match fs::metadata(config_file) {
        Ok(..) => {
            log::debug!("Config file exists: {}", config_file);
        }
        Err(..) => {
            log::debug!("Config file doesn't exist: {}", config_file);
            println!("Creating config 🗃: {}", config_file);
            fs::File::create(config_file)?;
            fs::write(config_file, b"---\nprojects: []\n")?;
        }
    }
    Ok(())
}

pub fn read_config_file(filename: &str) -> String {
    let config_file_home = get_config_home();
    let config_file_pathbuf = Path::new(&config_file_home);
    let config_file_path = config_file_pathbuf.join(filename);

    let config_file = match config_file_path.to_str() {
        Some(filename) => filename,
        None => panic!("Unable to get project config file path!"),
    };

    let content = util::read_file(config_file.to_string());
    content
}

pub fn write_config_file(filename: &str, content: &str) {
    let config_file_home = get_config_home();
    let config_file_pathbuf = Path::new(&config_file_home);
    let config_file_path = config_file_pathbuf.join(filename);

    let config_file = match config_file_path.to_str() {
        Some(filename) => filename,
        None => panic!("Unable to get project config file path!"),
    };

    util::write_file(content.to_string(), config_file.to_string());
}

pub fn get_all_projects() -> Vec<String> {
    let content = read_config_file("projects.yaml");
    let yaml_config: Projects = match serde_yaml::from_str(&content) {
        Ok(projects) => projects,
        Err(..) => panic!("Unable to read config file as yaml 💔"),
    };

    match yaml_config {
        Projects { projects } => projects,
        _ => vec![],
    }
}

pub fn add_project(name: &str) {
    let mut projects = get_all_projects();

    if projects.contains(&name.to_string()) {
        println!("Project already added 🙏")
    } else {
        projects.push(name.to_string());
        let projects = Projects { projects: projects };
        let content = match serde_yaml::to_string(&projects) {
            Ok(content) => content,
            Err(..) => panic!("Unable to get yaml string fot project config"),
        };
        write_config_file("projects.yaml", &content);
        println!("Project added: {} ✔", name);
    }
}

pub fn remove_project(name: &str) {
    let mut projects = get_all_projects();

    if projects.contains(&name.to_string()) {
        projects.retain(|x| x != name);
        let projects = Projects { projects: projects };
        let content = match serde_yaml::to_string(&projects) {
            Ok(content) => content,
            Err(..) => panic!("Unable to get yaml string fot project config"),
        };
        write_config_file("projects.yaml", &content);
        println!("Project deleted: {} ✔", name);
    } else {
        println!("Project not found: {} ❌", name);
    }
}
