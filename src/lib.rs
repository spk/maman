#![crate_name = "maman"]

#[macro_use]
extern crate string_cache;
extern crate tendril;
extern crate html5ever;
extern crate url;
extern crate hyper;
extern crate redis;
extern crate rustc_serialize;
extern crate time;
extern crate rand;

mod maman;
pub use maman::{Spider, Page};
