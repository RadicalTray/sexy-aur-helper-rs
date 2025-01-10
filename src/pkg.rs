// TODO: refactor

use crate::alpm::get_local_aur_pkgs;
use crate::cmds::{fetch_pkgbase, fetch_pkglist};
use crate::git::Git;
use crate::globals::*;
use crate::makepkg::Makepkg;
use crate::pacman::Pacman;
use crate::threadpool::ThreadPool;
use alpm::{Alpm, Package};
use alpm_utils::depends::satisfies_nover;
use std::collections::HashSet;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::{Arc, Mutex};
use std::{env, fs};

fn prepare(g: &Globals) {
    let clone_path = g.cache_path.clone().join("clone");
    if !clone_path.exists() {
        fs::create_dir(&clone_path).unwrap();
    }
    env::set_current_dir(clone_path).unwrap();
}

pub fn upgrade(g: &Globals) {
    prepare(g);

    let status = Pacman::Syu_status();
    if !status.success() {
        process::exit(1);
    }

    let handle = Alpm::new("/", "/var/lib/pacman").unwrap();
    let (aur_pkgs, err_pkgs) = get_local_aur_pkgs(&handle, &g);
    for pkg in err_pkgs {
        println!("{} not found in aur!", pkg.name());
    }
    println!();

    let clone_path = g.cache_path.clone().join("clone");
    fetch_pkgs(
        &clone_path,
        aur_pkgs.iter().map(|x| x.name().to_string()).collect(),
    );

    println!("{} packages to be built:", aur_pkgs.len());
    for pkg in &aur_pkgs {
        println!("\t{}", pkg.name());
    }
    prompt_accept();

    // below could be run in another thread then wait for it while user is reading
    let build_stack = setup_build_stack(&aur_pkgs);
    let mut set: HashSet<&str> = HashSet::new();

    let mut err_pkgs: Vec<String> = Vec::new();
    for pkg in build_stack.iter().rev() {
        if set.contains(pkg.name()) {
            continue;
        }
        set.insert(pkg.name());

        // TODO: getver
        //  - run `makepkg --nobuild`
        //  - run `source PKGBUILD; echo $pkgver`
        // then build with `--noextract` (because --nobuild already fetched things)

        // if new_ver(pkg.ver(), new_ver) {
        //
        // }

        let cwd = clone_path.clone().join(pkg.name());
        env::set_current_dir(cwd).unwrap();
        let built_pkg_paths = match makepkg(pkg.name()) {
            Ok(v) => v,
            Err(pkg) => {
                err_pkgs.push(pkg);
                continue;
            }
        };

        install(built_pkg_paths);
    }

    if set.len() != aur_pkgs.len() {
        println!("pkg not in build stack:");
        for pkg in aur_pkgs {
            if !set.contains(pkg.name()) {
                println!("\t{}", pkg.name());
            }
            // set containing pkgs not in aur_pkgs should be impossible
        }
        process::exit(1);
    }
}

fn setup_build_stack<'a>(aur_pkgs: &Vec<&'a Package>) -> Vec<&'a Package> {
    let mut build_stack: Vec<_> = Vec::with_capacity(aur_pkgs.len());

    // BUG: will not include a package that depends on each other
    // in reverse so the build stack has first packages last
    for pkg in aur_pkgs.iter().rev() {
        let is_aur_pkg_dep = {
            let mut tmp = false;
            for pkg_name in pkg.required_by() {
                if aur_pkgs
                    .iter()
                    .map(|x| x.name())
                    .collect::<Vec<&str>>()
                    .contains(&pkg_name.as_str())
                {
                    tmp = true;
                    break;
                }
            }

            tmp
        };

        if !is_aur_pkg_dep {
            push_to_build_stack(&aur_pkgs, &mut build_stack, pkg);
        }
    }

    build_stack
}

// algorithm to build deps first
fn push_to_build_stack<'a>(
    all_pkgs: &Vec<&'a Package>,
    stack: &mut Vec<&'a Package>,
    pkg: &'a Package,
) {
    stack.push(pkg);
    let mut deps = pkg.depends().to_list_mut();
    deps.extend(pkg.makedepends().iter());
    deps.extend(pkg.checkdepends().iter()); // NOTE: likely not needed
    for dep in deps {
        for pkg in all_pkgs {
            if satisfies_nover(dep, pkg.name(), pkg.provides().into_iter()) {
                push_to_build_stack(all_pkgs, stack, pkg);
                break;
            }
        }
    }
}

fn prompt_accept() {
    print!("Accept [Y/n] ");
    io::stdout().flush().unwrap();
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("read_line");
    let buffer = buffer.trim();
    if buffer != "" && buffer != "y" {
        println!();
        process::exit(1);
    }
}

// TODO:
//  0 notify changes in PKGBUILD
//  1 manage dependencies
//      - create dep tree
//  2 check if the pkg needs to be built
//  3 only install pkg that is new (in install())
pub fn sync(g: &Globals, pkgs: Vec<String>, quit_on_err: bool) {
    prepare(g);

    let clone_path = g.cache_path.clone().join("clone");
    let (cloned_pkgs, err_pkgs) = fetch_pkgs(&clone_path, pkgs);
    if quit_on_err && err_pkgs.len() > 0 {
        eprintln!("Error happened while cloning:");
        for pkg in err_pkgs {
            eprintln!("\t{pkg}");
        }
        return;
    }

    let (built_pkg_paths, err_pkgs) = makepkg_all(&clone_path, cloned_pkgs);
    if quit_on_err && err_pkgs.len() > 0 {
        eprintln!("Error happened while building:");
        for pkg in err_pkgs {
            eprintln!("\t{pkg}");
        }
        return;
    }

    // TODO: print stats

    let status_code = install(built_pkg_paths);

    process::exit(status_code);
}

// TODO: resolve the deps in here
fn fetch_pkgs(clone_path: &PathBuf, pkgs: Vec<String>) -> (Vec<String>, Vec<String>) {
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
    prompt_accept();

    let mut fetched_pkgs = Vec::from(old_pkgs);
    // NOTE: is there a way without cloning?
    fetched_pkgs.extend(new_pkgs_n_outputs.iter().map(move |x| x.0.clone()));
    (fetched_pkgs, err_pkgs)
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

fn makepkg_all(clone_path: &PathBuf, pkgs: Vec<String>) -> (Vec<String>, Vec<String>) {
    let mut built_pkg_paths = Vec::with_capacity(pkgs.len());
    let mut err_pkgs = Vec::new();

    for pkg in pkgs {
        let cwd = clone_path.clone().join(&pkg);

        Git::cwd(cwd.clone()).reset_hard_origin();

        env::set_current_dir(cwd).unwrap();
        match makepkg(&pkg) {
            Ok(v) => built_pkg_paths.extend(v),
            Err(pkg) => {
                err_pkgs.push(pkg);
                continue;
            }
        };
    }

    (built_pkg_paths, err_pkgs)
}

fn makepkg(pkg: &str) -> Result<Vec<String>, String> {
    let status = Makepkg::new().status();
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

fn install(pkg_paths: Vec<String>) -> i32 {
    Pacman::new().U_all_status(pkg_paths).code().unwrap()
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

fn read_file_lines_to_strings<P: AsRef<Path>>(filepath: P) -> Vec<String> {
    read_lines_to_strings(fs::read_to_string(filepath).unwrap())
}

fn read_lines_to_strings(s: String) -> Vec<String> {
    s.lines().map(String::from).collect()
}
