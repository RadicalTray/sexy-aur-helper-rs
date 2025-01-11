use std::io;
use std::io::Write;
use std::process::{Child, Command, ExitStatus, Stdio};

pub struct Pacman {
    pub yes: bool,
    pub asexplicit: bool,
    pub asdeps: bool,
    pub needed: bool,
}

impl Default for Pacman {
    fn default() -> Self {
        Pacman {
            yes: false,
            asexplicit: false,
            asdeps: false,
            needed: false,
        }
    }
}

#[allow(non_snake_case)]
impl Pacman {
    pub fn new() -> Self {
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

    #[allow(dead_code)]
    pub fn S_status(&self, pkg: &str) -> ExitStatus {
        let proc = Command::new("sudo")
            .arg("pacman")
            .arg("-S")
            .args(self.get_args())
            .arg(pkg)
            .stdin(Stdio::piped())
            .spawn()
            .expect("can't run pacman");

        self.enter_and_wait(proc)
    }

    #[allow(dead_code)]
    pub fn S_all_status(&self, pkgs: Vec<String>) -> ExitStatus {
        let proc = Command::new("sudo")
            .arg("pacman")
            .arg("-S")
            .args(self.get_args())
            .args(pkgs)
            .stdin(Stdio::piped())
            .spawn()
            .expect("can't run pacman");

        self.enter_and_wait(proc)
    }

    #[allow(dead_code)]
    pub fn U_status(&self, pkg: &str) -> ExitStatus {
        let proc = Command::new("sudo")
            .arg("pacman")
            .arg("-U")
            .args(self.get_args())
            .arg(pkg)
            .stdin(Stdio::piped())
            .spawn()
            .expect("can't run pacman");

        self.enter_and_wait(proc)
    }

    pub fn U_all_status(&self, pkgs: Vec<String>) -> ExitStatus {
        let proc = Command::new("sudo")
            .arg("pacman")
            .arg("-U")
            .args(self.get_args())
            .args(pkgs)
            .stdin(Stdio::piped())
            .spawn()
            .expect("can't run pacman");

        self.enter_and_wait(proc)
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

    fn enter_and_wait(&self, mut proc: Child) -> ExitStatus {
        if self.yes {
            proc.stdin
                .as_ref()
                .unwrap()
                .write("y\n".as_bytes())
                .unwrap();

            println!();
            io::stdout().flush().unwrap();
        }

        proc.wait().unwrap()
    }
}
