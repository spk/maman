use std::default::Default;
use std::collections::BTreeMap;

use url::{Url, ParseError};
use sidekiq::{Job, JobOpts, Value};
use url_serde::Serde as UrlSerde;
use html5ever::tokenizer::{TokenSink, Token, TagToken, TokenSinkResult};

#[derive(Serialize, Debug)]
pub struct Page {
    pub url: UrlSerde<Url>,
    pub document: String,
    pub headers: BTreeMap<String, String>,
    pub status: String,
    pub http_version: String,
    pub urls: Vec<UrlSerde<Url>>,
}

impl TokenSink for Page {
    type Handle = ();

    #[cfg_attr(feature = "clippy", allow(single_match))]
    fn process_token(&mut self, token: Token) -> TokenSinkResult<()> {
        if let TagToken(tag) = token {
            match tag.name {
                local_name!("a") => {
                    for attr in &tag.attrs {
                        if &*attr.name.local == "href" {
                            if let Some(u) = self.can_enqueue(&attr.value) {
                                self.urls.push(UrlSerde(u));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        TokenSinkResult::Continue
    }
}

impl Page {
    pub fn new(url: Url,
               document: String,
               headers: BTreeMap<String, String>,
               status: String,
               http_version: String)
               -> Self {
        Page {
            url: UrlSerde(url),
            document: document,
            headers: headers,
            status: status,
            http_version: http_version,
            urls: Vec::new(),
        }
    }

    pub fn to_job(&self) -> Job {
        let job_opts = JobOpts {
            queue: maman_name!().to_string().to_lowercase(),
            ..Default::default()
        };
        Job::new(maman_name!().to_string(), vec![self.as_object()], job_opts)
    }

    pub fn as_object(&self) -> Value {
        json!({
            "url": &self.url,
            "document": &self.document,
            "headers": &self.headers,
            "status": &self.status,
            "http_version": &self.http_version,
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
