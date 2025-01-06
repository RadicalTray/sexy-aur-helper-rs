use crate::search;
use crate::sync;
use crate::update;
use crate::upgrade;

pub fn print_help(to_stderr: bool) {
    let s = format!(
        "\
Search the AUR package list:
\tsaur {} <search string>

Sync a package or multiple packages:
\tsaur {} <package name> [package name] ...

Upgrade system and AUR packages:
\tsaur {}

Update the AUR package list:
\tsaur {}
",
        search::STR,
        sync::STR,
        upgrade::STR,
        update::STR,
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
