use std::{env, path::PathBuf, process::Command};

fn cargo_install() {
    let current_dir = env::current_dir().unwrap();
    let project_dir = current_dir.canonicalize().unwrap().join("call_chain");
    // println!("{}", project_dir.to_string_lossy());
    let install_output = Command::new("cargo")
        .args(["install", "--path", "."])
        .current_dir(project_dir)
        .output()
        .expect("Failed to install call_chain");

    if !install_output.status.success() {
        eprintln!("Install failed!");
        std::process::exit(10);
    }
}

pub fn cargo_clean(work_path: &PathBuf) {
    // println!("Cleaning the project...");
    let clean_output = Command::new("cargo")
        .arg("clean")
        .current_dir(work_path)
        .output()
        .expect("Failed to clean the project");

    if !clean_output.status.success() {
        eprintln!("Clean failed.");
        return;
    }
}

fn call_chain(crate_path: &PathBuf) {
    let call_chain_output = Command::new("cargo")
        .arg("call-chain")
        .current_dir(crate_path)
        .output()
        .expect("Failed to run call_chain");

    if !call_chain_output.status.success() {
        eprintln!("Call_chain failed!");
        std::process::exit(11);
    }
}

pub fn run_call_chain(crate_path: &PathBuf) {
    cargo_install();
    cargo_clean(crate_path);
    call_chain(crate_path);
}
