use crate::alpm::get_local_aur_pkgs;
use crate::build::*;
use crate::fetch::*;
use crate::globals::Globals;
use crate::pacman::Pacman;
use crate::utils::*;
use crate::ver::get_pkgs_to_upgrade;
use alpm::{Alpm, Version};
use std::collections::HashSet;
use std::process;

pub const STR: &str = "upgrade-old";

pub fn run(g: Globals, args: Vec<String>) -> Result<(), String> {
    if args.len() != 0 {
        return Err("unexpected arguments".to_string());
    }

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
    let (mut base_pkgs, _not_base_pkgs, err_pkgs) = get_local_aur_pkgs(&handle, &g);
    for pkg in err_pkgs {
        println!("{} not found in aur!", pkg.name());
    }
    println!();

    let clone_path = g.cache_path.clone().join("clone");
    let (mut old_pkgs, new_pkgs, _err_pkgs) = fetch_pkgs(
        &clone_path,
        base_pkgs
            .iter()
            .map(|x| {
                (
                    PkgInfo::new(x.name().to_string()),
                    Some(Version::new(x.version().as_str())),
                )
            })
            .collect(),
    );
    old_pkgs.extend(new_pkgs);
    let fetched_pkgs = old_pkgs;

    let (pkgs_to_upgrade, err_pkgs) = get_pkgs_to_upgrade(&clone_path, fetched_pkgs);
    if err_pkgs.len() > 0 {
        for pkg in err_pkgs {
            eprintln!("{pkg}: error while fetching package source!");
        }
        eprintln!();
    }

    base_pkgs.retain(|x| pkgs_to_upgrade.contains(x.name()));
    let pkgs_to_upgrade: Vec<_> = base_pkgs;

    if pkgs_to_upgrade.len() == 0 {
        println!("Nothing to upgrade.");
        return;
    }

    println!("{} packages to be built:", pkgs_to_upgrade.len());
    for pkg in &pkgs_to_upgrade {
        println!("\t{}", pkg.name());
    }
    prompt_accept();

    let build_stack = setup_build_stack(&pkgs_to_upgrade);
    let mut set: HashSet<&str> = HashSet::new();

    let mut err_pkgs: Vec<String> = Vec::new();
    for pkg in build_stack.iter().rev() {
        if set.contains(pkg.name()) {
            continue;
        }
        set.insert(pkg.name());

        let cwd = clone_path.clone().join(pkg.name());
        let install_infos = match build(cwd, true, PkgInfo::new(pkg.name().to_string())) {
            Ok(v) => v,
            Err(pkg) => {
                err_pkgs.push(pkg);
                continue;
            }
        };

        let mut success = true;
        for install_info in install_infos {
            let s = Pacman { yes: true }.U_all_status(&install_info).success();
            if !s {
                success = false;
                break;
            }
        }

        if !success {
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

    if set.len() != pkgs_to_upgrade.len() {
        println!("pkg not in build stack:");
        for pkg in pkgs_to_upgrade {
            if !set.contains(pkg.name()) {
                println!("\t{}", pkg.name());
            }
            // set containing pkgs not in pkgs_to_upgrade should be impossible
        }
        process::exit(1);
    }
}
