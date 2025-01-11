use crate::git::Git;
use crate::globals::*;
use crate::threadpool::ThreadPool;
use crate::utils::*;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub fn fetch_pkgs(
    clone_path: &PathBuf,
    pkgs: Vec<String>,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let clone_path = Arc::new(clone_path.clone());
    env::set_current_dir(&*clone_path).unwrap();

    let pool = ThreadPool::new(std::thread::available_parallelism().unwrap().get()).unwrap();
    let old_pkgs = Arc::new(Mutex::new(Vec::with_capacity(pkgs.len())));
    let new_pkgs_n_outputs = Arc::new(Mutex::new(Vec::new()));
    let err_pkgs = Arc::new(Mutex::new(Vec::new()));

    for pkg in pkgs {
        let clone_path = Arc::clone(&clone_path);
        let o = Arc::clone(&old_pkgs);
        let n = Arc::clone(&new_pkgs_n_outputs);
        let e = Arc::clone(&err_pkgs);
        pool.execute(move || {
            fetch_pkg(&*clone_path, pkg, o, n, e);
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
    println!();
    for (pkg, output) in &new_pkgs_n_outputs {
        println!("{pkg}:");
        println!("{output}");
        println!();
    }
    for pkg in &err_pkgs {
        println!("{pkg}: Error happend while fetching/cloning!");
    }
    println!();
    prompt_accept();

    // do i need to clone x.0
    (
        old_pkgs,
        new_pkgs_n_outputs.iter().map(|x| x.0.clone()).collect(),
        err_pkgs,
    )
}

fn fetch_pkg(
    clone_path: &PathBuf,
    pkg: String,
    old_pkgs: Arc<Mutex<Vec<String>>>,
    new_pkgs_n_outputs: Arc<Mutex<Vec<(String, String)>>>,
    err_pkgs: Arc<Mutex<Vec<String>>>,
) {
    println!("Fetching {pkg}");
    let pkg_dir = clone_path.clone().join(&pkg);
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
            Git::cwd(clone_path.to_path_buf()).clone(format!("{URL_AUR}/{pkg}.git").as_str());

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
