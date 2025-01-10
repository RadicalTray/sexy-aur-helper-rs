#![feature(extract_if)]

mod alpm;
mod cmds;
mod git;
mod globals;
mod makepkg;
mod pacman;
mod pkg;
mod search;
mod sync;
mod threadpool;
mod update;
mod upgrade;
mod utils;

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
