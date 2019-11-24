//! Maman is a Rust Web Crawler saving pages on Redis.
//!
//! # Default environment variables
//!
//! * `MAMAN_ENV`=development
//! * `REDIS_URL`="redis://127.0.0.1/"
#![doc(html_root_url = "https://docs.rs/maman/0.13.1")]
#![deny(warnings)]
#![crate_name = "maman"]

#[macro_use]
extern crate html5ever;
#[macro_use]
extern crate log;
extern crate mime;
extern crate reqwest;
extern crate robotparser;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate sidekiq;
extern crate url;

#[macro_export]
macro_rules! maman_name {
    () => {
        "Maman"
    };
}
#[macro_export]
macro_rules! maman_version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
}
#[macro_export]
macro_rules! maman_version_string {
    () => {
        concat!(maman_name!(), " v", maman_version!())
    };
}
#[macro_export]
macro_rules! maman_user_agent {
    () => {
        concat!(maman_version_string!(), " (https://crates.io/crates/maman)")
    };
}

pub use crate::maman::{Page, Spider};
pub mod maman;
