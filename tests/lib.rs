extern crate url;
#[macro_use]
extern crate maman;

use maman::{Spider, Page};

use std::env;
use std::collections::BTreeMap;

fn visit_page(input: &str) -> Spider {
    env::set_var("MAMAN_ENV", "test");
    let url = "http://example.net/";
    let mut spider = Spider::new(url.to_string(), 0, vec![]);
    let page = Page::new(url.to_string(), input.to_string(), BTreeMap::new(), vec![]);
    let tok = Spider::read_page(page, input);
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
    let url = "http://example.net/".to_string();
    let mut headers = BTreeMap::new();
    headers.insert("content-type".to_string(), "text/html".to_string());
    let page = Page::new(url, input.to_string(), headers.clone(), vec![]);
    let page_serialize = page.serialized();
    let job = page.to_job();
    assert_eq!(job.class, maman_name!());
    assert_eq!(job.retry, 25);
    assert_eq!(job.queue, maman_name!().to_string().to_lowercase());
    assert_eq!(job.args, page_serialize);
}
