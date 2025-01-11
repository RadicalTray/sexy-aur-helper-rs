use crate::fetch::{get_pkgbases, is_in_pkgbases};
use crate::globals::Globals;
use crate::pkg::sync;

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
