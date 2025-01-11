use std::process::{Command, ExitStatus, Output};

pub struct Makepkg {
    pub nobuild: bool,
    pub noextract: bool,
    pub force: bool,
    pub packagelist: bool,
    pub cleanbuild: bool,
    pub clean: bool,
}

impl Default for Makepkg {
    fn default() -> Self {
        Makepkg {
            nobuild: false,
            noextract: false,
            force: false,
            packagelist: false,
            cleanbuild: false,
            clean: false,
        }
    }
}

impl Makepkg {
    pub fn new() -> Self {
        Makepkg {
            ..Default::default()
        }
    }

    pub fn status(&self) -> ExitStatus {
        Command::new("makepkg")
            .args(self.get_args())
            .status()
            .expect("can't run makepkg")
    }

    pub fn output(&self) -> Output {
        Command::new("makepkg")
            .args(self.get_args())
            .output()
            .expect("can't run makepkg")
    }

    // NOTE: Is there a way to write this better in rust?
    //
    // PERF: this practically makes 2 vec if compiler doesn't see it
    fn get_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        let flags = self;
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
