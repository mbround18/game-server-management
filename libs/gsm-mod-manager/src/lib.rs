//! # gsm-mod-manager
//!
//! Mod download and installation support for GSM game server instances.
//!
//! This crate provides the [`ManagedMod`] type, which downloads a mod from a URL (or
//! constructs one from a Thunderstore package string such as
//! `"denikson-BepInExPack_Valheim-5.4.2202"`) and installs it into the correct directory
//! inside a game server installation.
//!
//! BepInEx framework mods (identified by the presence of `winhttp.dll` or a `BepInEx`
//! directory after extraction) are installed into the *game* directory, while all other
//! mods are installed into the *plugin* directory.
#![warn(missing_docs)]

mod errors;
pub use errors::*;

mod managed_mod;
pub use managed_mod::ManagedMod;

mod constants;
mod parse_mod_string;

/// Placeholder entry-point kept for compatibility; not intended for direct use.
pub fn main() {
    println!("Hello, world!");
}
