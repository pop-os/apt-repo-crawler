extern crate reqwest;
extern crate url_scraper;

use reqwest::{Client, header::CONTENT_TYPE, Url};
use url_scraper::UrlScraper;

pub struct Crawler {
    client: Client
}

impl Crawler {
    pub fn new() -> Self {
        Self {
            client: Client::new()
        }
    }

    pub fn scrape(&self, s_url: &str, func: &mut impl FnMut(AptFile) -> bool) -> Result<(), Error> {
        for (_, url) in UrlScraper::new(s_url)?.into_iter() {
            // Never look back
            if url.as_str().len() < s_url.len() { continue }

            // Check the content type to determine if we should follow the link, or that we've found a file.
            let head = self.client.head(url.clone()).send()?;
            if let Some(content_type) = head.headers().get(CONTENT_TYPE).and_then(|c| c.to_str().ok()) {
                if content_type.starts_with("text/html") {
                    self.scrape(url.as_str(), func)?;
                } else {
                    if ! func(AptFile { url }) { break }
                }
            }
        }

        Ok(())
    }
}

pub struct AptFile {
    pub url: Url,
}

impl AptFile {
    pub fn get_file_name(&self) -> &str {
        let url = self.url.as_str();
        let pos = url.rfind('/').unwrap_or(0);
        &url[pos+1..]
    }

    pub fn get_details(&self) -> Option<AptFileDetails> {
        let mut file_name = self.get_file_name();

        let mut pos = file_name.find('_')?;
        let name = &file_name[..pos];
        file_name = &file_name[pos+1..];

        pos = file_name.find('_')?;
        let version = &file_name[..pos];
        file_name = &file_name[pos+1..];

        pos = file_name.find(".d").or_else(|| file_name.find(".t"))?;
        let arch = &file_name[..pos];
        let extension = &file_name[pos+1..];

        Some(AptFileDetails {
            name,
            version,
            arch,
            extension
        })
    }
}

#[derive(Debug)]
pub struct AptFileDetails<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub arch: &'a str,
    pub extension: &'a str

}

pub enum Error {
    Scraper { why: url_scraper::Error },
    Request { why: reqwest::Error }
}

impl From<url_scraper::Error> for Error {
    fn from(why: url_scraper::Error) -> Error {
        Error::Scraper { why }
    }
}

impl From<reqwest::Error> for Error {
    fn from(why: reqwest::Error) -> Error {
        Error::Request { why }
    }
}