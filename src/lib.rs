#![crate_name = "maman"]

#[macro_use]
extern crate string_cache;
#[macro_use]
extern crate log;
extern crate tendril;
extern crate html5ever;
extern crate url;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
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
