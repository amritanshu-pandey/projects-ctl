use crate::config::{canonicalise_path, read_config_file, write_config_file};
use colored::*;
use git2::Repository;
use prettytable::{format, row, Table};
use serde::{Deserialize, Serialize};
use std::fs;
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
    // Highest ID ever handed out; missing in configs written before 0.2.0
    #[serde(default)]
    pub last_id: u64,
}

pub enum Lookup {
    Auto,
    Id,
    Name,
    Path,
}

fn load_projects() -> Projects {
    let content = read_config_file("projects.yaml");
    match serde_yaml::from_str(&content) {
        Ok(projects) => projects,
        Err(..) => panic!("Unable to read config file as yaml 💔"),
    }
}

pub fn get_all_projects() -> Vec<Project> {
    load_projects().projects
}

fn save_projects(projects: Projects) -> Result<(), String> {
    let content = serde_yaml::to_string(&projects)
        .map_err(|e| format!("Unable to get yaml string for project config: {}", e))?;
    write_config_file("projects.yaml", &content);
    Ok(())
}

// IDs are never reused, so an ID never silently starts pointing at a different project
fn next_id(config: &Projects) -> u64 {
    let highest = config.projects.iter().map(|p| p.id).max().unwrap_or(0);
    highest.max(config.last_id) + 1
}

pub fn add_project(
    path: &str,
    remote_url: Option<String>,
    remote_name: String,
    name: Option<String>,
) -> Result<(), String> {
    if fs::metadata(path).is_err() {
        return Err(format!("Path does not exist: {}", path));
    }

    let mut config = load_projects();
    let projects = &config.projects;

    if projects.iter().any(|p| p.path == path) {
        return Err(format!("Project with same path already added: {}", path));
    }
    if let Some(proj_name) = &name {
        if projects.iter().any(|p| p.name.as_ref() == Some(proj_name)) {
            return Err(format!(
                "Project with same name already added: {}",
                proj_name
            ));
        }
    }

    let remote_url = remote_url.or_else(|| find_git_remote(path, &remote_name));

    let project: Project = Project {
        path: path.to_string(),
        git_remote: remote_url,
        name: name,
        id: next_id(&config),
    };
    config.last_id = project.id;
    config.projects.push(project);
    save_projects(config)?;
    println!("Project added: {} ✔️", path);
    Ok(())
}

pub fn remove_project(name: &str) -> Result<(), String> {
    let mut config = load_projects();

    if !config.projects.iter().any(|p| p.path == name) {
        return Err(format!("Project not found: {}", name));
    }
    config.projects.retain(|x| x.path != name);
    save_projects(config)?;
    println!("Project deleted: {} ✔️", name);
    Ok(())
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

pub fn find_git_remote(path: &str, remote_name: &str) -> Option<String> {
    let repo = Repository::open(path).ok()?;
    let remote_url = match repo.find_remote(remote_name) {
        Ok(remote) => remote.url().map(|url| url.to_string()),
        Err(..) => {
            eprintln!(
                "Git remote '{}' not found in {}, adding without a remote",
                remote_name, path
            );
            None
        }
    };
    remote_url
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

// An exact match wins over prefix matches, so "api" can still be opened when "api-v2" exists
fn find_matches<'a, F>(projects: &'a [Project], value: &str, field: F) -> Vec<&'a Project>
where
    F: Fn(&Project) -> Option<&str>,
{
    let exact: Vec<&Project> = projects
        .iter()
        .filter(|p| field(p) == Some(value))
        .collect();
    if !exact.is_empty() {
        return exact;
    }
    projects
        .iter()
        .filter(|p| field(p).map_or(false, |f| f.starts_with(value)))
        .collect()
}

fn find_by_id<'a>(projects: &'a [Project], value: &str) -> Vec<&'a Project> {
    match value.parse::<u64>() {
        Ok(id) => projects.iter().filter(|p| p.id == id).collect(),
        Err(..) => Vec::new(),
    }
}

fn find_by_name<'a>(projects: &'a [Project], value: &str) -> Vec<&'a Project> {
    find_matches(projects, value, |p| p.name.as_deref())
}

fn find_by_path<'a>(projects: &'a [Project], value: &str) -> Vec<&'a Project> {
    find_matches(projects, &canonicalise_path(value), |p| {
        Some(p.path.as_str())
    })
}

pub fn open_project(lookup: Lookup, value: &str, ide: &str, show: bool) -> Result<(), String> {
    let projects = get_all_projects();

    let candidates = match lookup {
        Lookup::Id => {
            if value.parse::<u64>().is_err() {
                return Err(format!("Not a valid project ID: {}", value));
            }
            find_by_id(&projects, value)
        }
        Lookup::Name => find_by_name(&projects, value),
        Lookup::Path => find_by_path(&projects, value),
        Lookup::Auto => {
            let mut candidates = find_by_id(&projects, value);
            if candidates.is_empty() {
                candidates = find_by_name(&projects, value);
            }
            if candidates.is_empty() {
                candidates = find_by_path(&projects, value);
            }
            candidates
        }
    };

    match candidates.as_slice() {
        [] => Err(format!("Project not found: {}", value)),
        [project] => launch(project, ide, show),
        _ => {
            let mut message = format!("'{}' matches multiple projects:", value);
            for project in candidates {
                message.push_str(&format!(
                    "\n  {}: {} ({})",
                    project.id,
                    project.path,
                    project.name.as_deref().unwrap_or("-")
                ));
            }
            Err(message)
        }
    }
}

fn launch(project: &Project, ide: &str, show: bool) -> Result<(), String> {
    if show {
        println!("{}", project.path);
        return Ok(());
    }

    println!("Opening project '{}' in program '{}'", project.path, ide);
    // status() inherits the terminal, so terminal editors like vim work too
    let status = Command::new(ide)
        .arg(&project.path)
        .status()
        .map_err(|e| format!("Unable to run '{}': {}", ide, e))?;
    if !status.success() {
        return Err(format!("'{}' exited with {}", ide, status));
    }
    Ok(())
}
