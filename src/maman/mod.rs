mod page;
pub use self::page::Page;

use std::env;
use std::io::Read;
use std::ascii::AsciiExt;
use std::default::Default;
use std::collections::BTreeMap;

use url::Url;
use tendril::SliceExt;
use hyper::header::UserAgent;
use hyper::Client as HyperClient;
use hyper::client::RedirectPolicy;
use hyper::client::Response as HttpResponse;
use hyper::status::StatusCode;
use robotparser::RobotFileParser;
use html5ever::tokenizer::Tokenizer;
use sidekiq::Client as SidekiqClient;
use sidekiq::ClientOpts as SidekiqClientOpts;
use sidekiq::RedisPool;
use encoding::{Encoding, DecoderTrap};
use encoding::all::UTF_8;

const MAMAN_ENV: &'static str = "MAMAN_ENV";

pub struct Spider<'a> {
    pub base_url: Url,
    pub visited_urls: Vec<Url>,
    pub unvisited_urls: Vec<Url>,
    pub extra: Vec<String>,
    pub env: String,
    pub limit: isize,
    sidekiq: SidekiqClient,
    robot_parser: RobotFileParser<'a>,
}

impl<'a> Spider<'a> {
    pub fn new(redis_pool: RedisPool,
               base_url: Url,
               limit: isize,
               extra: Vec<String>)
               -> Spider<'a> {
        let maman_env = env::var(&MAMAN_ENV.to_string()).unwrap_or("development".to_string());
        let robots_txt = base_url.join("/robots.txt").unwrap();
        let robot_file_parser = RobotFileParser::new(robots_txt);
        let client_opts =
            SidekiqClientOpts { namespace: Some(maman_env.to_string()), ..Default::default() };
        let sidekiq = SidekiqClient::new(redis_pool, client_opts);
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
                error!("SidekiqClient push failed: {}", err);
            }
            Ok(_) => {}
        }
    }

    pub fn crawl(&mut self) {
        self.robot_parser.read();
        let base_url = self.base_url.clone();
        if let Some(response) = Spider::load_url(self.base_url.as_ref()) {
            self.visit(&base_url, response);
            while let Some(url) = self.unvisited_urls.pop() {
                if self.continue_to_crawl() {
                    if !self.visited_urls.contains(&url) {
                        if let Some(response) = Spider::load_url(url.as_ref()) {
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
        tok.feed(document.to_tendril());
        tok.end();
        tok
    }

    fn visit(&mut self, page_url: &Url, response: HttpResponse) {
        if self.can_visit(page_url) {
            info!("{}", page_url);
            if let Some(page) = Spider::read_response(page_url, response, self.extra.clone()) {
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

    fn read_response(page_url: &Url,
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
                let page = Page::new(page_url.clone(), doc.to_string(), headers.clone(), extra);
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
