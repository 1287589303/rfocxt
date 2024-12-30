use std::{fs, path::PathBuf, process};

use clap::Parser;
use collect_context::crate_context::CrateContext;

mod collect_context;

#[derive(Parser)]
#[command(name = "rust focxt")]
#[command(author = "AbeZbm")]
#[command(version = "1.0")]
#[command(about="A rust program to get focal context for a crate.",long_about=None)]
struct Cli {
    ///Sets crate path
    #[arg(short = 'c', long = "crate", required = true)]
    crate_path: String,
}

fn main() {
    let cli = Cli::parse();
    let input_crate_path = PathBuf::from(cli.crate_path);
    let crate_path = fs::canonicalize(&input_crate_path).unwrap_or_else(|_err| {
        eprintln!("The crate path {:?} doesn't exisit!", &input_crate_path);
        process::exit(1)
    });
    let mut crate_context = CrateContext::new(crate_path);
    crate_context.parse_crate();
    // crate_context.parse_all_context();
    crate_context.cout_in_one_file_for_test();
    crate_context.cout_all_mod_trees_in_on_file_for_test();
    crate_context.cout_complete_function_name_in_on_file_for_test();
}
