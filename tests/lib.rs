extern crate mockito;
extern crate sidekiq;
extern crate url;
#[macro_use]
extern crate maman;

use maman::{Page, Spider};
use sidekiq::create_redis_pool;

use std::collections::BTreeMap;
use std::env;
use std::str::FromStr;

use url::Url;

fn visit_page(input: &str) -> Spider {
    env::set_var("MAMAN_ENV", "test");
    let url = Url::parse("https://example.net/").unwrap();
    let redis_pool = create_redis_pool().unwrap();
    let mut spider = Spider::new(redis_pool, url.clone(), 0, Vec::new());
    let page = Page::new(
        url,
        input.to_string(),
        BTreeMap::new(),
        "200 OK".to_string(),
    );
    let tok = Spider::read_page(page, input);
    spider.visit_page(tok.sink);
    spider
}

#[test]
fn test_ignore_initial_url_link() {
    let input = "<html><body><a href='/' /><a href='/new' /></html>";
    let spider = visit_page(input);
    assert_eq!(spider.visited_urls.len(), 1);
    assert_eq!(spider.unvisited_urls.len(), 1);
}

#[test]
fn test_ignore_fragment_link() {
    let input = "<html><body><a href='#' /><a href='/new' /></html>";
    let spider = visit_page(input);
    assert_eq!(spider.visited_urls.len(), 1);
    assert_eq!(spider.unvisited_urls.len(), 1);
}

#[test]
fn test_ignore_mailto_link() {
    let input = "<html><body><a href='mailto:example@example.net' /><a href='/new' /></html>";
    let spider = visit_page(input);
    assert_eq!(spider.visited_urls.len(), 1);
    assert_eq!(spider.unvisited_urls.len(), 1);
}

#[test]
fn test_new_with_fragment_link() {
    let input = "<html><body><a href='/todo#new' /><a href='/new' /></html>";
    let spider = visit_page(input);
    assert_eq!(spider.visited_urls.len(), 1);
    assert_eq!(spider.unvisited_urls.len(), 2);
}

#[test]
fn test_other_domain_link() {
    let input = "<html><body><a href='https://github.com/' /></html>";
    let spider = visit_page(input);
    assert_eq!(spider.visited_urls.len(), 1);
    assert_eq!(spider.unvisited_urls.len(), 0);
}

#[test]
fn test_json_job_format() {
    env::set_var("MAMAN_ENV", "test");
    let input = "<html><body><a href='/todo#new' /><a href='/new' /></html>";
    let url = Url::parse("http://example.net/").unwrap();
    let mut headers = BTreeMap::new();
    headers.insert("content-type".to_string(), "text/html".to_string());
    let page = Page::new(
        url,
        input.to_string(),
        headers.clone(),
        "200 OK".to_string(),
    );
    let page_object = page.as_object();
    let job = page.to_job();
    assert_eq!(job.class, maman_name!());
    assert_eq!(job.retry, 25);
    assert_eq!(job.queue, maman_name!().to_string().to_lowercase());
    assert_eq!(job.args, vec![page_object]);
}

#[test]
fn test_integration() {
    use mockito::mock;
    let _r = mock("GET", "/robots.txt")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("User-agent: *\nAllow: /")
        .create();
    let _m1 = mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body("<html><a href='/hello'>hello</a>")
        .create();
    let _m2 = mock("GET", "/hello")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body("<html><a href='/world'>world</a></html>")
        .create();
    let _m3 = mock("GET", "/world")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body("<html>!</html>")
        .create();
    let redis_pool = create_redis_pool().unwrap();
    let url = Url::parse(mockito::SERVER_URL).unwrap();
    let mut spider = Spider::new(redis_pool, url, 0, Vec::new());
    spider.crawl();
    assert_eq!(spider.visited_urls.len(), 3);
}

#[test]
fn test_integration_filter() {
    use mockito::mock;
    let _r = mock("GET", "/robots.txt")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("User-agent: *\nAllow: /")
        .create();
    let _m1 = mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html; charset=utf-8")
        .with_body("<html><a href='/hello'>hello</a>")
        .create();
    let _m2 = mock("GET", "/hello")
        .with_status(200)
        .with_header("content-type", "text/html; charset=utf-8")
        .with_body("<html><a href='/world'>world</a></html>")
        .create();
    let _m3 = mock("GET", "/world")
        .with_status(200)
        .with_header("content-type", "text/html; charset=utf-8")
        .with_body("<html>!</html>")
        .create();
    let redis_pool = create_redis_pool().unwrap();
    let url = Url::parse(mockito::SERVER_URL).unwrap();
    let mut spider = Spider::new(redis_pool, url, 0, vec![mime::Mime::from_str("text/html").unwrap()]);
    spider.crawl();
    assert_eq!(spider.visited_urls.len(), 3);
}
