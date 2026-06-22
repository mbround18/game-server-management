//! # gsm-serde
//!
//! INI-format serialization and deserialization for GSM game server configuration files.
//!
//! This crate exposes the [`serde_ini`] module which provides:
//!
//! - [`serde_ini::to_string`] – serialize any `Serialize + IniHeader` type into an
//!   INI-formatted string with a `[section]` header.
//! - [`serde_ini::from_str`] – deserialize an INI-formatted string back into any
//!   `DeserializeOwned` type.
//! - [`serde_ini::IniHeader`] – a trait (also derivable via `ini-derive`'s
//!   `#[derive(IniSerialize)]`) that provides the `[section]` header string.
//!
//! Nested structs are represented as INI-style parenthesised blocks, e.g.:
//!
//! ```ini
//! [/Script/Pal.PalGameWorldSettings]
//! OptionSettings=(
//!     Difficulty="Hard",
//! )
//! ```
#![warn(missing_docs)]

/// INI serialization and deserialization utilities.
///
/// See the module-level documentation for [`serde_ini::to_string`] and
/// [`serde_ini::from_str`] for usage examples.
pub mod serde_ini;
