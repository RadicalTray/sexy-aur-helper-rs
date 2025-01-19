use nix::unistd::geteuid;
use saur::run;
use std::env;
use std::process;

fn main() {
    if geteuid() == 0.into() {
        eprintln!("Makepkg cannot be run as root!");
        process::exit(1);
    }

    run(env::args());
}
