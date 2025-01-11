// TODO: refactor

use crate::alpm::get_local_aur_pkgs;
use crate::build::*;
use crate::fetch::*;
use crate::globals::*;
use crate::pacman::Pacman;
use crate::utils::*;
use crate::ver::get_pkgs_to_upgrade;
use alpm::{Alpm, Package, Version};
use alpm_utils::depends::satisfies_nover;
use std::collections::HashSet;
use std::process;
use std::{env, fs};

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
    let (mut old_pkgs, new_pkgs, _err_pkgs) = fetch_pkgs(
        &clone_path,
        aur_pkgs
            .iter()
            .map(|x| {
                (
                    x.name().to_string(),
                    Some(Version::new(x.version().as_str())),
                )
            })
            .collect(),
    );
    old_pkgs.extend(new_pkgs);
    let fetched_pkgs = old_pkgs;

    let pkgs_to_build = get_pkgs_to_upgrade(&clone_path, fetched_pkgs);

    println!("{} packages to be built:", aur_pkgs.len());
    for pkg in &aur_pkgs {
        println!("\t{}", pkg.name());
    }
    prompt_accept();

    // TODO: getver
    //  - run `makepkg --nobuild`
    //  - run `source PKGBUILD; echo $pkgver`
    // then build with `--noextract` (because --nobuild already fetched things)

    // return and compare

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

        Pacman::new().U_all_status(built_pkg_paths).code().unwrap();
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
