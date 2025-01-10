use crate::globals::*;
use std::path::PathBuf;
use std::process::Command;

/// `filename` should be escapeable with ''
pub fn fetch_aur_data(url: &str, curr_dir: &PathBuf, filename: &str) -> Result<(), String> {
    let sh_cmd = format!("curl {url} | gzip -cd > '{filename}'");

    let status = Command::new("sh")
        .args(["-c", sh_cmd.as_str()])
        .current_dir(curr_dir)
        .status();

    let status = match status {
        Ok(o) => o,
        Err(_) => return Err(String::from("sh could not be executed")),
    };

    if !status.success() {
        return Err(format!("`{sh_cmd}` failed"));
    }

    Ok(())
}

pub fn fetch_pkglist(g: &Globals) -> Result<(), String> {
    fetch_aur_data(URL_PKG, &g.cache_path, FILENAME_PKG)
}

pub fn fetch_pkgbase(g: &Globals) -> Result<(), String> {
    fetch_aur_data(URL_PKGBASE, &g.cache_path, FILENAME_PKGBASE)
}
