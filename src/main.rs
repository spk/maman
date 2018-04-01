extern crate env_logger;
#[macro_use]
extern crate log;
extern crate mime;
extern crate url;

#[macro_use]
extern crate maman;
extern crate sidekiq;

use std::env;
use std::process;
use std::str::FromStr;

use url::Url;
use maman::Spider;
use sidekiq::create_redis_pool;

const DEFAULT_LIMIT: isize = 0;

fn print_usage() {
    println!(maman_version_string!());
    println!("Usage: maman URL [LIMIT] [MIME_TYPES]");
}

fn fetch_url(url_arg: Option<String>) -> Url {
    match url_arg {
        Some(url) => match Url::parse(url.as_ref()) {
            Ok(u) => return u,
            Err(_) => {
                print_usage();
                process::exit(1);
            }
        },
        None => {
            print_usage();
            process::exit(1);
        }
    }
}

fn fetch_limit(limit_arg: Option<String>) -> isize {
    match limit_arg {
        Some(limit) => match limit.parse::<isize>() {
            Err(_) => DEFAULT_LIMIT,
            Ok(l) => l,
        },
        None => DEFAULT_LIMIT,
    }
}

fn fetch_mime_types(mime_types_arg: Option<String>) -> Vec<mime::Mime> {
    let mut mime_types = Vec::new();
    match mime_types_arg {
        Some(mts) => {
            let v: Vec<&str> = mts.split(" ").collect();
            for m in v {
                match mime::Mime::from_str(&m.as_ref()) {
                    Ok(mime) => {
                        mime_types.push(mime);
                    }
                    Err(_) => {}
                }
            }
            mime_types
        }
        None => mime_types,
    }
}

fn main() {
    env_logger::init();

    match create_redis_pool() {
        Ok(redis_pool) => {
            let mut spider = Spider::new(
                redis_pool,
                fetch_url(env::args().nth(1)),
                fetch_limit(env::args().nth(2)),
                fetch_mime_types(env::args().nth(3)),
            );
            spider.crawl()
        }
        Err(err) => {
            error!("Redis error: {}", err);
            process::exit(1);
        }
    }
}
