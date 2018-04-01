# Maman

Maman is a Rust Web Crawler saving pages on Redis.

Pages are send to list `<MAMAN_ENV>:queue:maman` using
[Sidekiq job format](https://github.com/mperham/sidekiq/wiki/Job-Format)

``` json
{
"class": "Maman",
"jid": "b4a577edbccf1d805744efa9",
"retry": true,
"created_at": 1461789979, "enqueued_at": 1461789979,
"args": {
    "document":"<html><body><a href='#' /><a href='/new' /></html>",
    "urls": ["https://example.net/new"],
    "headers": {"content-type": "text/html"},
    "url": "https://example.net/"
    }
}
```

## Dependencies

* [Redis](http://redis.io/)

## Installation

### With cargo

~~~
cargo install maman
~~~

### With make

~~~
PREFIX=~/.local make install
~~~

## Usage

~~~
maman URL [LIMIT] [MIME_TYPES]
~~~

`LIMIT` must be an integer or `0` is the default, meaning no limit.

## Environment variables

### Defaults

* MAMAN_ENV=development
* REDIS_URL="redis://127.0.0.1/"

### Others

* RUST_LOG=maman=info

## LICENSE

The MIT License

Copyright (c) 2016-2018 Laurent Arnoud <laurent@spkdev.net>

---
[![Build](https://img.shields.io/travis-ci/spk/maman/master.svg)](https://travis-ci.org/spk/maman)
[![Version](https://img.shields.io/crates/v/maman.svg)](https://crates.io/crates/maman)
[![Documentation](https://img.shields.io/badge/doc-rustdoc-blue.svg)](https://docs.rs/maman/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT "MIT")
[![Project status](https://img.shields.io/status/experimental.png?color=red)](https://github.com/spk/maman)
[![Dependency status](https://deps.rs/repo/github/spk/maman/status.svg)](https://deps.rs/repo/github/spk/maman)
