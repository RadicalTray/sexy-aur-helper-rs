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
    for pkg in err_pkgs {
        println!("{:?}", pkg);
    }
    for pkg in aur_pkgs {
        println!("{:?}", pkg.provides());
    }
}

struct Node {
    parents: Vec<Box<Node>>,
    children: Vec<Box<Node>>,
}

impl Node {
    fn new() -> Node {
        Node {
            parents: Vec::new(),
            children: Vec::new(),
        }
    }
    fn add_parent(&mut self, parent: Node) {
        self.parents.push(parent);
    }
    fn add_child(&mut self, child: Node) {
        self.children.push(child);
    }
}

fn build_tree() {}
