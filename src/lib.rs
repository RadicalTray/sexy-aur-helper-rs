mod utils;
mod search;
mod sync;
mod upgrade;
mod update;

use std::process;
use utils::print_help;

pub fn run(mut args: impl Iterator<Item = String>) {
    args.next();

    let cmd = match args.next() {
        Some(arg) => arg,
        None => {
            print_help(true);
            process::exit(1);
        }
    };

    if let Err(e) = match cmd.as_str() {
        search::STR => search::run(args),
        sync::STR => sync::run(args),
        upgrade::STR => upgrade::run(args),
        update::STR => update::run(),
        _ => Err(format!("invalid command {cmd}")),
    } {
        eprintln!("Error: {e}");
        eprintln!();
        print_help(true);
    }
}
