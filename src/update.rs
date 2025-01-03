use crate::cmds::{fetch_pkg, fetch_pkgbase};
use crate::globals::*;

pub const STR: &str = "update-pkg-list";

pub fn run(g: Globals, _args: Vec<String>) -> Result<(), String>  {
    fetch_pkg(&g)?;
    fetch_pkgbase(&g)?;
    Ok(())
}
