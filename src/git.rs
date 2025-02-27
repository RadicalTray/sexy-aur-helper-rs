use std::path::PathBuf;
use std::process::{Command, ExitStatus, Output, Stdio};

pub struct Git {
    cwd: PathBuf,
}

impl Git {
    pub fn cwd(cwd: PathBuf) -> Self {
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

    pub fn reset_hard_origin_mute(&self) -> ExitStatus {
        // keep stderr, just in case
        Command::new("git")
            .args(["reset", "--hard", "origin"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
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
