use crate::config::{read_config_file, write_config_file};
use git2::Repository;
use prettytable::{cell, format, row, Table};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub path: String,
    pub git_remote: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Projects {
    pub projects: Vec<Project>,
}

pub fn get_all_projects() -> Vec<Project> {
    let content = read_config_file("projects.yaml");
    let yaml_config: Projects = match serde_yaml::from_str(&content) {
        Ok(projects) => projects,
        Err(..) => panic!("Unable to read config file as yaml 💔"),
    };

    match yaml_config {
        Projects { projects } => projects,
    }
}

pub fn find_project(path: String) -> bool {
    for project in get_all_projects() {
        if project.path == path {
            return true;
        }
    }

    false
}

pub fn add_project(name: &str, remote_url: Option<String>, remote_name: String) {
    let mut projects = get_all_projects();
    let mut remote_url = remote_url;

    match remote_url {
        Some(..) => {}
        None => {
            remote_url = match Repository::open(name) {
                Ok(..) => {
                    Some(find_git_remote(name, remote_name).expect("Unable to find git remote"))
                }
                Err(..) => None,
            };
        }
    }

    if find_project(name.to_string()) {
        println!("Project already added 🙏")
    } else {
        let project: Project = Project {
            path: name.to_string(),
            git_remote: remote_url,
        };
        projects.push(project);
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

    if find_project(name.to_string()) {
        projects.retain(|x| x.path != name);
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

pub fn find_git_remote(path: &str, remote_name: String) -> Option<String> {
    let repo = match Repository::open(path) {
        Ok(repo) => repo,
        Err(..) => panic!("Not a git repo: {}", path),
    };

    let remote = repo
        .find_remote(&remote_name)
        .expect(&format!("Unable to find remote {}", remote_name));

    Some(remote.url().expect("Unable to find remote URL").to_string())
}

pub fn list_repositories() {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER);
    table.set_titles(row!(
        "Path".to_string(),
        "Exists".to_string(),
        "Git".to_string(),
        "Remote".to_string()
    ));
    for project in &get_all_projects() {
        let remote = match &project.git_remote {
            Some(remote) => remote,
            None => "-",
        };
        table.add_row(row!(
            project.path.to_string(),
            check_project_exists(&project.path),
            check_if_git_enabled(&project.path),
            remote
        ));
        // find_git_remote(&project.path, remote_name).expect("Unable to find git remote");
    }

    println!("\n");
    table.printstd();
}
