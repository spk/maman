#[macro_use]
extern crate maman;

use std::env;
use std::process;

use maman::Spider;

#[cfg(not(test))]
fn main() {
    let url = match env::args().nth(1) {
        Some(url) => url,
        None => {
            println!(maman_version_string!());
            println!("Usage: maman URL [LIMIT]");
            process::exit(1);
        }
    };
    let limit = match env::args().nth(2) {
        Some(limit) => {
            match limit.parse::<isize>() {
                Err(_) => 0,
                Ok(l) => l,
            }
        }
        None => 0,
    };

    let mut spider = Spider::new(url, limit, vec![]);
    spider.crawl()
}
