use crate::config::{read_config_file, write_config_file};
use git2::Repository;
use prettytable::{cell, format, row, Table};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Projects {
    pub projects: Vec<String>,
}

pub fn get_all_projects() -> Vec<String> {
    let content = read_config_file("projects.yaml");
    let yaml_config: Projects = match serde_yaml::from_str(&content) {
        Ok(projects) => projects,
        Err(..) => panic!("Unable to read config file as yaml 💔"),
    };

    match yaml_config {
        Projects { projects } => projects,
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
        println!("Project added: {} ✔️", name);
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
        println!("Project deleted: {} ✔️", name);
    } else {
        println!("Project not found: {} 💔", name);
    }
}

pub fn check_project_exists(path: &str) -> String {
    match fs::metadata(path) {
        Ok(..) => "✔️".to_string(),
        Err(..) => "❌".to_string(),
    }
}

pub fn check_if_git_enabled(path: &str) -> String {
    match Repository::open(path) {
        Ok(..) => "✔️".to_string(),
        Err(..) => "❌".to_string(),
    }
}

pub fn list_repositories() {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);
    table.set_titles(row!(
        "Path".to_string(),
        "Exists".to_string(),
        "Git".to_string()
    ));
    for project in &get_all_projects() {
        table.add_row(row!(
            project.to_string(),
            check_project_exists(project),
            check_if_git_enabled(project)
        ));
    }

    table.printstd();
}
