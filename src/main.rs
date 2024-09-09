use clap::Parser;
use collect_context::file_context::RsFile;
use std::{
    fs::{self, create_dir_all, read_to_string, File},
    io::Write,
    path::{Path, PathBuf},
};
use syn::parse_file;
use utils::find_rs_files;

mod collect_context;
mod utils;

#[derive(Parser)]
#[command(name = "rust focxt")]
#[command(author = "AbeZbm")]
#[command(version = "1.0")]
#[command(about = "A rust program to get focal context.", long_about = None)]
struct Cli {
    ///Sets project path
    #[arg(short = 'p', long = "project")]
    project: Option<String>,
}
fn main() {
    let cli = Cli::parse();
    let project_path = fs::canonicalize(PathBuf::from(cli.project.unwrap())).unwrap();
    println!("project path {}", project_path.to_str().unwrap());
    let rs_files: Vec<PathBuf> = find_rs_files(&project_path);
    println!("{:#?}", rs_files);
    // let mut file_contexts: Vec<FileContext> = Vec::new();
    let mut file_contexts: Vec<RsFile> = Vec::new();
    for rs_file in rs_files {
        let code = read_to_string(&rs_file).unwrap();
        let syntax = parse_file(&code).unwrap();
        // println!("{:#?}", syntax);
        let mut syn_file = RsFile::from_syn_file(
            rs_file.file_name().unwrap().to_string_lossy().to_string(),
            &syntax,
        );
        let output_path = project_path.join("ast").join(format!(
            "{}",
            rs_file.file_name().unwrap().to_string_lossy()
        ));
        println!("{:#?}", output_path);
        if let Some(parent) = Path::new(&output_path).parent() {
            // 创建所有必要的父目录
            create_dir_all(parent);
        }
        let mut file = File::create(output_path).unwrap();
        // file.write_all(format!("{:#?}", syntax).as_bytes());
        file.write_all(syn_file.to_string().as_bytes());
        // file.write_all(format!("{:#?}", syn_file).as_bytes());
        // file_contexts.push(collect_file_context(
        //     rs_file.file_name().unwrap().to_string_lossy().to_string(),
        //     &syntax,
        // ));
        file_contexts.push(syn_file);
    }
    println!("{:#?}", file_contexts);
    let mut all_rs_file = RsFile::new();
    for file_context in file_contexts {
        all_rs_file.uses.extend(file_context.uses);
        all_rs_file.mods.extend(file_context.mods);
        all_rs_file.structs.extend(file_context.structs);
        all_rs_file.functions.extend(file_context.functions);
        all_rs_file.traits.extend(file_context.traits);
    }
    let output_path = project_path.join("ast").join("all.rs");
    let mut file = File::create(output_path).unwrap();
    file.write_all(all_rs_file.to_string().as_bytes());
}
