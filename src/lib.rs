use std::{
    process,
    fmt,
};

enum Cmd {
    Search,
    Sync,
    Upgrade,
    Update,
}

impl fmt::Display for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = match self {
            Cmd::Search => "search",
            Cmd::Sync => "sync",
            Cmd::Upgrade => "upgrade",
            Cmd::Update => "update-pkg-list",
        };
        write!(f, "{val}")
    }
}

fn print_help(to_stderr: bool) {
    let s = format!("\
Search the AUR package list:
\tsaur {} <search string>
Sync a package or multiple packages:
\tsaur {} <package name> [package name] ...
Upgrade system and AUR packages:
\tsaur {}
Update the AUR package list:
\tsaur {}",
        Cmd::Search,
        Cmd::Sync,
        Cmd::Upgrade,
        Cmd::Update
    );

    if to_stderr {
        eprintln!("{}", s);
    } else {
        println!("{}", s);
    }
}

pub fn run(mut args: impl Iterator<Item = String>) {
    args.next();

    let cmd = match args.next() {
        Some(arg) => arg,
        None => {
            print_help(true);
            process::exit(1);
        }
    };

}
