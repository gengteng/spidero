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

#[derive(Debug, Serialize, Deserialize)]
pub struct Prey {
    pub url: String,
    pub title: String,
    pub content: Option<String>
}

pub struct Spider {
    client: Client
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

    pub fn hatch_with_proxy(proxy: Proxy) -> Result<Spider, Box<dyn std::error::Error>> {

        match Client::builder().proxy(proxy).build() {
            Ok(client) => {
                Ok(Spider {
                    client
                })
            },
            Err(e) => {
                Err(Box::new(e))
            }
        }
    }

    pub fn weave(&self, engine: SearchEngine, keyword: &str, count: u32) -> Web {
        Web {
            spider: self,
            engine,
            keyword: keyword.to_string(),
            count
        }
    }
}

pub struct Web<'a> {
    spider: &'a Spider,
    engine: SearchEngine,
    keyword: String,
    count: u32
}

impl<'a> futures::Stream for Web<'a> {
    type Item = Prey;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        unimplemented!()
    }
}