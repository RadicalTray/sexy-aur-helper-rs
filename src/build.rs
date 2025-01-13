use crate::git::Git;
use crate::makepkg::Makepkg;
use crate::utils::read_lines_to_strings;
use std::path::PathBuf;

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
pub fn build_all(clone_path: &PathBuf, pkgs: Vec<PkgInfo>) -> (Vec<String>, Vec<String>) {
    let mut built_pkg_paths = Vec::with_capacity(pkgs.len());
    let mut err_pkgs = Vec::new();

    for pkg in pkgs {
        let cwd = clone_path.clone().join(&pkg.name);

        Git::cwd(cwd.clone()).reset_hard_origin();

        match build(cwd, false, &pkg.name) {
            Ok(v) => built_pkg_paths.extend(v),
            Err(pkg) => {
                err_pkgs.push(pkg);
                continue;
            }
        };
    }

    (built_pkg_paths, err_pkgs)
}

/// # NOTES
/// WON'T `git reset orign --hard` the aur repo
pub fn build(cwd: PathBuf, noextract: bool, pkg: &str) -> Result<Vec<String>, String> {
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
                ..Default::default()
            }
            .output();

            Ok(read_lines_to_strings(
                String::from_utf8(output.stdout).expect("Output not UTF-8"),
            ))
        }
        _ => Err(pkg.to_string()),
    }
}
