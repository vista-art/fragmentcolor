use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::PathBuf,
};

pub fn create_temp_files(dir: &tempfile::TempDir, files: &[&str]) {
    for file in files {
        create_temp_file(dir, file);
    }
}

fn create_temp_file(dir: &tempfile::TempDir, filename: &str) {
    let file_path = dir.path().join(filename);
    if let Some(parent) = file_path.parent() {
        create_dir_all(parent).unwrap();
    }
    let mut file = File::create(&file_path).unwrap();
    let content = generate_file_content(file_path, filename);

    writeln!(file, "{}", content).unwrap();
}

pub fn generate_file_content(path: PathBuf, filename: &str) -> String {
    let module = path.parent().unwrap().to_str().unwrap();
    let module = module.replace("\\", "_").replace("/", "_");
    let name = String::from(filename);

    generate_module(&module, &name, true)
}

fn generate_module(_module: &str, name: &str, first: bool) -> String {
    // @TODO generate `use` statements from siblings

    format!(
        "
        pub struct Pub{}Struct {{
            pub pub_{}_pubstruct_field: i32,
            priv_{}_pubstruct_field: i32,
        }};

        struct Priv{}Struct {{
            pub pub_{}_privstruct_field: i32,
            priv_{}_privstruct_field: i32,
        }};

        pub enum Pub{}Enum {{
            Member1,
            Member2,
        }}

        enum Priv{}Enum {{
            Member1,
            Member2,
        }}

        pub fn pub_{}_fn(arg: i32) -> String {{
            String::from(\"pub_{}_fn\")
        }}

        fn priv_{}_fn(arg: i32) -> String {{
            String::from(\"priv_{}_fn\")
        }}

        {}

        {}
        ",
        name,
        name,
        name,
        name,
        name,
        name,
        name,
        name,
        name,
        name,
        name,
        name,
        // modules
        if first {
            format!(
                "pub mod pub_{}_mod {{
                    {}
                }}",
                name,
                generate_module(
                    &format!("pub_{}_mod", name),
                    &format!("pub_{}_mod", name),
                    false
                )
            )
        } else {
            "".to_string()
        },
        if first {
            format!(
                "mod priv_{}_mod {{
                    {}
                }}",
                name,
                generate_module(
                    &format!("priv_{}_mod", name),
                    &format!("priv_{}_mod", name),
                    false
                )
            )
        } else {
            "".to_string()
        },
    )
}
