mod utils;
mod urls;
mod globals;
mod cmds;
mod search;
mod sync;
mod upgrade;
mod update;

use std::process;
use utils::print_error;

pub fn run(mut args: impl Iterator<Item = String>) {
    let globals = match globals::Globals::build() {
        Ok(g) => g,
        Err(e) => {
            print_error(e);
            process::exit(1);
        }
    };

    args.next();

    let cmd = match args.next() {
        Some(arg) => arg,
        None => {
            print_error("no command specified");
            process::exit(1);
        }
    };

    if let Err(e) = match cmd.as_str() {
        search::STR => search::run(args),
        sync::STR => sync::run(args),
        upgrade::STR => upgrade::run(args),
        update::STR => update::run(globals),
        _ => Err(format!("invalid command `{cmd}`")),
    } {
        print_error(&e);
        process::exit(1);
    }
}
