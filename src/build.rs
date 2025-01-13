use crate::git::Git;
use crate::makepkg::Makepkg;
use crate::pacman::InstallInfo;
use crate::utils::read_lines_to_strings;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PkgInfo {
    pub name: String,
    pub needed: bool,
    pub asdeps: bool,
    pub asexplicit: bool,
    pub force: bool,
}

impl PkgInfo {
    pub fn new(name: String) -> Self {
        PkgInfo {
            name,
            needed: false,
            asdeps: false,
            asexplicit: false,
            force: false,
        }
    }
}

/// # NOTES
/// WILL `git reset orign --hard` the aur repo
pub fn build_all(clone_path: &PathBuf, pkgs: Vec<PkgInfo>) -> (Vec<InstallInfo>, Vec<String>) {
    let mut install_infos = Vec::with_capacity(pkgs.len());
    let mut err_pkgs = Vec::new();

    for pkg in pkgs {
        let cwd = clone_path.clone().join(&pkg.name);

        Git::cwd(cwd.clone()).reset_hard_origin();

        match build(cwd, false, pkg) {
            Ok(v) => install_infos.extend(v),
            Err(pkg) => {
                err_pkgs.push(pkg);
                continue;
            }
        };
    }

    (install_infos, err_pkgs)
}

/// # NOTES
/// WON'T `git reset orign --hard` the aur repo
pub fn build(cwd: PathBuf, noextract: bool, pkg: PkgInfo) -> Result<Vec<InstallInfo>, String> {
    match (Makepkg {
        cwd: cwd.clone(),
        noextract,
        ..Default::default()
    })
    .status()
    .code()
    .unwrap()
    {
        13 | 0 => {
            let output = Makepkg {
                cwd: cwd,
                packagelist: true,
                force: pkg.force,
                ..Default::default()
            }
            .output();

            let built_pkg_paths =
                read_lines_to_strings(String::from_utf8(output.stdout).expect("Output not UTF-8"));
            let mut install_infos = Vec::with_capacity(1);
            if built_pkg_paths.len() > 1 {
                let (main_pkgs, extra_pkgs) = built_pkg_paths.into_iter().partition(|p| {
                    Path::new(p)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .contains(&pkg.name)
                });
                install_infos.push(InstallInfo {
                    pkg_paths: main_pkgs,
                    needed: pkg.needed,
                    asdeps: pkg.asdeps,
                    asexplicit: pkg.asexplicit,
                });
                install_infos.push(InstallInfo {
                    pkg_paths: extra_pkgs,
                    needed: pkg.needed,
                    asdeps: true,
                    asexplicit: false,
                });
            } else {
                install_infos.push(InstallInfo {
                    pkg_paths: built_pkg_paths,
                    needed: pkg.needed,
                    asdeps: pkg.asdeps,
                    asexplicit: pkg.asexplicit,
                });
            }

            Ok(install_infos)
        }
        _ => Err(pkg.name),
    }
}
