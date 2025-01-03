mod utils;
mod globals;
mod cmds;
mod search;
mod sync;
mod upgrade;
mod update;

use std::process;
use utils::print_error;

pub fn run(args: impl Iterator<Item = String>) {
    let args: Vec<String> = args.collect();

    let globals = match globals::Globals::build() {
        Ok(g) => g,
        Err(e) => {
            print_error(e);
            process::exit(1);
        }
    };

    let cmd = match args.get(1) {
        Some(arg) => arg,
        None => {
            print_error("no command specified");
            process::exit(1);
        }
    };

    if let Err(e) = match cmd.as_str() {
        search::STR => search::run(globals, args),
        sync::STR => sync::run(globals, args),
        upgrade::STR => upgrade::run(globals, args),
        update::STR => update::run(globals),
        _ => Err(format!("invalid command `{cmd}`")),
    } {
        print_error(&e);
        process::exit(1);
    }
}
