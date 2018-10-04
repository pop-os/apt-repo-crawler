extern crate chrono;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate reqwest;
extern crate url_scraper;

use chrono::{DateTime, FixedOffset};
use reqwest::{Client, header::{CONTENT_LENGTH, CONTENT_TYPE, LAST_MODIFIED}, Url};
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
            let headers = head.headers();

            if let Some(content_type) = head.headers().get(CONTENT_TYPE).and_then(|c| c.to_str().ok()) {
                if content_type.starts_with("text/html") {
                    self.scrape(url.as_str(), func)?;
                } else {
                    let length: u64 = headers.get(CONTENT_LENGTH)
                        .and_then(|c| c.to_str().ok())
                        .and_then(|c| c.parse().ok())
                        .unwrap_or(0);
                    
                    let modified = headers.get(LAST_MODIFIED)
                        .and_then(|c| c.to_str().ok())
                        .and_then(|c| DateTime::parse_from_rfc2822(c).ok());

                    if ! func(AptFile { url, length, modified }) { break }
                }
            }
        }

        Ok(())
    }
}

pub struct AptFile {
    pub url: Url,
    pub length: u64,
    pub modified: Option<DateTime<FixedOffset>>
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

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "error while scraping a page: {}", why)]
    Scraper { why: url_scraper::Error },
    #[fail(display = "error while requesting content: {}", why)]
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