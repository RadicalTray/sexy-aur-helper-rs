use crate::alpm::get_local_foreign_pkgs;
use crate::config::Config;
use crate::build::PkgInfo;
use crate::pacman::Pacman;
use crate::gen_config;
use crate::globals::Globals;
use crate::utils::prepare_clone;
use alpm::Alpm;
use std::process;

pub const STR: &str = "upgrade";

pub fn run(g: Globals, args: Vec<String>) -> Result<(), String> {
    prepare_clone(&g);

    if args.len() != 0 {
        return Err("unexpected arguments.".to_string());
    }

    // let status = Pacman::Syu_status();
    // if !status.success() {
    //     process::exit(status.code().unwrap());
    // }

    let config_path = g.config_path;
    if !config_path.exists() {
        eprintln!("Run `saur {}` to generate config", gen_config::STR);
        return Err("config file doesn't exist.".to_string());
    }

    let config: Config = Config::new(config_path);
    config.check_config()?;

    let handle = Alpm::new("/", "/var/lib/pacman").unwrap();
    let local_foreign_pkgs = get_local_foreign_pkgs(&handle);
    if let Err(_) = config.check_pkgs(&local_foreign_pkgs) {
        process::exit(1);
    }

    // TODO:
    //  [ ] get version from local_foreign_pkgs
    //  [ ] turn into pkginfo
    //  [ ] fetch pkgs
    //  [ ] build and install fetched pkgs

    Ok(())
}

// fn get_versions(local_pkgs: Vec<&Package>, pkg_list: toml::Value::Array) {
// }

// fn build_and_install(pkgs: Vec<PkgInfo>) {
//     for pkg in pkgs {
//         let cwd = clone_path.clone().join(pkg.name());
//         let install_infos = match build(cwd, true, PkgInfo::new(pkg.name().to_string())) {
//             Ok(v) => v,
//             Err(pkg) => {
//                 err_pkgs.push(pkg);
//                 continue;
//             }
//         };
//
//         let mut success = true;
//         for install_info in install_infos {
//             let s = Pacman { yes: true }.U_all_status(&install_info).success();
//             if !s {
//                 success = false;
//                 break;
//             }
//         }
//         if !success {
//             eprintln!("Error while installing: {}", pkg.name());
//             break;
//         }
//     }
//
//     if err_pkgs.len() > 0 {
//         eprintln!("Error while building:");
//         for pkg in err_pkgs {
//             eprintln!("\t{pkg}");
//         }
//         process::exit(1);
//     }
// }
