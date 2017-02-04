use std::default::Default;
use std::collections::BTreeMap;

use url::{Url, ParseError};
use sidekiq::{Job, JobOpts};
use serde_json::value::Value;
use url_serde::Serde;
use html5ever::tokenizer::{TokenSink, Token, TagToken, TokenSinkResult};

#[derive(Serialize, Debug)]
pub struct Page {
    pub url: Serde<Url>,
    pub document: String,
    pub headers: BTreeMap<String, String>,
    pub urls: Vec<Serde<Url>>,
}

impl TokenSink for Page {
    type Handle = ();

    fn process_token(&mut self, token: Token) -> TokenSinkResult<()> {
        match token {
            TagToken(tag) => {
                match tag.name {
                    local_name!("a") => {
                        for attr in tag.attrs.iter() {
                            if attr.name.local.to_string() == "href" {
                                match self.can_enqueue(&attr.value) {
                                    Some(u) => {
                                        self.urls.push(Serde(u));
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
        TokenSinkResult::Continue
    }
}

impl Page {
    pub fn new(url: Url, document: String, headers: BTreeMap<String, String>) -> Self {
        Page {
            url: Serde(url),
            document: document,
            headers: headers,
            urls: Vec::new(),
        }
    }

    pub fn to_job(&self) -> Job {
        let job_opts =
            JobOpts { queue: maman_name!().to_string().to_lowercase(), ..Default::default() };
        Job::new(maman_name!().to_string(), vec![self.as_object()], job_opts)
    }

    pub fn as_object(&self) -> Value {
        json!({
            "url": &self.url,
            "document": &self.document,
            "headers": &self.headers,
            "urls": &self.urls,
        })
    }

    fn normalize_url(&self, url: &str) -> Option<Url> {
        match Url::parse(url) {
            Ok(u) => Some(u),
            Err(ParseError::RelativeUrlWithoutBase) => Some(self.url.join(url).unwrap()),
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

    fn url_eq(&self, url: &Url) -> bool {
        self.url == *url
    }

    fn domain_eq(&self, url: &Url) -> bool {
        self.url.domain() == url.domain()
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
