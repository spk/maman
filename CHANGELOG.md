
0.13.1 / 2018-12-01
===================

  * Fix mime filter when content type include charset
  * Update env_logger and mockito

0.13.0 / 2018-10-15
===================

  * Use mockito for integration tests
  * Update reqwest to 0.9
  * Drop encoding crate already done in reqwest now
  * Fix mime type filter when charset is present
  * remove `use std::ascii::AsciiExt` for Rust 1.23
  * Update env_logger to 0.5
  * Update html5ever to 0.22
  * Update to log and env_logger 0.4
  * Update sidekiq to 0.8
  * Add filter mime types and update deps
  * Update reqwest to 0.7 and robotparser to 0.9

0.12.1 / 2017-06-18
===================

  * Remove unused hyper_serde crate

0.12.0 / 2017-06-12
===================

  * Fix owned instance just for comparison
  * Update deps serde 1.0
  * Update clippy and add html_root_url
  * Add http status and http version to page
  * Less strict deps
  * Add clippy check

0.11.0 / 2017-03-12
===================

  * Use sidekiq pub Value
  * Update sidekiq client to 0.6
  * Add travis-ci badge to crates.io
  * Update serde to version 0.9

0.10.0 / 2017-01-08
===================

  * Update html5ever to 0.10
  * Update reqwest to 0.2.0 and sidekiq to 0.4.0

0.9.0 / 2016-11-19
==================

  * Use reqwest as http client and upgrade robotparser

0.8.0 / 2016-10-09
==================

  * Remove unused extra vector
  * Readme updates

0.7.0 / 2016-09-11
==================

  * Use makefile for install and add manpage
  * Cleanup main
  * Better error handling for redis pool
  * Use properly env_logger and fix tests
  * Use log and env_logger crate
  * Print sidekiq error to stderr
  * Use rust-url feature serde for serialization
  * Add continue_to_crawl fn
  * Move page to own file

0.6.0 / 2016-09-03
==================

  * Fix robots.txt path from base_url
  * Use encoding crate
  * Update robotparser to 0.5.0

0.5.1 / 2016-08-21
==================

  * Dont follow redirect on crawling
  * Add rustfmt.toml config

0.5.0 / 2016-08-20
==================

  * Fix sidekiq push error display
  * Update url and sidekiq move to serde
  * Add LIMIT option

0.4.0 / 2016-06-07
==================

  * Add urls and extra to Page
  * Move and fix private public functions
  * Use String instead of Url and cleanup
  * Update sidekiq to v0.1.2

0.3.0 / 2016-05-29
==================

  * Only follow StatusCode::Ok and StatusCode::NotModified
  * Move job logic from Page to Job
  * Use rust-sidekiq

0.2.0 / 2016-05-09
==================

  * Set redis per default to 127.0.0.1
  * Use env var for REDIS_URL or default to redis://localhost/
  * Update rust-url to 1.1
  * use robotparser::RobotFileParser

0.1.0 / 2016-05-03
==================

  * Initial release
