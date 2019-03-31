use futures::{
    Future,
    Stream,
    Poll
};

use reqwest::{
    r#async::{
        Client,
        Decoder,
    },
    Proxy
};

use crate::errors::SpiderError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Prey {
    pub url: String,
    pub title: String,
    pub content: Option<String>
}

pub struct Spider {
    pub client: Client
}

pub enum SearchEngine {
    Google,
    Baidu,
    Bing
}

impl Spider {
    pub fn hatch() -> Spider {
        Spider {
            client: Client::new()
        }
    }

    pub fn hatch_with_proxy(proxy: Proxy) -> Result<Spider, SpiderError> {

        let client = Client::builder().proxy(proxy).build()?;

        Ok(Spider {
            client
        })
    }

    pub fn weave<'a>(&'a self, engine: SearchEngine, keyword: &'a str, count: u32) -> Web<'a> {
        Web {
            spider: &self,
            engine,
            keyword,
            count,
            f: None
        }
    }
}

pub struct Web<'a> {
    spider: &'a Spider,
    engine: SearchEngine,
    keyword: &'a str,
    count: u32,
    f: Option<Box<Future<Item=Vec<Prey>, Error=SpiderError>>>
}

impl<'a> Future for Web<'a> {
    type Item = Vec<Prey>;
    type Error = SpiderError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {

        unimplemented!()
    }
}