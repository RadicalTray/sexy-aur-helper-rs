use crate::globals::Globals;
use crate::pkg::upgrade;

pub const STR: &str = "upgrade";

pub fn run(g: Globals, _args: Vec<String>) -> Result<(), String> {
    upgrade(&g);
    Ok(())
}
