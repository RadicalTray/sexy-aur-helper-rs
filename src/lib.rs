#![feature(extract_if)]

mod alpm;
mod cmds;
mod globals;
mod pkg;
mod search;
mod sync;
mod update;
mod upgrade;
mod utils;

use ::alpm::Alpm;
use ::alpm::Package;
use std::process;
use utils::print_error_w_help;

pub fn run(mut args: impl Iterator<Item = String>) {
    args.next();

    let globals = match globals::Globals::build() {
        Ok(g) => g,
        Err(e) => {
            print_error_w_help(e);
            process::exit(1);
        }
    };

    let cmd = match args.next() {
        Some(arg) => arg,
        None => {
            print_error_w_help("no command specified");
            process::exit(1);
        }
    };

    let args: Vec<String> = args.collect();

    if let Err(e) = match cmd.as_str() {
        search::STR => search::run(globals, args),
        sync::STR => sync::run(globals, args),
        upgrade::STR => upgrade::run(globals, args),
        update::STR => update::run(globals, args),
        _ => Err(format!("invalid command `{cmd}`")),
    } {
        print_error_w_help(&e);
        process::exit(1);
    }
}

// manage deps
//  - get AUR packages that isn't required by anything
//      - push to stack
//  - check each depends
//      - push to stack
//  - build according to the stack
//      - check if there are any duplicates (set)

use std::collections::HashSet;

pub fn run_test() {
    let g = match globals::Globals::build() {
        Ok(g) => g,
        Err(e) => {
            print_error_w_help(e);
            process::exit(1);
        }
    };

    let handle = Alpm::new("/", "/var/lib/pacman").unwrap();
    let (aur_pkgs, err_pkgs) = alpm::get_local_aur_pkgs(&handle, &g);

    // let mut set = HashSet::new();
    let mut stack: Vec<_> = Vec::with_capacity(aur_pkgs.len());
    for pkg in &aur_pkgs {
        if pkg.required_by().len() == 0 {
            push_to_stack(&aur_pkgs, &mut stack, pkg);
        }
    }

    // println!("{:#?}", stack);
}

use alpm_utils::depends::satisfies_nover;

fn push_to_stack<'a>(all_pkgs: &Vec<&'a Package>, stack: &mut Vec<&'a Package>, pkg: &'a Package) {
    stack.push(pkg);
    for dep in pkg.depends() {
        for pkg in all_pkgs {
            if satisfies_nover(dep, pkg.name(), pkg.provides().into_iter()) {
                push_to_stack(all_pkgs, stack, pkg);
                break;
            }
        }
    }
}
