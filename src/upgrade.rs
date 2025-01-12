use crate::alpm::get_local_aur_pkgs;
use crate::build::*;
use crate::fetch::*;
use crate::globals::Globals;
use crate::pacman::Pacman;
use crate::utils::*;
use crate::ver::get_pkgs_to_upgrade;
use alpm::{Alpm, Package, Version};
use alpm_utils::depends::satisfies_nover;
use std::collections::HashSet;
use std::process;

pub const STR: &str = "upgrade";

pub fn run(g: Globals, _args: Vec<String>) -> Result<(), String> {
    upgrade(&g);
    Ok(())
}

fn upgrade(g: &Globals) {
    prepare_clone(g);

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

    let (pkgs_to_build, err_pkgs) = get_pkgs_to_upgrade(&clone_path, fetched_pkgs);
    if err_pkgs.len() > 0 {
        for pkg in err_pkgs {
            eprintln!("{pkg}: error while filtering packages to upgrade!");
        }
        eprintln!();
    }

    let pkgs_to_build: Vec<_> = aur_pkgs
        .iter()
        .filter(|x| pkgs_to_build.contains(x.name()))
        .collect();

    if pkgs_to_build.len() == 0 {
        println!("Nothing to build.");
        return;
    }

    println!("{} packages to be built:", pkgs_to_build.len());
    for pkg in &pkgs_to_build {
        println!("\t{}", pkg.name());
    }
    prompt_accept();

    let build_stack = setup_build_stack(&pkgs_to_build);
    let mut set: HashSet<&str> = HashSet::new();

    let mut err_pkgs: Vec<String> = Vec::new();
    for pkg in build_stack.iter().rev() {
        if set.contains(pkg.name()) {
            continue;
        }
        set.insert(pkg.name());

        let cwd = clone_path.clone().join(pkg.name());
        let built_pkg_paths = match build(cwd, true, pkg.name()) {
            Ok(v) => v,
            Err(pkg) => {
                err_pkgs.push(pkg);
                continue;
            }
        };

        if !(Pacman {
            yes: true,
            ..Default::default()
        })
        .U_all_status(built_pkg_paths)
        .success()
        {
            err_pkgs.push(pkg.name().to_string());
            break;
        }
    }

    if err_pkgs.len() > 0 {
        eprintln!("Error happened while building:");
        for pkg in err_pkgs {
            eprintln!("\t{pkg}");
        }
    }

    if set.len() != pkgs_to_build.len() {
        println!("pkg not in build stack:");
        for pkg in pkgs_to_build {
            if !set.contains(pkg.name()) {
                println!("\t{}", pkg.name());
            }
            // set containing pkgs not in pkgs_to_build should be impossible
        }
        process::exit(1);
    }
}

fn setup_build_stack<'a>(pkgs: &Vec<&'a &'a Package>) -> Vec<&'a &'a Package> {
    let mut build_stack: Vec<_> = Vec::with_capacity(pkgs.len());

    // BUG: will not include a package that depends on each other
    // in reverse so the build stack has first packages last
    for pkg in pkgs.iter().rev() {
        let is_aur_pkg_dep = {
            let mut tmp = false;
            for pkg_name in pkg.required_by() {
                if pkgs
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
            push_to_build_stack(&pkgs, &mut build_stack, pkg);
        }
    }

    build_stack
}

// algorithm to build deps first
fn push_to_build_stack<'a>(
    all_pkgs: &Vec<&'a &'a Package>,
    stack: &mut Vec<&'a &'a Package>,
    pkg: &'a &'a Package,
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
