use crate::cmds::{fetch_pkg, fetch_pkgbase};
use crate::globals::*;

pub const STR: &str = "update-pkg-list";

pub fn run(g: Globals) -> Result<(), String>  {
    let cache_path = &g.cache_path;

    fetch_pkg(&g)?;
    fetch_pkgbase(&g)?;

    Ok(())
}
