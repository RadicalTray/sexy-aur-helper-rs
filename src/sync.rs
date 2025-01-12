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

    let pkgbases = get_pkgbases(&g)?;

    let (pkgs, pkgs_not_found) = is_in_pkgbases(&pkgbases, args);
    if pkgs_not_found.len() > 0 {
        println!("package not found in package base list:");
        for pkg in pkgs_not_found {
            println!("\t{pkg}");
        }
        return Ok(());
    }

    sync(&g, pkgs, true);

    Ok(())
}

struct SyncConfig {
    needed: bool,
    asdeps: bool,
    asexplicit: bool,
    force: bool,
}

fn parse_config() {}

fn sync(g: &Globals, pkgs: Vec<String>, quit_on_err: bool) {
    prepare_clone(g);

    let clone_path = g.cache_path.clone().join("clone");
    let (mut old_pkgs, new_pkgs, err_pkgs) =
        fetch_pkgs(&clone_path, pkgs.into_iter().map(|x| (x, None)).collect());
    old_pkgs.extend(new_pkgs);
    let fetched_pkgs = old_pkgs.iter().map(|x| x.0.clone()).collect();

    if quit_on_err && err_pkgs.len() > 0 {
        eprintln!("Error happened while cloning:");
        for pkg in err_pkgs {
            eprintln!("\t{}", pkg.0);
        }
        return;
    }

    let (built_pkg_paths, err_pkgs) = build_all(&clone_path, fetched_pkgs);
    if quit_on_err && err_pkgs.len() > 0 {
        eprintln!("Error happened while building:");
        for pkg in err_pkgs {
            eprintln!("\t{pkg}");
        }
        return;
    }

    // TODO: print stats

    let status_code = Pacman::new().U_all_status(built_pkg_paths).code().unwrap();
    if err_pkgs.len() > 0 {
        eprintln!("Error happened while building:");
        for pkg in err_pkgs {
            eprintln!("\t{pkg}");
        }
    }

    process::exit(status_code);
}
