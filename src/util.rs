use std::fs;

pub fn read_file(file_path: String) -> String {
    let contents = fs::read_to_string(file_path).expect("Unable to read file");
    contents
}

pub fn write_file(content: String, file_path: String) {
    fs::write(file_path, content).expect("Unable to write file")
}
