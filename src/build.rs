use crate::git::Git;
use crate::makepkg::Makepkg;
use crate::pacman::InstallInfo;
use crate::utils::read_lines_to_strings;
use alpm::Package;
use alpm_utils::depends::satisfies_nover;
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
/// WON'T `git reset origin --hard` the aur repo
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

pub fn setup_build_stack<'a>(pkgs_to_build: &Vec<&'a Package>) -> Vec<&'a Package> {
    let mut build_stack: Vec<_> = Vec::with_capacity(pkgs_to_build.len());

    // BUG: will not include a package that depends on each other
    // in reverse so the build stack has first packages last
    for pkg in pkgs_to_build.iter().rev() {
        let mut is_dep = false;
        for pkg_name in pkg.required_by() {
            if pkgs_to_build
                .iter()
                .map(|x| x.name())
                .collect::<Vec<&str>>()
                .contains(&pkg_name.as_str())
            {
                is_dep = true;
                break;
            }
        }

        if !is_dep {
            push_to_build_stack(&pkgs_to_build, &mut build_stack, pkg);
        }
    }

    build_stack
}

// algorithm to build deps first
//
// NOTE: doesn't check for makedepends and checkdepends
//  alpm's makedepends and checkdepends don't work on makepkg packages
fn push_to_build_stack<'a>(
    all_pkgs: &Vec<&'a Package>,
    stack: &mut Vec<&'a Package>,
    pkg: &'a Package,
) {
    stack.push(pkg);
    let deps = pkg.depends();
    for dep in deps {
        for pkg in all_pkgs {
            if satisfies_nover(dep, pkg.name(), pkg.provides().into_iter()) {
                push_to_build_stack(all_pkgs, stack, pkg);
                break;
            }
        }
    }
}
