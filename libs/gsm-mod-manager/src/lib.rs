mod errors;
pub use errors::*;

mod managed_mod;
pub use managed_mod::ManagedMod;

mod constants;
mod parse_mod_string;

pub fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_runs_without_panicking() {
        main();
    }
}
