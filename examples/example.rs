extern crate apt_repo_crawler;

use apt_repo_crawler::Crawler;
use std::sync::mpsc::channel;
use std::thread;

pub fn main() {
    let (tx, rx) = channel();

    thread::spawn(move || {
        Crawler::new().scrape(
            "http://apt.pop-os.org/proprietary/pool/bionic/",
            &mut |url| tx.send(url).is_ok()
        )
    });

    for path in rx{
        if let Some(desc) = path.get_details() {
            println!("URL: {}", path.url);
            println!("Length: {}, Last Modified: {:?}", path.length, path.modified);
            println!("name: {:#?}", desc);
        }
    }
}