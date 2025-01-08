use std::path::Path;
use std::process::{Command, ExitStatus};

pub struct MakepkgOpts {
    nobuild: bool,
    noextract: bool,
    needed: bool,
    force: bool,
    packagelist: bool,
    cleanbuild: bool,
    clean: bool,
}

impl Default for MakepkgOpts {
    fn default() -> MakepkgOpts {
        MakepkgOpts {
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

/// will set cwd to path
pub fn makepkg<P: AsRef<Path>>(path: P, flags: MakepkgOpts) -> ExitStatus {
    Command::new("makepkg")
        .args(get_args(flags))
        .current_dir(path)
        .status()
        .expect("can't run makepkg")
}

// NOTE: Is there a way to write this better in rust?
fn get_args(flags: MakepkgOpts) -> Vec<String> {
    let mut args = Vec::new();
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
