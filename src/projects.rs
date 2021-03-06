use crate::config::{read_config_file, write_config_file};
use colored::*;
use git2::Repository;
use prettytable::{cell, format, row, Table};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::process::Command;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub path: String,
    pub git_remote: Option<String>,
    pub name: Option<String>,
    pub id: u64,
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

pub fn find_project_by_name(name: String) -> bool {
    for project in get_all_projects() {
        match project.name {
            Some(proj_name) => {
                if proj_name == name {
                    return true;
                }
            }
            None => {}
        }
    }
    return false;
}

pub fn find_project_path_by_id(id: u64) -> Option<String> {
    for project in get_all_projects() {
        if project.id == id {
            return Some(project.path);
        }
    }
    return None;
}

fn find_hash(name: String) -> u64 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish() / 1000000000000000
}

pub fn add_project(
    path: &str,
    remote_url: Option<String>,
    remote_name: String,
    name: Option<String>,
) {
    let mut projects = get_all_projects();
    let mut remote_url = remote_url;

    match remote_url {
        Some(..) => {}
        None => {
            remote_url = match Repository::open(path) {
                Ok(..) => {
                    Some(find_git_remote(path, remote_name).expect("Unable to find git remote"))
                }
                Err(..) => None,
            };
        }
    }

    if find_project(path.to_string()) {
        println!("Project with same path already added 🙏")
    } else {
        match &name {
            Some(proj_name) => {
                if find_project_by_name(proj_name.to_string()) {
                    println!("Project with same name already added 🙏");
                    return ();
                }
            }
            None => {}
        };

        let project: Project = Project {
            path: path.to_string(),
            git_remote: remote_url,
            name: name,
            id: find_hash(path.to_string()),
        };
        projects.push(project);
        let projects = Projects { projects: projects };
        let content = match serde_yaml::to_string(&projects) {
            Ok(content) => content,
            Err(..) => panic!("Unable to get yaml string fot project config"),
        };
        write_config_file("projects.yaml", &content);
        println!("Project added: {} ✔️", path);
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

pub fn list_repositories(wide: bool) {
    if wide {
        let mut table = Table::new();
        let mut counter: i32 = 0;
        table.set_format(*format::consts::FORMAT_NO_BORDER);
        table.set_titles(row!(
            "Id".to_string(),
            "Path".to_string(),
            "Name".to_string(),
            "Exists".to_string(),
            "Git".to_string(),
            "Remote".to_string()
        ));
        for project in &get_all_projects() {
            counter = counter + 1;
            let name = match &project.name {
                Some(name) => name,
                None => "-",
            };
            let remote = match &project.git_remote {
                Some(remote) => remote,
                None => "-",
            };

            table.add_row(row!(
                project.id.to_string().blue(),
                project.path.to_string(),
                name.green(),
                check_project_exists(&project.path),
                check_if_git_enabled(&project.path),
                remote
            ));
        }
        if counter > 0 {
            println!("\n");
            table.printstd();
            println!("\n")
        }
    } else {
        for project in &get_all_projects() {
            let name = match &project.name {
                Some(name) => name,
                None => "-",
            };
            println!(
                "{}: {} ({})",
                project.id.to_string().blue(),
                project.path.to_string().italic(),
                name.green()
            )
        }
    }
}

pub fn open_by_id(value: String, ide: String, show: bool) {
    match find_project_path_by_id(
        value
            .parse::<u64>()
            .expect("Unable to parse the ID as integer"),
    ) {
        Some(path) => {
            if show {
                println!("{}", path);
            } else {
                println!("Path: {}", &path);
                Command::new(&ide)
                    .arg(&path)
                    .output()
                    .expect("failed to execute process");
                println!("Opening project '{}' in program '{}' ✔️", path, ide);
            }
        }
        None => {
            println!("Project not found ❌");
        }
    }
}

pub fn open_by_name(value: String, ide: String, show: bool) {
    let mut similar_projects: Vec<Project> = Vec::new();
    for project in get_all_projects() {
        match &project.name {
            Some(proj_name) => {
                if proj_name.starts_with(&value) {
                    similar_projects.push(project);
                }
            }
            None => {}
        };
    }

    if similar_projects.len() == 1 {
        if show {
            println!("{}", similar_projects[0].path)
        } else {
            Command::new(&ide)
                .arg(&similar_projects[0].path)
                .output()
                .expect("failed to execute process");
            println!(
                "Opening project '{}' in program '{}' ✔️",
                similar_projects[0].path, ide
            );
        }
    } else {
        println!("Project not found ❌");
    }
}

pub fn open_by_path(value: String, ide: String, show: bool) {
    let mut similar_projects: Vec<Project> = Vec::new();
    for project in get_all_projects() {
        if project.path.starts_with(&value) {
            similar_projects.push(project);
        }
    }

    if similar_projects.len() == 1 {
        if show {
            println!("{}", similar_projects[0].path)
        } else {
            Command::new(&ide)
                .arg(&similar_projects[0].path)
                .output()
                .expect("failed to execute process");
            println!(
                "Opening project '{}' in program '{}' ✔️",
                similar_projects[0].path, ide
            );
        }
    } else {
        println!("Project not found ❌");
    }
}
