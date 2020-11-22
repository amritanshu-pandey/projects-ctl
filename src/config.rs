extern crate shellexpand;
use crate::cli::get_config_home;
use crate::util;
use log;
extern crate path_absolutize;
use path_abs::PathAbs;
use std::fs;
use std::path::Path;

pub fn canonicalise_path(path: &str) -> String {
    let expanded_dir: &str = &shellexpand::tilde(path);

    match fs::metadata(expanded_dir) {
        Ok(..) => {
            let path = match fs::canonicalize(expanded_dir) {
                Ok(path) => path,
                Err(..) => panic!("Unable to canonicalise the path: {}", expanded_dir),
            };
            path.to_str()
                .expect("Unable to convert resolved path to string")
                .to_string()
        }
        Err(..) => {
            let expanded_dir = match PathAbs::new(expanded_dir) {
                Ok(path) => path,
                Err(..) => panic!("Unable to convert normalised path to fs::path"),
            };
            expanded_dir
                .as_path()
                .to_str()
                .expect("Unable to convert resolved path to string")
                .to_string()
        }
    }
}

pub fn ensure_config_dir_exist(directory: &str) -> std::io::Result<()> {
    match fs::metadata(directory) {
        Ok(..) => {
            log::debug!("Config home directory exist: {}", directory);
        }
        Err(..) => {
            log::debug!("Config home directory doesn't exist: {}", directory);
            println!("Creating config 🏠: {}", directory);
            fs::create_dir_all(directory)?;
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
