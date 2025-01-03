use std::fs::File;
use std::path::PathBuf;
use std::io::{self, BufRead, ErrorKind::NotFound};
use crate::globals::*;
use crate::cmds::{fetch_pkg, fetch_pkgbase};

pub const STR: &str = "search";

pub fn run(g: Globals, args: Vec<String>) -> Result<(), String> {
    let (pkgs, pkgbases) = get_pkglists(g)?;
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

    Ok((vec![], vec![]))
}
