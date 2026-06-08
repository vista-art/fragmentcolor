mod meta {
    use std::{
        fs, io,
        path::{Path, PathBuf},
        process::Command,
    };

    /// Returns the workspace root directory for this crate.
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

    /// Write the file only if its contents differ. Returns Ok(true) if written.
    pub fn write_if_changed(path: &Path, contents: &str) -> io::Result<bool> {
        // Safety guard: never write into source docs under <workspace>/docs/api from build.rs
        let is_source_docs_api = {
            let root = workspace_root();
            if let Ok(rel) = path.strip_prefix(&root) {
                use std::path::Component;
                let mut comps = rel.components();
                matches!(
                    (comps.next(), comps.next()),
                    (Some(Component::Normal(a)), Some(Component::Normal(b))) if a == "docs" && b == "api"
                )
            } else {
                false
            }
        };
        if is_source_docs_api {
            panic!("build.rs attempted to write inside docs/api (source docs). This is forbidden.");
        }

        match fs::read_to_string(path) {
            Ok(existing) if existing == contents => Ok(false),
            _ => {
                ensure_parent_dir(path)?;
                fs::write(path, contents)?;
                Ok(true)
            }
        }
    }

    /// Ensure the parent directory for the given file exists.
    pub fn ensure_parent_dir(path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }
}
