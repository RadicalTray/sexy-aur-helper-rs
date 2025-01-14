use crate::clear;
use crate::globals::Globals;
use crate::search;
use crate::sync;
use crate::update;
use crate::upgrade;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process;

pub fn print_help(to_stderr: bool) {
    let s = format!(
        "\
Usage: saur <command> <arguments> ...

Search the AUR package list:
\tsaur {} <search string>

Sync a package or multiple packages:
\tsaur {} <package name> [package name] ...

Sync aur package a, b as dependencies and sync package c normally sequentially
\tsaur {} --asdeps a b --no-asdeps c

Upgrade system and AUR packages:
\tsaur {}

Update the AUR package list:
\tsaur {}

Clear built packages:
\tsaur {}
",
        search::STR,
        sync::STR,
        sync::STR,
        upgrade::STR,
        update::STR,
        clear::STR,
    );

    if to_stderr {
        eprint!("{}", s);
    } else {
        print!("{}", s);
    }
}

pub fn print_error_w_help(e: &str) {
    eprintln!("Error: {e}");
    eprintln!();
    print_help(true);
}

pub fn prompt_accept() {
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

pub fn read_file_lines_to_strings<P: AsRef<Path>>(filepath: P) -> Vec<String> {
    read_lines_to_strings(fs::read_to_string(filepath).unwrap())
}

pub fn read_lines_to_strings(s: String) -> Vec<String> {
    s.lines().map(String::from).collect()
}

pub fn prepare_clone(g: &Globals) {
    let clone_path = g.cache_path.clone().join("clone");
    if !clone_path.exists() {
        fs::create_dir(&clone_path).unwrap();
    }
    env::set_current_dir(clone_path).unwrap();
}
