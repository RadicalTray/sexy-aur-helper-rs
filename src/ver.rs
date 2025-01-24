use crate::bash::Bash;
use crate::fetch::{PV, VPV};
use crate::git::Git;
use crate::makepkg::Makepkg;
use crate::threadpool::ThreadPool;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub fn get_pkgs_to_upgrade(clone_path: &PathBuf, pkgs: VPV) -> (HashSet<String>, Vec<String>) {
    println!("Fetching package sources.");
    let clone_path = Arc::new(clone_path.clone());
    env::set_current_dir(&*clone_path).unwrap();

    let pkgs_to_upgrade = Arc::new(Mutex::new(HashSet::new()));
    let err_pkgs = Arc::new(Mutex::new(Vec::new()));

    let pool = ThreadPool::new(std::thread::available_parallelism().unwrap().get()).unwrap();
    for pv in pkgs {
        let cp = Arc::clone(&clone_path);
        let p = Arc::clone(&pkgs_to_upgrade);
        let e = Arc::clone(&err_pkgs);
        pool.execute(move || {
            get_pkg_to_upgrade(&*cp, pv, p, e);
        });
    }
    drop(pool);
    let pkgs_to_upgrade = Arc::try_unwrap(pkgs_to_upgrade)
        .unwrap()
        .into_inner()
        .unwrap();
    let err_pkgs = Arc::try_unwrap(err_pkgs).unwrap().into_inner().unwrap();

    (pkgs_to_upgrade, err_pkgs)
}

fn get_pkg_to_upgrade(
    clone_path: &PathBuf,
    pv: PV,
    pkgs_to_upgrade: Arc<Mutex<HashSet<String>>>,
    err_pkgs: Arc<Mutex<Vec<String>>>,
) {
    let (pkg, old_ver) = match pv {
        (p, Some(v)) => (p, v),
        (_, None) => panic!("what the fuck are you doing?"),
    };

    let cwd = clone_path.clone().join(&pkg.name);

    let status = Git::cwd(cwd.clone()).reset_hard_origin_mute();
    if !status.success() {
        err_pkgs.lock().unwrap().push(pkg.name);
        return;
    }

    let makepkg = Makepkg {
        cwd: cwd.clone(),
        nobuild: true,
        ..Default::default()
    };
    let status = makepkg.status_mute();
    if !status.success() {
        err_pkgs.lock().unwrap().push(pkg.name);
        return;
    }

    let bash = Bash::cwd(cwd);
    let output = bash.output(
        "
source PKGBUILD
source /usr/share/makepkg/util/pkgbuild.sh
get_full_version
",
    );
    if !output.status.success() {
        err_pkgs.lock().unwrap().push(pkg.name);
        return;
    }

    let new_ver = String::from_utf8(output.stdout).expect("pkgver not UTF-8");
    let new_ver = new_ver.trim();

    match alpm::vercmp(old_ver.as_str(), new_ver) {
        Less => {
            pkgs_to_upgrade.lock().unwrap().insert(pkg.name);
        }
        Equal => (),
        Greater => {
            println!("`{}` has a higher version!", pkg.name);
        }
    }
}
