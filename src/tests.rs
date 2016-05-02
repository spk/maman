use std::env;
use std::collections::BTreeMap;
use url::Url;
use super::{Spider, Page};

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
fn test_redis_queue_name() {
    env::set_var("MAMAN_ENV", "test");
    let spider = Spider::new("http://example.net/".to_string());
    assert_eq!(spider.redis_queue_name, "test:queue:maman");
}

#[test]
fn test_json_job_format() {
    use time::now_utc;
    use rustc_serialize::json::{ToJson, Json};

    env::set_var("MAMAN_ENV", "test");
    let input = "<html><body><a href='/todo#new' /><a href='/new' /></html>";
    let url = Url::parse("http://example.net/").unwrap();
    let mut headers = BTreeMap::new();
    headers.insert("content-type".to_string(), "text/html".to_string());
    let page = Page::new(url.clone(), input.to_string(), headers.clone());

    let object = {
        let mut root = BTreeMap::new();
        let mut object = BTreeMap::new();
        let mut args = Vec::new();
        object.insert("url".to_string(), url.to_string().to_json());
        object.insert("document".to_string(), input.to_json());
        object.insert("headers".to_string(), headers.to_json());
        args.push(object);
        root.insert("class".to_string(), "Maman".to_json());
        root.insert("retry".to_string(), true.to_json());
        root.insert("args".to_string(), args.to_json());
        root.insert("jid".to_string(), page.jid.to_json());
        root.insert("created_at".to_string(),
                    now_utc().to_timespec().sec.to_json());
        root.insert("enqueued_at".to_string(),
                    now_utc().to_timespec().sec.to_json());
        Json::Object(root)
    };
    assert_eq!(page.to_json(), object);
}
