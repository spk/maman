use std::env;
use std::io::Read;
use std::error::Error;
use std::ascii::AsciiExt;
use std::default::Default;
use std::collections::BTreeMap;

use rand::{Rng, thread_rng};
use time::now_utc;
use tendril::SliceExt;
use url::{Url, ParseError};
use hyper::header::UserAgent;
use hyper::Client as HyperClient;
use hyper::client::Response as HttpResponse;
use robotparser::RobotFileParser;
use redis::Client as RedisClient;
use redis::{Commands, RedisResult};
use rustc_serialize::json::{ToJson, Json};
use html5ever::tokenizer::{TokenSink, Token, TagToken, Tokenizer};

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
    pub visited_urls: Vec<Url>,
    pub unvisited_urls: Vec<Url>,
    pub env: String,
    pub redis_queue_name: String,
    robot_parser: RobotFileParser<'a>,
}

pub struct Page {
    pub url: Url,
    pub document: String,
    pub headers: BTreeMap<String, String>,
    pub urls: Vec<Url>,
    pub jid: String,
}

impl ToJson for Page {
    fn to_json(&self) -> Json {
        let mut root = BTreeMap::new();
        let mut object = BTreeMap::new();
        let mut args = Vec::new();
        let now = now_utc().to_timespec().sec;
        object.insert("url".to_string(), self.url.to_string().to_json());
        object.insert("document".to_string(), self.document.to_json());
        object.insert("headers".to_string(), self.headers.to_json());
        args.push(object);
        root.insert("class".to_string(), maman_name!().to_json());
        root.insert("retry".to_string(), true.to_json());
        root.insert("args".to_string(), args.to_json());
        root.insert("jid".to_string(), self.jid.to_json());
        root.insert("created_at".to_string(), now.to_json());
        root.insert("enqueued_at".to_string(), now.to_json());
        Json::Object(root)
    }
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
                                        self.urls.push(u);
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
    pub fn new(url: Url, document: String, headers: BTreeMap<String, String>) -> Page {
        let jid = thread_rng().gen_ascii_chars().take(24).collect::<String>();
        Page {
            url: url,
            document: document,
            headers: headers,
            urls: Vec::new(),
            jid: jid,
        }
    }

    fn parsed_url(&self, url: &str) -> Option<Url> {
        match Url::parse(url) {
            Ok(u) => Some(u),
            Err(ParseError::RelativeUrlWithoutBase) => Some(self.url.join(url).unwrap()),
            Err(_) => None,
        }
    }

    fn parsed_url_without_fragment(&self, url: &str) -> Option<Url> {
        match self.parsed_url(url) {
            Some(mut u) => {
                u.set_fragment(None);
                Some(u)
            }
            None => None,
        }
    }

    fn url_eq(&self, url: &Url) -> bool {
        self.url == *url
    }

    fn domain_eq(&self, url: &Url) -> bool {
        self.url.domain() == url.domain()
    }

    fn can_enqueue(&self, url: &str) -> Option<Url> {
        match self.parsed_url_without_fragment(url) {
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
    pub fn new(base_url: String) -> Spider<'a> {
        let maman_env = env::var(&MAMAN_ENV.to_string()).unwrap_or("development".to_string());
        let redis_queue_name = format!("{}:{}:{}", maman_env, "queue", "maman");
        let robots_txt = format!("{}/{}", base_url, "robots.txt");
        let robot_parser = RobotFileParser::new(robots_txt);
        Spider {
            base_url: base_url,
            visited_urls: Vec::new(),
            unvisited_urls: Vec::new(),
            env: maman_env,
            redis_queue_name: redis_queue_name,
            robot_parser: robot_parser,
        }
    }

    pub fn is_visited(&self, url: &Url) -> bool {
        self.visited_urls.contains(url)
    }

    pub fn visited_urls(&self) -> &Vec<Url> {
        &self.visited_urls
    }

    pub fn read_response(&self, page_url: Url, mut response: HttpResponse) -> Option<Page> {
        let mut headers = BTreeMap::new();
        {
            for h in response.headers.iter() {
                headers.insert(h.name().to_ascii_lowercase(), h.value_string());
            }
        }
        let mut document = String::new();
        // handle CharsError::NotUtf8
        match response.read_to_string(&mut document) {
            Ok(_) => {
                let page = Page::new(page_url, document.to_string(), headers.clone());
                let read = self.read_page(page, &document).unwrap();
                Some(read)
            }
            Err(_) => None,
        }
    }

    pub fn read_page(&self, page: Page, document: &str) -> Tokenizer<Page> {
        let mut tok = Tokenizer::new(page, Default::default());
        tok.feed(document.to_tendril());
        tok.end();
        tok
    }

    pub fn visit_page(&mut self, page: Page) {
        self.add_visited_url(page.url.clone());
        for u in page.urls.iter() {
            self.add_unvisited_url(u.clone());
        }
        match self.send_to_redis(page) {
            Err(err) => {
                println!("Redis {}: {}", err.category(), err.description());
            }
            Ok(()) => {}
        }
    }

    fn can_visit(&self, page_url: Url) -> bool {
        self.robot_parser.can_fetch(maman_name!(), page_url.path())
    }

    pub fn visit(&mut self, page_url: &str, response: HttpResponse) {
        match Url::parse(page_url) {
            Ok(u) => {
                if self.can_visit(u.clone()) {
                    if let Some(page) = self.read_response(u, response) {
                        self.visit_page(page);
                    }
                }
            },
            Err(_) => {},
        }
    }

    pub fn crawl(&mut self) {
        self.robot_parser.read();
        let base_url = self.base_url.clone();
        if let Some(response) = self.load_url(&base_url) {
            self.visit(&base_url, response);
            while let Some(url) = self.unvisited_urls.pop() {
                if !self.is_visited(&url) {
                    let url_ser = &url.to_string();
                    if let Some(response) = self.load_url(url_ser) {
                        self.visit(url_ser, response);
                    }
                }
            }
        }
    }

    fn send_to_redis(&self, page: Page) -> RedisResult<()> {
        let client = try!(RedisClient::open("redis://127.0.0.1/"));
        let connection = try!(client.get_connection());
        let _: () = try!(connection.lpush(self.redis_queue_name.to_string(), page.to_json()));

        Ok(())
    }

    fn load_url(&self, url: &str) -> Option<HttpResponse> {
        let client = HyperClient::new();
        let request = client.get(url).header(UserAgent(maman_user_agent!().to_owned()));
        match request.send() {
            Ok(response) => Some(response),
            Err(_) => None,
        }
    }

    fn add_visited_url(&mut self, url: Url) {
        self.visited_urls.push(url);
    }

    fn add_unvisited_url(&mut self, url: Url) {
        self.unvisited_urls.push(url);
    }
}
