use crate::build::*;
use crate::fetch::*;
use crate::fetch::{get_pkgbases, is_in_pkgbases};
use crate::globals::Globals;
use crate::pacman::Pacman;
use crate::utils::*;
use std::process;

pub const STR: &str = "sync";

pub fn run(g: Globals, args: Vec<String>) -> Result<(), String> {
    if args.len() == 0 {
        return Err(String::from("no package specified"));
    }

    let pkg_infos = match parse_args(args) {
        Ok(v) => v,
        Err(s) => return Err(s),
    };

    let pkgbases = get_pkgbases(&g)?;

    let (pkg_infos, pkgs_not_found) = is_in_pkgbases(&pkgbases, pkg_infos);
    if pkgs_not_found.len() > 0 {
        println!("package not found in package base list:");
        for pkg in pkgs_not_found {
            println!("\t{}", pkg.name);
        }
        return Ok(());
    }

    sync(&g, pkg_infos);

    Ok(())
}

struct State {
    needed: bool,
    asdeps: bool,
    asexplicit: bool,
    force: bool,
}

impl State {
    fn new() -> Self {
        State {
            needed: false,
            asdeps: false,
            asexplicit: false,
            force: false,
        }
    }
}

fn parse_args(args: Vec<String>) -> Result<Vec<PkgInfo>, String> {
    let mut state = State::new();
    let mut err_flags = Vec::new();
    let mut pkgs = Vec::with_capacity(args.len());
    for arg in args {
        match arg.as_str() {
            "--needed" => {
                state.needed = true;
                continue;
            }
            "--asdeps" => {
                if state.asexplicit {
                    return Err("`--asdeps` cannot be used with `--asexplicit`".to_string());
                }

                state.asdeps = true;
                continue;
            }
            "--asexplicit" => {
                if state.asdeps {
                    return Err("`--asexplicit` cannot be used with `--asdeps`".to_string());
                }

                state.asexplicit = true;
                continue;
            }
            "-f" | "--force" => {
                state.force = true;
                continue;
            }
            _ => {
                if arg[0..1] == *"--" {
                    err_flags.push(arg[2..].to_string());
                } else if arg.chars().nth(0) == Some('-') {
                    err_flags.push(arg[1..].to_string());
                } else {
                    pkgs.push(arg);
                }
            }
        }
    }
    if err_flags.len() > 0 {
        let mut s = String::from("unexpected flags:");
        for flag in err_flags {
            s.push_str(format!(" `{}`", flag).as_str());
        }
        return Err(s);
    }

    let pkg_infos = pkgs.into_iter().map(|x| PkgInfo {
        name: x,
        needed: state.needed,
        asdeps: state.asdeps,
        asexplicit: state.asexplicit,
        force: state.force,
    }).collect();

    Ok(pkg_infos)
}

fn sync(g: &Globals, pkgs: Vec<PkgInfo>) {
    prepare_clone(g);

    let clone_path = g.cache_path.clone().join("clone");
    let (mut old_pkgs, new_pkgs, err_pkgs) =
        fetch_pkgs(&clone_path, pkgs.into_iter().map(|x| (x, None)).collect());
    if err_pkgs.len() > 0 {
        eprintln!("Error happened while cloning:");
        for pkg in err_pkgs {
            eprintln!("\t{}", pkg.0.name);
        }
        return;
    }

    old_pkgs.extend(new_pkgs);
    let fetched_pkgs = old_pkgs.iter().map(|x| x.0.clone()).collect();
    let (install_infos, err_pkgs) = build_all(&clone_path, fetched_pkgs);
    let mut status_code = 0;
    for install_info in install_infos {
        let c = Pacman { yes: true }
            .U_all_status(install_info)
            .code()
            .unwrap();
        if c != 0 {
            status_code = c;
        }
    }

    if err_pkgs.len() > 0 {
        eprintln!("Error happened while building:");
        for pkg in err_pkgs {
            eprintln!("\t{pkg}");
        }
    }

    process::exit(status_code);
}
