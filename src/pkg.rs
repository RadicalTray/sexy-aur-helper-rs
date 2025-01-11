// TODO: refactor

use crate::alpm::get_local_aur_pkgs;
use crate::cmds::{fetch_pkgbase, fetch_pkglist};
use crate::fetch::*;
use crate::git::Git;
use crate::globals::*;
use crate::makepkg::Makepkg;
use crate::pacman::Pacman;
use crate::utils::*;
use alpm::{Alpm, Package, Version};
use alpm_utils::depends::satisfies_nover;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process;
use std::{env, fs};

type PV = (String, Option<Version>);

fn prepare(g: &Globals) {
    let clone_path = g.cache_path.clone().join("clone");
    if !clone_path.exists() {
        fs::create_dir(&clone_path).unwrap();
    }
    env::set_current_dir(clone_path).unwrap();
}

pub fn upgrade(g: &Globals) {
    prepare(g);

    let status = Pacman::Syu_status();
    if !status.success() {
        process::exit(status.code().unwrap());
    }

    let handle = Alpm::new("/", "/var/lib/pacman").unwrap();
    let (aur_pkgs, err_pkgs) = get_local_aur_pkgs(&handle, &g);
    for pkg in err_pkgs {
        println!("{} not found in aur!", pkg.name());
    }
    println!();

    let clone_path = g.cache_path.clone().join("clone");
    let (old, new, err) = fetch_pkgs(
        &clone_path,
        aur_pkgs.iter().map(|x| x.name().to_string()).collect(),
    );

    println!("{} packages to be built:", aur_pkgs.len());
    for pkg in &aur_pkgs {
        println!("\t{}", pkg.name());
    }
    prompt_accept();

    // TODO: getver
    //  - run `makepkg --nobuild`
    //  - run `source PKGBUILD; echo $pkgver`
    // then build with `--noextract` (because --nobuild already fetched things)

    // if new_ver(pkg.ver(), new_ver) {
    //
    // }

    // prob don't need multithread?
    let build_stack = setup_build_stack(&aur_pkgs);
    let mut set: HashSet<&str> = HashSet::new();

    let mut err_pkgs: Vec<String> = Vec::new();
    for pkg in build_stack.iter().rev() {
        if set.contains(pkg.name()) {
            continue;
        }
        set.insert(pkg.name());

        let cwd = clone_path.clone().join(pkg.name());
        env::set_current_dir(cwd).unwrap();
        let built_pkg_paths = match build(pkg.name()) {
            Ok(v) => v,
            Err(pkg) => {
                err_pkgs.push(pkg);
                continue;
            }
        };

        install(built_pkg_paths);
    }

    if err_pkgs.len() > 0 {
        eprintln!("Error happened while building:");
        for pkg in err_pkgs {
            eprintln!("\t{pkg}");
        }
    }

    if set.len() != aur_pkgs.len() {
        println!("pkg not in build stack:");
        for pkg in aur_pkgs {
            if !set.contains(pkg.name()) {
                println!("\t{}", pkg.name());
            }
            // set containing pkgs not in aur_pkgs should be impossible
        }
        process::exit(1);
    }
}

fn setup_build_stack<'a>(aur_pkgs: &Vec<&'a Package>) -> Vec<&'a Package> {
    let mut build_stack: Vec<_> = Vec::with_capacity(aur_pkgs.len());

    // BUG: will not include a package that depends on each other
    // in reverse so the build stack has first packages last
    for pkg in aur_pkgs.iter().rev() {
        let is_aur_pkg_dep = {
            let mut tmp = false;
            for pkg_name in pkg.required_by() {
                if aur_pkgs
                    .iter()
                    .map(|x| x.name())
                    .collect::<Vec<&str>>()
                    .contains(&pkg_name.as_str())
                {
                    tmp = true;
                    break;
                }
            }

            tmp
        };

        if !is_aur_pkg_dep {
            push_to_build_stack(&aur_pkgs, &mut build_stack, pkg);
        }
    }

    build_stack
}

// algorithm to build deps first
fn push_to_build_stack<'a>(
    all_pkgs: &Vec<&'a Package>,
    stack: &mut Vec<&'a Package>,
    pkg: &'a Package,
) {
    stack.push(pkg);
    let mut deps = pkg.depends().to_list_mut();
    deps.extend(pkg.makedepends().iter());
    deps.extend(pkg.checkdepends().iter()); // NOTE: likely not needed
    for dep in deps {
        for pkg in all_pkgs {
            if satisfies_nover(dep, pkg.name(), pkg.provides().into_iter()) {
                push_to_build_stack(all_pkgs, stack, pkg);
                break;
            }
        }
    }
}

// TODO:
//  0 notify changes in PKGBUILD
//  1 manage dependencies
//      - create dep tree
//  2 check if the pkg needs to be built
//  3 only install pkg that is new (in install())
pub fn sync(g: &Globals, pkgs: Vec<String>, quit_on_err: bool) {
    prepare(g);

    let clone_path = g.cache_path.clone().join("clone");
    let (mut old_pkgs, new_pkgs, err_pkgs) = fetch_pkgs(&clone_path, pkgs);
    old_pkgs.extend(new_pkgs);
    let fetched_pkgs = old_pkgs;

    if quit_on_err && err_pkgs.len() > 0 {
        eprintln!("Error happened while cloning:");
        for pkg in err_pkgs {
            eprintln!("\t{pkg}");
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

    let status_code = install(built_pkg_paths);
    if err_pkgs.len() > 0 {
        eprintln!("Error happened while building:");
        for pkg in err_pkgs {
            eprintln!("\t{pkg}");
        }
    }

    process::exit(status_code);
}

fn build_all(clone_path: &PathBuf, pkgs: Vec<String>) -> (Vec<String>, Vec<String>) {
    let mut built_pkg_paths = Vec::with_capacity(pkgs.len());
    let mut err_pkgs = Vec::new();

    for pkg in pkgs {
        let cwd = clone_path.clone().join(&pkg);

        Git::cwd(cwd.clone()).reset_hard_origin();

        env::set_current_dir(cwd).unwrap();
        match build(&pkg) {
            Ok(v) => built_pkg_paths.extend(v),
            Err(pkg) => {
                err_pkgs.push(pkg);
                continue;
            }
        };
    }

    (built_pkg_paths, err_pkgs)
}

fn build(pkg: &str) -> Result<Vec<String>, String> {
    let status = Makepkg::new().status();
    match status.code().unwrap() {
        13 | 0 => {
            let output = Makepkg {
                packagelist: true,
                ..Default::default()
            }
            .output();

            Ok(read_lines_to_strings(
                String::from_utf8(output.stdout).expect("Output not UTF-8"),
            ))
        }
        _ => Err(pkg.to_string()),
    }
}

fn install(pkg_paths: Vec<String>) -> i32 {
    Pacman::new().U_all_status(pkg_paths).code().unwrap()
}

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
        fetch_pkglist(g)?;
    }

    Ok(read_file_lines_to_strings(pkg_path))
}

pub fn is_in_pkgbases(pkgbases: &Vec<String>, mut pkgs: Vec<String>) -> (Vec<String>, Vec<String>) {
    let err_pkgs = pkgs.extract_if(.., |pkg| !pkgbases.contains(pkg)).collect();
    (pkgs, err_pkgs)
}
