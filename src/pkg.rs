use crate::cmds::{fetch_pkg, fetch_pkgbase};
use crate::globals::*;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::{env, fs};

pub fn get_pkgbases(g: &Globals) -> Result<Vec<String>, String> {
    let pkgbase_path = g.cache_path.clone().join(FILENAME_PKGBASE);
    if !pkgbase_path.exists() {
        fetch_pkgbase(g)?;
    }
    Ok(read_file_lines_to_strings(pkgbase_path))
}

pub fn get_pkgs(g: &Globals) -> Result<Vec<String>, String> {
    let pkg_path = g.cache_path.clone().join(FILENAME_PKG);
    if !pkg_path.exists() {
        fetch_pkg(g)?;
    }

    Ok(read_file_lines_to_strings(pkg_path))
}

pub fn is_in_pkgbases(pkgbases: &Vec<String>, mut pkgs: Vec<String>) -> (Vec<String>, Vec<String>) {
    let err_pkgs = pkgs.extract_if(.., |pkg| !pkgbases.contains(pkg)).collect();
    (pkgs, err_pkgs)
}

pub fn upgrade(g: &Globals) {}

// TODO:
//  1 manage dependencies
//      - create dep tree
//  2 check if the pkg needs to be built
//  3 only install pkg that is new (in install())
pub fn sync(g: &Globals, pkgs: Vec<String>, quit_on_err: bool) {
    let clone_path = g.cache_path.clone().join("clone");
    if !clone_path.exists() {
        fs::create_dir(&clone_path).unwrap();
    }

    let (cloned_pkgs, err_pkgs) = clone(&clone_path, pkgs);
    if quit_on_err && err_pkgs.len() > 0 {
        eprintln!("Error happened while cloning:");
        for pkg in err_pkgs {
            eprintln!("\t{pkg}");
        }
        return;
    }

    let (built_pkg_paths, err_pkgs) = makepkg(&clone_path, cloned_pkgs);
    if quit_on_err && err_pkgs.len() > 0 {
        eprintln!("Error happened while building:");
        for pkg in err_pkgs {
            eprintln!("\t{pkg}");
        }
        return;
    }

    let status_code = install(&clone_path, built_pkg_paths);

    // TODO: print stats

    process::exit(status_code);
}

fn clone(clone_path: &PathBuf, pkgs: Vec<String>) -> (Vec<String>, Vec<String>) {
    env::set_current_dir(clone_path).unwrap();

    let mut cloned_pkgs = Vec::with_capacity(pkgs.len());
    let mut err_pkgs = Vec::new();

    for pkg in pkgs {
        let pkg_dir = clone_path.clone().join(&pkg);

        let status = if pkg_dir.exists() {
            Command::new("git").arg("fetch").status().unwrap()
        } else {
            let url = format!("{URL_AUR}/{pkg}.git");
            Command::new("git")
                .args(["clone", url.as_str()])
                .status()
                .unwrap()
        };

        if status.success() {
            cloned_pkgs.push(pkg);
        } else {
            err_pkgs.push(pkg);
        }
    }

    (cloned_pkgs, err_pkgs)
}

// fn fetch(clone_path: &PathBuf, pkgs: Vec<String>) -> (Vec<String>, Vec<String>) {
// }

fn makepkg(clone_path: &PathBuf, pkgs: Vec<String>) -> (Vec<String>, Vec<String>) {
    let mut built_pkg_paths = Vec::with_capacity(pkgs.len());
    let mut err_pkgs = Vec::new();

    for pkg in pkgs {
        env::set_current_dir(clone_path.clone().join(&pkg)).unwrap();

        Command::new("git")
            .args(["reset", "--hard", "origin"])
            .spawn()
            .unwrap();

        let status = Command::new("makepkg").status().unwrap();
        if status.code().unwrap() == 13 {
            // already built
            let output = Command::new("makepkg")
                .arg("--packagelist")
                .output()
                .unwrap();
            built_pkg_paths.extend(read_lines_to_strings(
                String::from_utf8(output.stdout).expect("Output not UTF-8"),
            ));
        } else if status.code().unwrap() == 0 {
            // newly built
            let output = Command::new("makepkg")
                .arg("--packagelist")
                .output()
                .unwrap();
            built_pkg_paths.extend(read_lines_to_strings(
                String::from_utf8(output.stdout).expect("Output not UTF-8"),
            ));
        } else {
            // failed
            err_pkgs.push(pkg);
        }
    }

    (built_pkg_paths, err_pkgs)
}

fn install(_clone_path: &PathBuf, pkg_paths: Vec<String>) -> i32 {
    Command::new("sudo")
        .arg("pacman")
        .arg("-U")
        .args(pkg_paths)
        .status()
        .unwrap()
        .code()
        .unwrap()
}

pub fn read_file_lines_to_strings<P: AsRef<Path>>(filepath: P) -> Vec<String> {
    read_lines_to_strings(fs::read_to_string(filepath).unwrap())
}

fn read_lines_to_strings(s: String) -> Vec<String> {
    s.lines().map(String::from).collect()
}
