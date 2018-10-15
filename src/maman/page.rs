use std::collections::BTreeMap;
use std::default::Default;

use html5ever::tokenizer::{TagToken, Token, TokenSink, TokenSinkResult};
use sidekiq::{Job, JobOpts, Value};
use url::{ParseError, Url};
use url_serde::Serde as UrlSerde;

#[derive(Serialize, Debug)]
pub struct Page {
    pub url: UrlSerde<Url>,
    pub document: String,
    pub headers: BTreeMap<String, String>,
    pub status: String,
    pub urls: Vec<UrlSerde<Url>>,
}

impl TokenSink for Page {
    type Handle = ();

    fn process_token(&mut self, token: Token, _: u64) -> TokenSinkResult<()> {
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
    pub fn new(
        url: Url,
        document: String,
        headers: BTreeMap<String, String>,
        status: String,
    ) -> Self {
        Page {
            url: UrlSerde(url),
            document,
            headers,
            status,
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
            Some(u) => match u.scheme() {
                "http" | "https" => {
                    if !self.url_eq(&u) && self.domain_eq(&u) {
                        Some(u)
                    } else {
                        None
                    }
                }
                _ => None,
            },
            None => None,
        }
    }
}
