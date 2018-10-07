extern crate apt_repo_crawler;

use apt_repo_crawler::*;
use std::sync::Arc;

pub struct Filter;

impl AptPackageFilter for Filter {
    fn validate(&self, package: AptPackage) -> bool {
        package.extension == "deb"
    }
}

pub fn main() {
    let crawler = AptCrawler::new("http://apt.pop-os.org/".into())
        .filter(Arc::new(Filter));

    for file in crawler.crawl() {
        println!("{:#?}", file);
    }
}
