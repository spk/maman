//! Maman is a Rust Web Crawler saving pages on Redis.
//!
//! # Default environment variables
//!
//! * `MAMAN_ENV`=development
//! * `REDIS_URL`="redis://127.0.0.1/"
#![doc(html_root_url = "https://docs.rs/maman/0.11.0")]
#![deny(warnings)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

#![crate_name = "maman"]

extern crate string_cache;
#[macro_use]
extern crate log;
extern crate html5ever;
#[macro_use]
extern crate html5ever_atoms;
extern crate tendril;
extern crate url;
extern crate url_serde;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate robotparser;
extern crate sidekiq;
extern crate encoding;

#[macro_export]
macro_rules! maman_name {
    () => ( "Maman" )
}
#[macro_export]
macro_rules! maman_version {
    () => ( env!("CARGO_PKG_VERSION") )
}
#[macro_export]
macro_rules! maman_version_string {
    () => ( concat!(maman_name!(), " v", maman_version!()) )
}
#[macro_export]
macro_rules! maman_user_agent {
    () => ( concat!(maman_version_string!(), " (https://crates.io/crates/maman)") )
}

pub use maman::{Spider, Page};
pub mod maman;
