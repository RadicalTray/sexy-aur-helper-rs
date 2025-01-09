use std::io::Write;
use std::process::{Command, ExitStatus, Stdio};

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

    pub fn Syu_status() -> ExitStatus {
        Command::new("sudo")
            .arg("pacman")
            .arg("-Syu")
            .status()
            .expect("can't run pacman")
    }

    pub fn S_status(&self, pkg: &str) -> ExitStatus {
        let mut proc = Command::new("sudo")
            .arg("pacman")
            .arg("-S")
            .args(self.get_args())
            .arg(pkg)
            .stdin(Stdio::piped())
            .spawn()
            .expect("can't run pacman");

        proc.stdin
            .as_ref()
            .unwrap()
            .write("y\n".as_bytes())
            .unwrap();

        proc.wait().unwrap()
    }

    pub fn S_all_status(&self, pkgs: Vec<String>) -> ExitStatus {
        let mut proc = Command::new("sudo")
            .arg("pacman")
            .arg("-S")
            .args(self.get_args())
            .args(pkgs)
            .stdin(Stdio::piped())
            .spawn()
            .expect("can't run pacman");

        proc.stdin
            .as_ref()
            .unwrap()
            .write("y\n".as_bytes())
            .unwrap();

        proc.wait().unwrap()
    }

    pub fn U_status(&self, pkg: &str) -> ExitStatus {
        let mut proc = Command::new("sudo")
            .arg("pacman")
            .arg("-U")
            .args(self.get_args())
            .arg(pkg)
            .stdin(Stdio::piped())
            .spawn()
            .expect("can't run pacman");

        proc.stdin
            .as_ref()
            .unwrap()
            .write("y\n".as_bytes())
            .unwrap();

        proc.wait().unwrap()
    }

    pub fn U_all_status(&self, pkgs: Vec<String>) -> ExitStatus {
        let mut proc = Command::new("sudo")
            .arg("pacman")
            .arg("-U")
            .args(self.get_args())
            .args(pkgs)
            .stdin(Stdio::piped())
            .spawn()
            .expect("can't run pacman");

        proc.stdin
            .as_ref()
            .unwrap()
            .write("y\n".as_bytes())
            .unwrap();

        proc.wait().unwrap()
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
