use crate::globals::Globals;
use crate::pkg::{get_pkgbases, sync};

pub const STR: &str = "sync";

pub fn run(g: Globals, args: Vec<String>) -> Result<(), String> {
    if args.len() == 0 {
        return Err(String::from("no package specified"));
    }

    let pkgbases = get_pkgbases(&g)?;

    let mut pkgs_not_found = Vec::new();
    for pkg in &args {
        if !pkgbases.contains(&pkg) {
            pkgs_not_found.push(pkg);
        }
    }
    if pkgs_not_found.len() > 0 {
        println!("package not found in package base list:");
        for pkg in pkgs_not_found {
            println!("\t{pkg}");
        }
        return Ok(());
    }

    sync(&g, args, true);

    Ok(())
}
