use std::io;
use std::io::Write;
use std::process::{Child, Command, ExitStatus, Stdio};

pub struct InstallInfo {
    pub pkg_paths: Vec<String>,
    pub needed: bool,
    pub asdeps: bool,
    pub asexplicit: bool,
}

pub struct Pacman {
    pub yes: bool,
}

#[allow(non_snake_case)]
impl Pacman {
    pub fn Syu_status() -> ExitStatus {
        Command::new("sudo")
            .arg("pacman")
            .arg("-Syu")
            .status()
            .expect("can't run pacman")
    }

    pub fn U_all_status(&self, install_info: &InstallInfo) -> ExitStatus {
        let proc = Command::new("sudo")
            .arg("pacman")
            .arg("-U")
            .args(Self::get_args(install_info))
            .args(&install_info.pkg_paths)
            .stdin(if self.yes {
                Stdio::piped()
            } else {
                Stdio::inherit()
            })
            .spawn()
            .expect("can't run pacman");

        self.enter_and_wait(proc)
    }

    // NOTE: Is there a way to write this better in rust?
    //
    // PERF: this practically makes 2 vec if compiler doesn't see it
    fn get_args(install_info: &InstallInfo) -> Vec<String> {
        let mut args = Vec::new();
        let flags = install_info;
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

    fn enter_and_wait(&self, mut proc: Child) -> ExitStatus {
        if self.yes {
            proc.stdin.as_ref().unwrap().write("\n".as_bytes()).unwrap();

            println!();
            io::stdout().flush().unwrap();
        }

        proc.wait().unwrap()
    }
}
