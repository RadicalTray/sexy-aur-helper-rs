use crate::cmds::fetch_aur_data;
use crate::urls;
use crate::globals;

pub const STR: &str = "update-pkg-list";

pub fn run(g: globals::Globals) -> Result<(), String>  {
    let cache_path = &g.cache_path;

    fetch_aur_data(urls::PKG, cache_path, "packages")?;
    fetch_aur_data(urls::PKGBASE, cache_path, "pkgbase")?;

    Ok(())
}
