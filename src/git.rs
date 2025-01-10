use std::process::{Command, ExitStatus, Output};
use std::path::PathBuf;

pub struct Git {
    cwd: PathBuf,
}

impl Git {
    pub fn cwd(cwd: PathBuf) -> Git {
        Git { cwd }
    }

    pub fn diff_fetch_color(&self) -> Output {
        Command::new("git")
            .args(["diff", "HEAD", "FETCH_HEAD", "--color=always"])
            .current_dir(&self.cwd)
            .output()
            .expect("can't run git")
    }

    pub fn fetch(&self) -> ExitStatus {
        Command::new("git")
            .arg("fetch")
            .current_dir(&self.cwd)
            .status()
            .unwrap()
    }

    pub fn reset_hard_origin(&self) -> ExitStatus {
        Command::new("git")
            .args(["reset", "--hard", "origin"])
            .current_dir(&self.cwd)
            .status()
            .unwrap()
    }

    pub fn clone(&self, url: &str) -> ExitStatus {
        Command::new("git")
            .args(["clone", url])
            .current_dir(&self.cwd)
            .status()
            .unwrap()
    }
}
