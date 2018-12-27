// #![deny(warnings)]

use std::mem;
use std::io::{self, Cursor};
use futures::{Future, Stream};
use reqwest::r#async::{Client, Decoder};
use tokio::fs::OpenOptions;

fn crawl(keyword: &str) -> impl Future<Item=(), Error=()> {
    Client::new()
        .get(&format!("http://www.baidu.com/s?wd={}", keyword))
        .send()
        .and_then(|mut res| {
            println!("{}", res.status());

            let body = mem::replace(res.body_mut(), Decoder::empty());
            body.concat2()
        })
        .map_err(|err| println!("request error: {}", err))
        .map(Cursor::new)
        .and_then(move|mut body| {
            OpenOptions::new().create(true).write(true).open("./prey.html").and_then(move |mut f| {
                let _ = io::copy(&mut body, &mut f)
                    .map_err(|err| {
                        println!("stdout error: {}", err);
                    });
                Ok(())
            }).map_err(|_|{})
        })
}

fn main() {
    if let Some(keyword) = std::env::args().nth(1) {
        tokio::run(crawl(&keyword));
    } else {
        println!("no keyword provided");
    }

}