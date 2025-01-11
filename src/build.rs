use crate::git::Git;
use crate::makepkg::Makepkg;
use crate::utils::read_lines_to_strings;
use std::path::PathBuf;

pub fn build_all(clone_path: &PathBuf, pkgs: Vec<String>) -> (Vec<String>, Vec<String>) {
    let mut built_pkg_paths = Vec::with_capacity(pkgs.len());
    let mut err_pkgs = Vec::new();

    for pkg in pkgs {
        let cwd = clone_path.clone().join(&pkg);

        Git::cwd(cwd.clone()).reset_hard_origin();

        match build(cwd, false, &pkg) {
            Ok(v) => built_pkg_paths.extend(v),
            Err(pkg) => {
                err_pkgs.push(pkg);
                continue;
            }
        };
    }

    (built_pkg_paths, err_pkgs)
}

pub fn build(cwd: PathBuf, noextract: bool, pkg: &str) -> Result<Vec<String>, String> {
    let status = Makepkg {
        cwd,
        noextract,
        ..Default::default()
    }
    .status();
    match status.code().unwrap() {
        13 | 0 => {
            let output = Makepkg {
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
