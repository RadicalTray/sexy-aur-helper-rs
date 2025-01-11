use std::path::PathBuf;
use std::process::{Command, ExitStatus, Output};

pub struct Bash {
    cwd: PathBuf,
}

impl Bash {
    pub fn cwd(cwd: PathBuf) -> Self {
        Bash { cwd }
    }

    pub fn output(&self, cmd: &str) -> Output {
        Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .current_dir(&self.cwd)
            .output()
            .unwrap()
    }

    #[allow(dead_code)]
    pub fn status(&self, cmd: &str) -> ExitStatus {
        Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .current_dir(&self.cwd)
            .status()
            .unwrap()
    }
}
