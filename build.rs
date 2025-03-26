use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=.git/index");
    println!("cargo:rerun-if-changed=.git/HEAD");

    println!("cargo::rustc-env=GIT_FULLHASH=UNKNOWN!");
    match Command::new("git").args(&["rev-parse", "HEAD"]).output() {
        Ok(output) => match String::from_utf8(output.stdout) {
            Ok(hash) => {
                if !hash.is_empty() && !hash.starts_with("fatal") {
                    println!("cargo::rustc-env=GIT_FULLHASH={}", hash.trim());
                }
            }
            Err(_) => (),
        },
        Err(_) => (),
    };
    println!("cargo::rustc-env=GIT_SHORTHASH=UNKNOWN!");
    match Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
    {
        Ok(output) => match String::from_utf8(output.stdout) {
            Ok(hash) => {
                if !hash.is_empty() && !hash.starts_with("fatal") {
                    println!("cargo::rustc-env=GIT_SHORTHASH={}", hash.trim());
                }
            }
            Err(_) => (),
        },
        Err(_) => (),
    }
    println!("cargo::rustc-env=GIT_PORCELAIN=UNKNOWN!");
    match Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
    {
        Ok(output) => match String::from_utf8(output.stdout) {
            Ok(status) => {
                if status.is_empty() {
                    println!("cargo::rustc-env=GIT_PORCELAIN=clean");
                } else {
                    println!("cargo::rustc-env=GIT_PORCELAIN=dirty")
                }
            }
            Err(_) => (),
        },
        Err(_) => (),
    }
}
