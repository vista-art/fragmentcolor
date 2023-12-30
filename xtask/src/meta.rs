use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub fn workspace_root() -> PathBuf {
    let output = Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}

pub fn workspace_crates() -> Vec<String> {
    let crates_path = workspace_root().join("crates");
    let contents = fs::read_dir(crates_path).unwrap();

    contents
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let is_directory = path.is_dir();

            match is_directory {
                true => Some(file_name.to_string()),
                false => None,
            }
        })
        .collect::<Vec<String>>()
}

pub fn crate_root(crate_name: &str) -> PathBuf {
    workspace_root().join(format!("crates/{}", crate_name))
}
