
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
