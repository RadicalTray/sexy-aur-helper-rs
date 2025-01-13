use crate::globals::Globals;
use std::fs;

pub const STR: &str = "clear-cache";

pub fn run(g: Globals, args: Vec<String>) -> Result<(), String> {
    if args.len() != 0 {
        return Err("unexpected arguments".to_string());
    }

    println!("Clearing built packages...");
    let clone_path = g.cache_path.join("clone");
    if clone_path.exists() {
        for clone_entry in fs::read_dir(clone_path).unwrap() {
            let clone_entry = clone_entry.unwrap();

            if clone_entry.file_type().unwrap().is_dir() {
                // assuming it's an aur repo
                for repo_entry in fs::read_dir(clone_entry.path()).unwrap() {
                    let repo_entry = repo_entry.unwrap();
                    if repo_entry.file_type().unwrap().is_file()
                        && repo_entry
                            .file_name()
                            .into_string()
                            .unwrap()
                            .contains(".pkg.tar.zst")
                    {
                        println!("{}", repo_entry.path().display());
                        fs::remove_file(repo_entry.path()).unwrap();
                    }
                }
            }
        }
    }

    Ok(())
}
