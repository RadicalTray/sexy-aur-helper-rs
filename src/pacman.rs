use std::path::Path;
use std::process::{Command, ExitStatus};

pub struct Pacman {
    asexplicit: bool,
    asdeps: bool,
    needed: bool,
}

impl Default for Pacman {
    fn default() -> Pacman {
        Pacman {
            asexplicit: false,
            asdeps: false,
            needed: false,
        }
    }
}

impl Pacman {
    pub fn new() -> Pacman {
        Pacman {
            ..Default::default()
        }
    }

    pub fn S_status<P: AsRef<Path>>(&self, cwd: P, pkg: &str) -> ExitStatus {
        Command::new("sudo")
            .arg("pacman")
            .arg("-S")
            .args(self.get_args())
            .arg(pkg)
            .current_dir(cwd)
            .status()
            .expect("can't run pacman")
    }

    pub fn S_all_status<P: AsRef<Path>>(&self, cwd: P, pkgs: Vec<String>) -> ExitStatus {
        Command::new("sudo")
            .arg("pacman")
            .arg("-S")
            .args(self.get_args())
            .args(pkgs)
            .current_dir(cwd)
            .status()
            .expect("can't run pacman")
    }

    pub fn U_status<P: AsRef<Path>>(&self, cwd: P, pkg: &str) -> ExitStatus {
        Command::new("sudo")
            .arg("pacman")
            .arg("-U")
            .args(self.get_args())
            .arg(pkg)
            .current_dir(cwd)
            .status()
            .expect("can't run pacman")
    }

    pub fn U_all_status<P: AsRef<Path>>(&self, cwd: P, pkgs: Vec<String>) -> ExitStatus {
        Command::new("sudo")
            .arg("pacman")
            .arg("-U")
            .args(self.get_args())
            .args(pkgs)
            .current_dir(cwd)
            .status()
            .expect("can't run pacman")
    }

    // NOTE: Is there a way to write this better in rust?
    //
    // PERF: this practically makes 2 vec if compiler doesn't see it
    fn get_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        let flags = self;
        if flags.asdeps {
            args.push("--asdeps");
        }
        if flags.asexplicit {
            args.push("--asexplicit");
        }
        if flags.needed {
            args.push("--needed");
        }

        args.iter().map(|x| x.to_string()).collect()
    }
}
