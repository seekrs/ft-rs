#![deny(clippy::all)]
#![deny(clippy::cargo)]
//#![deny(missing_docs)]

#![allow(clippy::module_name_repetitions)]
#![allow(clippy::items_after_statements)]

//! # ft-rs
//! A Rust library for the 42School API.

pub mod error;
pub use error::*;
pub mod client;
pub mod models;
pub use client::FtClient;
