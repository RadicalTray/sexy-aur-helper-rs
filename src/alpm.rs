use crate::fetch::{get_pkgbases, get_pkgs};
use crate::globals::Globals;
use alpm::{Alpm, Package};

pub fn get_local_aur_pkgs<'a>(
    handle: &'a Alpm,
    g: &Globals,
) -> (Vec<&'a Package>, Vec<&'a Package>, Vec<&'a Package>) {
    let pkgs = get_local_foreign_pkgs(handle);
    let pkgbases = get_pkgbases(g).unwrap();
    let all_pkgs = get_pkgs(g).unwrap();
    let mut base_pkgs = Vec::with_capacity(pkgs.len());
    let mut not_base_pkgs = Vec::new();
    let mut err_pkgs = Vec::new();
    for pkg in pkgs {
        if pkgbases.iter().any(|x| x == pkg.name()) {
            base_pkgs.push(pkg);
        } else if all_pkgs.iter().any(|x| x == pkg.name()) {
            not_base_pkgs.push(pkg);
        } else {
            err_pkgs.push(pkg);
        }
    }

    (base_pkgs, not_base_pkgs, err_pkgs)
}

pub fn get_local_foreign_pkgs<'a>(handle: &'a Alpm) -> Vec<&'a Package> {
    let localdb = handle.localdb();
    localdb
        .pkgs()
        .into_iter()
        .filter(|pkg| pkg.packager() == Some("Unknown Packager"))
        .collect()
}
