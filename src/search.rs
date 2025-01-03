use std::fs;
use std::path::PathBuf;
use crate::globals::*;
use crate::cmds::{fetch_pkg, fetch_pkgbase};

pub const STR: &str = "search";

pub fn run(g: Globals, args: Vec<String>) -> Result<(), String> {
    if args.len() == 0 {
        return Err(String::from("no search word specified"));
    } else if args.len() > 1 {
        return Err(String::from("unexpected arguments"));
    }

    let (_, pkgbases) = get_pkglists(g)?;

    let result: Vec<&String> = pkgbases
        .iter()
        .filter(|s| s.contains(&args[0]))
        .collect();

    println!("Found {} AUR matches", result.len());

    let max = 25;
    if result.len() > max {
        println!("Number of matches exceeded max limit, showing {max} matches");
    }

    println!();

    let mut i = 0;
    for pkg in result {
        if i == max {
            break;
        }

        println!("{}", pkg);
        i += 1;
    }

    Ok(())
}

fn get_pkglists(g: Globals)
-> Result<(Vec<String>, Vec<String>), String> {
    let pkg_path = g.cache_path.clone().join(FILENAME_PKG);
    let pkgbase_path = g.cache_path.clone().join(FILENAME_PKGBASE);

    if !pkg_path.exists() {
        fetch_pkg(&g)?;
    }
    if !pkgbase_path.exists() {
        fetch_pkgbase(&g)?;
    }

    Ok((
        get_pkglist(pkg_path),
        get_pkglist(pkgbase_path)
    ))
}

fn get_pkglist(filepath: PathBuf) -> Vec<String> {
    fs::read_to_string(filepath)
        .unwrap()
        .lines()
        .map(String::from)
        .collect()
}
