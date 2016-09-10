#[macro_use]
extern crate log;
extern crate env_logger;
extern crate url;

#[macro_use]
extern crate maman;
extern crate sidekiq;

use std::env;
use std::process;

use url::Url;
use maman::Spider;
use sidekiq::create_redis_pool;

fn print_usage() {
    println!(maman_version_string!());
    println!("Usage: maman URL [LIMIT]");
}

#[cfg(not(test))]
fn main() {
    env_logger::init().unwrap();
    let url = match env::args().nth(1) {
        Some(url) => {
            match Url::parse(url.as_ref()) {
                Ok(u) => u,
                Err(_) => {
                    print_usage();
                    process::exit(1);
                }
            }
        }
        None => {
            print_usage();
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

    match create_redis_pool() {
        Ok(redis_pool) => {
            let mut spider = Spider::new(redis_pool, url, limit, vec![]);
            spider.crawl()
        },
        Err(err) => {
            error!("Redis error: {}", err);
            process::exit(1);
        }
    }
}
