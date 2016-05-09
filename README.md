# Maman

Maman is a Rust Web Crawler saving pages on Redis.

Pages are send to list `<MAMAN_ENV>:queue:maman` using
[Sidekiq](https://github.com/mperham/sidekiq/wiki/Job-Format) job format:

``` json
{
"class": "Maman",
"jid": "b4a577edbccf1d805744efa9",
"retry": true,
"created_at": 1461789979, "enqueued_at": 1461789979,
"args": {
    "document":"<html><body><a href='#' /><a href='/new' /></html>",
    "headers": {"content-type": "text/html"},
    "url": "http://example.net/"
    }
}
```

## Dependencies

* Redis

## Installation

~~~
cargo install maman
~~~

## Usage

~~~
REDIS_URL="redis://127.0.0.1/" maman URL
~~~

## LICENSE

The MIT License

Copyright (c) 2016 Laurent Arnoud <laurent@spkdev.net>

---
[![Build](https://img.shields.io/travis-ci/spk/maman.svg)](https://travis-ci.org/spk/maman)
[![Version](https://img.shields.io/crates/v/maman.svg)](https://crates.io/crates/maman)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](http://opensource.org/licenses/MIT "MIT")
[![Project status](http://img.shields.io/status/experimental.png?color=red)](https://github.com/spk/maman)
