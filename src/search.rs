use crate::globals::Globals;
use crate::pkg::get_pkgbases;

pub const STR: &str = "search";

pub fn run(g: Globals, args: Vec<String>) -> Result<(), String> {
    if args.len() == 0 {
        return Err(String::from("no search word specified"));
    } else if args.len() > 1 {
        return Err(String::from("unexpected arguments"));
    }

    let pkgbases = get_pkgbases(&g)?;

    let result: Vec<&String> = pkgbases.iter().filter(|s| s.contains(&args[0])).collect();

    println!("Found {} AUR matches", result.len());

    let max = 25;
    if result.len() > max {
        println!("Number of matches exceeded max limit, showing {max} matches");
    }

    println!();

    let mut i = 0;
    for pkg in result {
        if i == max {
            break;
        }

        println!("{}", pkg);
        i += 1;
    }

    Ok(())
}
