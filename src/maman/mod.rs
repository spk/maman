use std::env;
use std::io::Read;
use std::ascii::AsciiExt;
use std::default::Default;
use std::collections::BTreeMap;

use tendril::SliceExt;
use url::{Url, ParseError};
use hyper::header::UserAgent;
use hyper::Client as HyperClient;
use hyper::client::RedirectPolicy;
use hyper::client::Response as HttpResponse;
use hyper::status::StatusCode;
use robotparser::RobotFileParser;
use html5ever::tokenizer::{TokenSink, Token, TagToken, Tokenizer};
use sidekiq::Client as SidekiqClient;
use sidekiq::ClientOpts as SidekiqClientOpts;
use sidekiq::{Job, JobOpts, create_redis_pool};
use serde_json::value::Value;
use serde_json::builder::ObjectBuilder;
use encoding::{Encoding, DecoderTrap};
use encoding::all::UTF_8;

#[macro_export]
macro_rules! maman_name {
    () => ( "Maman" )
}
#[macro_export]
macro_rules! maman_version {
    () => ( env!("CARGO_PKG_VERSION") )
}
#[macro_export]
macro_rules! maman_version_string {
    () => ( concat!(maman_name!(), " v", maman_version!()) )
}
#[macro_export]
macro_rules! maman_user_agent {
    () => ( concat!(maman_version_string!(), " (https://crates.io/crates/maman)") )
}

const MAMAN_ENV: &'static str = "MAMAN_ENV";

pub struct Spider<'a> {
    pub base_url: String,
    pub visited_urls: Vec<String>,
    pub unvisited_urls: Vec<String>,
    pub extra: Vec<String>,
    pub env: String,
    pub limit: isize,
    sidekiq: SidekiqClient,
    robot_parser: RobotFileParser<'a>,
}

pub struct Page {
    pub url: String,
    pub document: String,
    pub headers: BTreeMap<String, String>,
    pub urls: Vec<String>,
    pub extra: Vec<String>,
}

impl TokenSink for Page {
    fn process_token(&mut self, token: Token) {
        match token {
            TagToken(tag) => {
                match tag.name {
                    atom!("a") => {
                        for attr in tag.attrs.iter() {
                            if attr.name.local.to_string() == "href" {
                                match self.can_enqueue(&attr.value) {
                                    Some(u) => {
                                        self.urls.push(u.to_string());
                                    }
                                    None => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl Page {
    pub fn new(url: String,
               document: String,
               headers: BTreeMap<String, String>,
               extra: Vec<String>)
               -> Self {
        Page {
            url: url,
            document: document,
            headers: headers,
            urls: Vec::new(),
            extra: extra,
        }
    }

    pub fn to_job(&self) -> Job {
        let job_opts =
            JobOpts { queue: maman_name!().to_string().to_lowercase(), ..Default::default() };
        Job::new(maman_name!().to_string(), vec![self.as_object()], job_opts)
    }

    pub fn as_object(&self) -> Value {
        ObjectBuilder::new()
            .insert("url".to_string(), &self.url)
            .insert("document".to_string(), &self.document)
            .insert("headers".to_string(), &self.headers)
            .insert("urls".to_string(), &self.urls)
            .insert("extra".to_string(), &self.extra)
            .build()
    }

    fn normalize_url(&self, url: &str) -> Option<Url> {
        match Url::parse(url) {
            Ok(u) => Some(u),
            Err(ParseError::RelativeUrlWithoutBase) => Some(self.parsed_url().join(url).unwrap()),
            Err(_) => None,
        }
    }

    fn url_without_fragment(&self, url: &str) -> Option<Url> {
        match self.normalize_url(url) {
            Some(mut u) => {
                u.set_fragment(None);
                Some(u)
            }
            None => None,
        }
    }

    fn parsed_url(&self) -> Url {
        Url::parse(&self.url).unwrap()
    }

    fn url_eq(&self, url: &Url) -> bool {
        self.parsed_url() == *url
    }

    fn domain_eq(&self, url: &Url) -> bool {
        self.parsed_url().domain() == url.domain()
    }

    fn can_enqueue(&self, url: &str) -> Option<Url> {
        match self.url_without_fragment(url) {
            Some(u) => {
                match u.scheme() {
                    "http" | "https" => {
                        if !self.url_eq(&u) && self.domain_eq(&u) {
                            Some(u)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            None => None,
        }
    }
}

impl<'a> Spider<'a> {
    pub fn new(base_url: String, limit: isize, extra: Vec<String>) -> Spider<'a> {
        let maman_env = env::var(&MAMAN_ENV.to_string()).unwrap_or("development".to_string());
        let robots_txt = Url::parse(&base_url).unwrap().join("/robots.txt").unwrap();
        let robot_file_parser = RobotFileParser::new(robots_txt);
        let client_opts =
            SidekiqClientOpts { namespace: Some(maman_env.to_string()), ..Default::default() };
        let sidekiq = SidekiqClient::new(create_redis_pool(), client_opts);
        Spider {
            base_url: base_url,
            visited_urls: Vec::new(),
            unvisited_urls: Vec::new(),
            extra: extra,
            sidekiq: sidekiq,
            env: maman_env,
            robot_parser: robot_file_parser,
            limit: limit,
        }
    }

    pub fn visit_page(&mut self, page: Page) {
        self.visited_urls.push(page.url.clone());
        for u in page.urls.iter() {
            self.unvisited_urls.push(u.clone());
        }
        match self.sidekiq.push(page.to_job()) {
            Err(err) => {
                println!("SidekiqClient push failed: {}", err);
            }
            Ok(_) => {}
        }
    }

    pub fn crawl(&mut self) {
        self.robot_parser.read();
        let base_url = self.base_url.clone();
        if let Some(response) = Spider::load_url(&base_url) {
            self.visit(&base_url, response);
            while let Some(url) = self.unvisited_urls.pop() {
                if self.limit == 0 || (self.visited_urls.len() as isize) < self.limit {
                    if !self.visited_urls.contains(&url) {
                        let url_ser = &url.to_string();
                        if let Some(response) = Spider::load_url(url_ser) {
                            self.visit(url_ser, response);
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
        tok.feed(document.to_tendril());
        tok.end();
        tok
    }

    fn visit(&mut self, page_url: &str, response: HttpResponse) {
        match Url::parse(page_url) {
            Ok(u) => {
                if self.can_visit(u.clone()) {
                    if let Some(page) = Spider::read_response(u.to_string(),
                                                              response,
                                                              self.extra.clone()) {
                        self.visit_page(page);
                    }
                }
            }
            Err(_) => {}
        }
    }

    fn can_visit(&self, page_url: Url) -> bool {
        self.robot_parser.can_fetch(maman_name!(), page_url.path())
    }

    fn read_response(page_url: String,
                     mut response: HttpResponse,
                     extra: Vec<String>)
                     -> Option<Page> {
        let mut headers = BTreeMap::new();
        {
            for h in response.headers.iter() {
                headers.insert(h.name().to_ascii_lowercase(), h.value_string());
            }
        }
        let mut document = vec![];
        let _ = response.read_to_end(&mut document);
        match UTF_8.decode(&document, DecoderTrap::Ignore) {
            Ok(doc) => {
                let page = Page::new(page_url, doc.to_string(), headers.clone(), extra);
                let read = Spider::read_page(page, &doc).unwrap();
                Some(read)
            }
            Err(_) => None,
        }
    }

    fn load_url(url: &str) -> Option<HttpResponse> {
        let mut client = HyperClient::new();
        client.set_redirect_policy(RedirectPolicy::FollowNone);
        let request = client.get(url).header(UserAgent(maman_user_agent!().to_owned()));
        match request.send() {
            Ok(response) => {
                match response.status {
                    StatusCode::Ok | StatusCode::NotModified => Some(response),
                    _ => None,
                }
            }
            Err(_) => None,
        }
    }
}
