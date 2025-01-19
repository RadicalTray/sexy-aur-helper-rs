use std::process::{Command, ExitStatus};

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
        let mut flags = Self::get_args(install_info);
        if self.yes {
            flags.insert(0, "--noconfirm".to_string());
        }

        Command::new("sudo")
            .arg("pacman")
            .arg("-U")
            .args(flags)
            .arg("--")
            .args(&install_info.pkg_paths)
            .status()
            .expect("can't run pacman")
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
}
