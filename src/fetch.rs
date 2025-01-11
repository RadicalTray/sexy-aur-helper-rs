use crate::git::Git;
use crate::globals::*;
use crate::threadpool::ThreadPool;
use crate::utils::*;
use alpm::Version;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

// (PackageName, Option<Version>)
pub type PV = (String, Option<Version>);
pub type VPV = Vec<(String, Option<Version>)>;

pub fn fetch_pkgs(clone_path: &PathBuf, pkgs: VPV) -> (VPV, VPV, VPV) {
    let clone_path = Arc::new(clone_path.clone());
    env::set_current_dir(&*clone_path).unwrap();

    let pool = ThreadPool::new(std::thread::available_parallelism().unwrap().get()).unwrap();
    let old_pkgs = Arc::new(Mutex::new(Vec::with_capacity(pkgs.len())));
    let new_pkgs_n_outputs = Arc::new(Mutex::new(Vec::new()));
    let err_pkgs = Arc::new(Mutex::new(Vec::new()));

    for pv in pkgs {
        let clone_path = Arc::clone(&clone_path);
        let o = Arc::clone(&old_pkgs);
        let n = Arc::clone(&new_pkgs_n_outputs);
        let e = Arc::clone(&err_pkgs);
        pool.execute(move || {
            fetch_pkg(&*clone_path, pv, o, n, e);
        });
    }
    drop(pool); // basically wait (too lazy to properly impl it)
    let old_pkgs = Arc::try_unwrap(old_pkgs).unwrap().into_inner().unwrap();
    let new_pkgs_n_outputs = Arc::try_unwrap(new_pkgs_n_outputs)
        .unwrap()
        .into_inner()
        .unwrap();
    let err_pkgs = Arc::try_unwrap(err_pkgs).unwrap().into_inner().unwrap();

    // TODO: use pager?
    if new_pkgs_n_outputs.len() > 0 {
        println!("AUR Repo diffs:");
        for ((pkg, _), output) in &new_pkgs_n_outputs {
            println!("{pkg}:");
            println!("{output}");
            println!();
        }
        println!();
        prompt_accept();
    } else if err_pkgs.len() == 0 {
        println!("No changes in AUR repo.");
    }

    if err_pkgs.len() > 0 {
        println!("Error occured while fetching/cloning!");
        for (pkg, _) in &err_pkgs {
            println!("{pkg}: Error happend while fetching/cloning!");
        }
    }

    // do i need to clone x.0
    (
        old_pkgs,
        new_pkgs_n_outputs.iter().map(|x| x.0.clone()).collect(),
        err_pkgs,
    )
}

fn fetch_pkg(
    clone_path: &PathBuf,
    pkg: PV,
    old_pkgs: Arc<Mutex<VPV>>,
    new_pkgs_n_outputs: Arc<Mutex<Vec<(PV, String)>>>,
    err_pkgs: Arc<Mutex<VPV>>,
) {
    println!("Fetching {}", pkg.0);
    let pkg_dir = clone_path.clone().join(&pkg.0);
    if pkg_dir.exists() {
        let git = Git::cwd(pkg_dir);
        let status = git.fetch();

        if !status.success() {
            err_pkgs.lock().unwrap().push(pkg);
        } else {
            let diff_output = String::from_utf8(git.diff_fetch_color().stdout).expect("UTF-8");
            if !diff_output.trim().is_empty() {
                new_pkgs_n_outputs.lock().unwrap().push((pkg, diff_output));
            } else {
                old_pkgs.lock().unwrap().push(pkg);
            }
        }
    } else {
        let status =
            Git::cwd(clone_path.to_path_buf()).clone(format!("{URL_AUR}/{}.git", pkg.0).as_str());

        if !status.success() {
            err_pkgs.lock().unwrap().push(pkg);
        } else {
            let output = read_dir_files(&pkg_dir);
            new_pkgs_n_outputs.lock().unwrap().push((pkg, output));
        }
    };
}

fn read_dir_files(dir: &PathBuf) -> String {
    let mut outputs = String::new();
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let output = match fs::read_to_string(entry.path()) {
            Ok(s) => {
                format!("{}:\n{}\n", entry.path().display(), s)
            }
            Err(_) => {
                format!(
                    "{}: not UTF-8 or something idk.\n\n",
                    entry.path().display()
                )
            }
        };

        outputs.push_str(&output);
    }
    outputs
}

pub fn get_pkgbases(g: &Globals) -> Result<Vec<String>, String> {
    let pkgbase_path = g.cache_path.clone().join(FILENAME_PKGBASE);
    if !pkgbase_path.exists() {
        fetch_pkgbase(g)?;
    }
    Ok(read_file_lines_to_strings(pkgbase_path))
}

pub fn get_pkgs(g: &Globals) -> Result<Vec<String>, String> {
    let pkg_path = g.cache_path.clone().join(FILENAME_PKG);
    if !pkg_path.exists() {
        fetch_pkglist(g)?;
    }

    Ok(read_file_lines_to_strings(pkg_path))
}

pub fn is_in_pkgbases(pkgbases: &Vec<String>, mut pkgs: Vec<String>) -> (Vec<String>, Vec<String>) {
    let err_pkgs = pkgs.extract_if(.., |pkg| !pkgbases.contains(pkg)).collect();
    (pkgs, err_pkgs)
}

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
