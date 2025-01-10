use crate::cmds::{fetch_pkgbase, fetch_pkglist};
use crate::globals::*;

pub const STR: &str = "update-pkg-list";

pub fn run(g: Globals, _args: Vec<String>) -> Result<(), String> {
    fetch_pkglist(&g)?;
    fetch_pkgbase(&g)?;
    Ok(())
}
