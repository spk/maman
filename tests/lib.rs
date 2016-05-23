extern crate url;
#[macro_use]
extern crate maman;

use maman::{Spider, Page};

use std::env;
use std::collections::BTreeMap;
use url::Url;

fn visit_page(input: &str) -> Spider {
    env::set_var("MAMAN_ENV", "test");
    let url = "http://example.net/";
    let mut spider = Spider::new(url.to_string());
    let page = Page::new(Url::parse(url).unwrap(), input.to_string(), BTreeMap::new());
    let tok = spider.read_page(page, input);
    spider.visit_page(tok.unwrap());
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
    let page = Page::new(url.clone(), input.to_string(), headers.clone());
    let page_serialize = page.serialized();
    let job = page.to_job();
    assert_eq!(job.class, maman_name!());
    assert_eq!(job.retry, 25);
    assert_eq!(job.queue, maman_name!().to_string().to_lowercase());
    assert_eq!(job.args, page_serialize);
}
