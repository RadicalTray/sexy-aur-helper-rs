use crate::fetch::{fetch_pkgbase, fetch_pkglist};
use crate::globals::*;

pub const STR: &str = "update-pkg-list";

pub fn run(g: Globals, args: Vec<String>) -> Result<(), String> {
    if args.len() != 0 {
        return Err("unexpected arguments".to_string());
    }

    fetch_pkglist(&g)?;
    fetch_pkgbase(&g)?;
    Ok(())
}
