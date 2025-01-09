use std::process::{Command, ExitStatus};

pub struct Git;

impl Git {
    pub fn diff() {}

    pub fn fetch() -> ExitStatus {
        Command::new("git").arg("fetch").status().unwrap()
    }

    pub fn reset_hard_origin() -> ExitStatus {
        Command::new("git")
            .args(["reset", "--hard", "origin"])
            .status()
            .unwrap()
    }

    pub fn clone(url: &str) -> ExitStatus {
        Command::new("git").args(["clone", url]).status().unwrap()
    }
}
