use std::path::Path;
use std::process::{Command, ExitStatus, Output};

pub struct Makepkg {
    nobuild: bool,
    noextract: bool,
    needed: bool,
    force: bool,
    packagelist: bool,
    cleanbuild: bool,
    clean: bool,
}

impl Default for Makepkg {
    fn default() -> Makepkg {
        Makepkg {
            nobuild: false,
            noextract: false,
            needed: false,
            force: false,
            packagelist: false,
            cleanbuild: false,
            clean: false,
        }
    }
}

impl Makepkg {
    pub fn new() -> Makepkg {
        Makepkg {
            ..Default::default()
        }
    }

    pub fn status<P: AsRef<Path>>(&self, cwd: P) -> ExitStatus {
        Command::new("makepkg")
            .args(self.get_args())
            .current_dir(cwd)
            .status()
            .expect("can't run makepkg")
    }

    pub fn output<P: AsRef<Path>>(&self, cwd: P) -> Output {
        Command::new("makepkg")
            .args(self.get_args())
            .current_dir(cwd)
            .output()
            .expect("can't run makepkg")
    }

    // NOTE: Is there a way to write this better in rust?
    //
    // PERF: this practically makes 2 vec if compiler doesn't see it
    fn get_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        let flags = self;
        if flags.needed {
            args.push("--needed");
        }
        if flags.force {
            args.push("--force");
        }
        if flags.nobuild {
            args.push("--nobuild");
        }
        if flags.noextract {
            args.push("--noextract");
        }
        if flags.packagelist {
            args.push("--packagelist");
        }
        if flags.cleanbuild {
            args.push("--cleanbuild");
        }
        if flags.clean {
            args.push("--clean");
        }

        args.iter().map(|x| x.to_string()).collect()
    }
}
