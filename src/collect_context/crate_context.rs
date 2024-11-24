use std::{
    fs::{self, read_to_string, File},
    io::Write,
    path::PathBuf,
    process,
};

use syn::parse_file;
use toml::Value;

use super::mod_context::{ModContext, ModInfo, ModModInfo};

#[derive(Debug, Clone)]
pub struct CrateContext {
    crate_name: String,
    crate_path: PathBuf,
    entry_file_path: PathBuf,
    mod_context: ModContext,
}

impl CrateContext {
    pub fn new(crate_path: PathBuf) -> Self {
        let mut crate_context = CrateContext {
            crate_name: String::new(),
            crate_path: PathBuf::new(),
            entry_file_path: PathBuf::new(),
            mod_context: ModContext::new(),
        };
        let toml_path = crate_path.join("Cargo.toml");
        if fs::exists(&toml_path).unwrap() {
            let toml_content =
                read_to_string(toml_path).expect("Can not read the Cargo.toml file of the crate!");
            let toml_value: Value = toml_content
                .parse()
                .expect("Failed to parse the Cargo.toml file of the crate!");
            if let Some(package) = toml_value.get("package") {
                if let Some(name) = package.get("name") {
                    crate_context.crate_name = name.as_str().unwrap().to_string();
                } else {
                    eprintln!("Can not get the crate name of the crate!");
                    process::exit(2);
                }
            } else {
                eprintln!("Can not get the package infomation of the crate!");
                process::exit(3);
            }
        } else {
            eprintln!("Can not find the Cargo.toml file of the crate!");
            process::exit(4);
        }
        crate_context.crate_path = crate_path.clone();
        let main_path = crate_path.join("src/main.rs");
        let lib_path = crate_path.join("src/lib.rs");
        if fs::exists(&main_path).unwrap() {
            crate_context.entry_file_path = main_path;
        } else if fs::exists(&lib_path).unwrap() {
            crate_context.entry_file_path = lib_path;
        } else {
            eprintln!("Can not find the entry file of the crate!");
            process::exit(5);
        }
        crate_context
    }

    pub fn parse_crate(&mut self) {
        let entry_code = read_to_string(&self.entry_file_path).unwrap();
        let entry_syntax = parse_file(&entry_code).unwrap();
        let mut mod_mod_info = ModModInfo::new();
        mod_mod_info.insert_mod_name(&self.crate_name);
        mod_mod_info.insert_parent_mod_tree(&String::new());
        mod_mod_info.insert_file_path(&self.entry_file_path);
        mod_mod_info
            .insert_parent_directory_path(&self.entry_file_path.parent().unwrap().to_path_buf());
        let mod_info = ModInfo::Mod(mod_mod_info);
        self.mod_context.insert_mod_info(&mod_info);
        self.mod_context.parse_from_items(&entry_syntax.items);
    }

    pub fn parse_all_context(&self) {
        let output_path = self.crate_path.join("rfocxt");
        fs::create_dir_all(&output_path).unwrap();
        self.mod_context
            .get_all_context(&output_path, &self.mod_context);
    }

    pub fn cout_in_one_file_for_test(&self) {
        let output_path = self.crate_path.join("context.txt");
        let mut file = File::create(&output_path).unwrap();
        file.write_all(format!("{:#?}", self).as_bytes()).unwrap();
    }

    pub fn cout_all_mod_trees_in_on_file_for_test(&self) {
        let mut mod_trees: Vec<String> = Vec::new();
        self.mod_context.get_all_mod_trees(&mut mod_trees);
        let output_path = self.crate_path.join("mod_trees.txt");
        let mut file = File::create(output_path).unwrap();
        file.write_all(format!("{:#?}", mod_trees).as_bytes())
            .unwrap();
    }

    pub fn cout_complete_function_name_in_on_file_for_test(&self) {
        let mut function_names: Vec<String> = Vec::new();
        self.mod_context
            .get_complete_function_names(&mut function_names);
        function_names.sort();
        let output_path = self.crate_path.join("functions.txt");
        let mut file = File::create(&output_path).unwrap();
        file.write_all(format!("{:#?}", function_names).as_bytes())
            .unwrap();
    }
}
