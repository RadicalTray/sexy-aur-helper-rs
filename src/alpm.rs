use crate::globals::Globals;
use crate::pkg::{get_pkgbases, get_pkgs};
use alpm::{Alpm, Package};

pub fn get_local_aur_pkgs<'a>(
    handle: &'a Alpm,
    g: &Globals,
) -> (Vec<&'a Package>, Vec<&'a Package>) {
    let localdb = handle.localdb();
    let pkgbases = get_pkgbases(g).unwrap();
    let all_pkgs = get_pkgs(g).unwrap();

    let pkgs: Vec<_> = localdb
        .pkgs()
        .into_iter()
        .filter(|pkg| pkg.packager() == Some("Unknown Packager"))
        .collect();
    let mut aur_pkgs = Vec::with_capacity(pkgs.len());
    let mut err_pkgs = Vec::new();
    for pkg in pkgs {
        if pkgbases.iter().any(|x| x == pkg.name()) {
            aur_pkgs.push(pkg);
        } else if !all_pkgs.iter().any(|x| x == pkg.name()) {
            err_pkgs.push(pkg);
        }
    }

    (aur_pkgs, err_pkgs)
}
