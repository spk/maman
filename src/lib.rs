#![crate_name = "maman"]

#[macro_use]
extern crate string_cache;
extern crate tendril;
extern crate html5ever;
extern crate url;
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate robotparser;
extern crate sidekiq;
extern crate encoding;

mod maman;
pub use maman::{Spider, Page};
