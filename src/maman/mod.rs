mod page;
pub use self::page::Page;

use std::collections::BTreeMap;
use std::default::Default;
use std::env;
use std::str::FromStr;
use std::time::Duration;

use html5ever::tokenizer::BufferQueue;
use html5ever::tokenizer::Tokenizer;
use mime;
use reqwest::header::{CONTENT_TYPE, USER_AGENT};
use reqwest::Client as HttpClient;
use reqwest::Response as HttpResponse;
use reqwest::StatusCode;
use robotparser::RobotFileParser;
use sidekiq::Client as SidekiqClient;
use sidekiq::ClientOpts as SidekiqClientOpts;
use sidekiq::RedisPool;
use url::Url;
use url_serde::Serde;

const MAMAN_ENV: &str = "MAMAN_ENV";
const MAMAN_ENV_DEFAULT: &str = "development";

pub struct Spider<'a> {
    pub base_url: Url,
    pub visited_urls: Vec<Serde<Url>>,
    pub unvisited_urls: Vec<Serde<Url>>,
    pub env: String,
    pub limit: isize,
    pub mime_types: Vec<mime::Mime>,
    sidekiq: SidekiqClient,
    robot_parser: RobotFileParser<'a>,
}

impl<'a> Spider<'a> {
    pub fn new(
        redis_pool: RedisPool,
        base_url: Url,
        limit: isize,
        mime_types: Vec<mime::Mime>,
    ) -> Spider<'a> {
        let maman_env =
            env::var(&MAMAN_ENV.to_owned()).unwrap_or_else(|_| MAMAN_ENV_DEFAULT.to_owned());
        let robots_txt = base_url.join("/robots.txt").unwrap();
        let robot_file_parser = RobotFileParser::new(robots_txt);
        let client_opts = SidekiqClientOpts {
            namespace: Some(maman_env.to_string()),
        };
        let sidekiq = SidekiqClient::new(redis_pool, client_opts);
        Spider {
            base_url,
            visited_urls: Vec::new(),
            unvisited_urls: Vec::new(),
            sidekiq,
            env: maman_env,
            robot_parser: robot_file_parser,
            limit,
            mime_types,
        }
    }

    pub fn visit_page(&mut self, page: Page) {
        self.visited_urls.push(page.url.clone());
        for u in &page.urls {
            self.unvisited_urls.push(u.clone());
        }
        if let Err(err) = self.sidekiq.push(page.to_job()) {
            error!("SidekiqClient push failed: {}", err);
        }
    }

    pub fn crawl(&mut self) {
        self.robot_parser.read();
        let base_url = self.base_url.clone();
        if let Some(response) = Spider::load_url(self.base_url.as_ref(), &self.mime_types) {
            self.visit(&base_url, response);
            while let Some(url) = self.unvisited_urls.pop() {
                if self.continue_to_crawl() {
                    if !self.visited_urls.contains(&url) {
                        if let Some(response) = Spider::load_url(url.as_ref(), &self.mime_types) {
                            self.visit(&url, response);
                        }
                    }
                } else {
                    break;
                }
            }
        }
    }

    pub fn read_page(page: Page, document: &str) -> Tokenizer<Page> {
        let mut tok = Tokenizer::new(page, Default::default());
        let mut input = BufferQueue::new();
        input.push_back(String::from(document).into());
        let _ = tok.feed(&mut input);
        tok.end();
        tok
    }

    fn visit(&mut self, page_url: &Url, response: HttpResponse) {
        if self.can_visit(page_url) {
            info!("{}", page_url);
            if let Some(page) = Spider::read_response(page_url, response) {
                self.visit_page(page);
            }
        }
    }

    fn continue_to_crawl(&self) -> bool {
        self.limit == 0 || (self.visited_urls.len() as isize) < self.limit
    }

    fn can_visit(&self, page_url: &Url) -> bool {
        self.robot_parser.can_fetch(maman_name!(), page_url.path())
    }

    fn read_response(page_url: &Url, mut response: HttpResponse) -> Option<Page> {
        let mut headers = BTreeMap::new();
        {
            for (key, value) in response.headers().iter() {
                headers.insert(
                    key.as_str().to_string(),
                    value.to_str().unwrap_or("").to_string(),
                );
            }
        }
        match response.text() {
            Ok(content) => {
                let page = Page::new(
                    page_url.clone(),
                    content.to_string(),
                    headers,
                    response.status().to_string(),
                );
                let read = Spider::read_page(page, &content);
                Some(read.sink)
            }
            _ => None,
        }
    }

    fn load_url(url: &str, mime_types: &[mime::Mime]) -> Option<HttpResponse> {
        let client = HttpClient::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("HttpClient failed to construct");
        match client
            .get(url)
            .header(USER_AGENT, maman_user_agent!())
            .send()
        {
            Err(_) => None,
            Ok(response) => match response.status() {
                StatusCode::OK | StatusCode::NOT_MODIFIED => {
                    if mime_types.is_empty() {
                        Some(response)
                    } else {
                        let content_type = response
                            .headers()
                            .get(CONTENT_TYPE)
                            .and_then(|value| value.to_str().ok())
                            .and_then(|value| value.parse::<mime::Mime>().ok())
                            .and_then(|value|
                                      match (value.type_(), value.subtype()) {
                                          (type_, subtype) => {
                                              let mut text = type_.to_string();
                                              text.push_str("/");
                                              text.push_str(subtype.as_ref());
                                              mime::Mime::from_str(&text).ok()
                                          }
                                      }
                            );
                        match content_type {
                            Some(ct) => {
                                if mime_types.contains(&ct) {
                                    Some(response)
                                } else {
                                    None
                                }
                            }
                            None => None,
                        }
                    }
                }
                _ => None,
            },
        }
    }
}
