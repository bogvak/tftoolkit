use clap::builder::OsStr;
use clap::{Parser, Subcommand};
use std::fs::{self, File};
use std::io::{Read, BufReader, BufRead};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Clear all current terraform state and installation
    Clear,
    Debug,
}

const TO_DELETE: &'static [&'static str] = &[
    ".terraform",
    ".terraform.lock.hcl",
    ".terragrunt-cache",
    "tfplan",
    "terraform.tfstate",
    "terraform.tfstate.backup",
];

const TO_IGNORE: &'static [&'static str] = &[
    ".git"    
];

const EXTENSIONS_TO_CHECK: &'static [&'static str] = &[
    "tf"
];

fn read_first_line(path: &PathBuf) -> Result<String, std::io::Error> {
    // Open the file in read-only mode and create a buffered reader
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the first line from the buffered reader
    let mut first_line = String::new();
    if let Some(Ok(line)) = reader.lines().next() {
        first_line = line;
    }

    Ok(first_line)
}

fn delete_file_or_dir(path: &str) -> std::io::Result<()> {
    match fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
        }
        Err(_) => println!("Error: path not found"),
    };
    Ok(())
}

fn check_if_should_be_deleted(path: &PathBuf) -> bool {    
    let str_push = path.file_name().unwrap_or_default();
    // dbg!(str_push);
    
    // **** Checking if file is part of list of files to be deletes
    if TO_DELETE.iter().any(|&s| s.eq(str_push)) {return true}
    
    // **** Check if file is generated by Terragrunt
    if if_should_be_checked_for_terragrunt_generated(path) {
        match read_first_line(&path) {
            Ok(result) => {
                if result.contains("# Generated by Terragrunt") {
                    return true;
                }
                if result.contains("# Managed via Terragrunt") {
                    return true;
                }
            },
            Err(e) => {return false},
        }
    };
    false
}

fn check_if_should_be_ignored(path: &PathBuf) -> bool {
    // dbg!(path);
    let str_push = path.file_name().unwrap_or_default();
    // dbg!(str_push);
    TO_IGNORE.iter().any(|&s| s.eq(str_push))
}

fn if_should_be_checked_for_terragrunt_generated(path: &PathBuf) -> bool {
    if let Some(ext) = path.extension() {
        // If the path is a file, return the file extension
        return EXTENSIONS_TO_CHECK.iter().any(|&s| s.eq(ext))
    }
    false
}

fn recursive_delete(path: &PathBuf, file_list: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                if check_if_should_be_ignored(&entry_path) {
                    continue;
                };
                if check_if_should_be_deleted(&entry_path) {
                    let path_to_clear = entry_path.to_str().unwrap().to_string();
                    println!("Clearing {path_to_clear}");
                    delete_file_or_dir(&path_to_clear);
                    continue;
                };
                if entry_path.is_dir() {
                    recursive_delete(&entry_path, file_list);
                }
            }
        }
    }
}

fn recursive_clear() {
    let mut file_list: Vec<String> = Vec::new();
    recursive_delete(&PathBuf::from("."), &mut file_list);
}
fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Clear) => {
            println!("Clearing Terraform inits and local states...");
            recursive_clear();
        }
        Some(Commands::Debug) => {
            println!("Debug functions");
            recursive_clear();
        }
        None => {}
    }
}
