
#[macro_use]
extern crate serde_derive;

use std::{
    mem,
    io::{
        Cursor,
        Read
    },
    default::Default,
    iter::repeat
};


use html5ever::{
    driver::ParseOpts,
    rcdom::{
        Handle,
        RcDom,
        NodeData
    },
    tree_builder::TreeBuilderOpts,
    tendril::TendrilSink,
    parse_document
};

use url::{
    Url,
    ParseError
};

mod spider;
use self::spider::*;

mod errors;
use self::errors::SpiderError;

#[macro_use]
extern crate clap;

use clap::{
    App,
    Arg,
    //SubCommand
};

use futures::{
    future::*,
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

fn crawl_bing(keyword: &str) -> impl Future<Item=(), Error=()> {
    Client::new().get(&format!("https://www.bing.com/search?q={}", keyword))
        .send().and_then(|mut r|{
            let body = mem::replace(r.body_mut(), Decoder::empty());
            body.concat2().map(Cursor::new).map(|c| (c, r))
        }).and_then(|(mut body, r)|{
            let opts = ParseOpts {
                tree_builder: TreeBuilderOpts {
                    drop_doctype: true,
                    ..TreeBuilderOpts::default()
                },
                ..ParseOpts::default()
            };

            let result = parse_document(RcDom::default(), opts)
                .from_utf8()
                .read_from(&mut body);

            match result {
                Ok(dom) => {
                    walk_all(0, dom.document);
                },
                Err(e) => {
                    println!("parse error: {}", e);
                }
            }


            ok(())
        }).map_err(|_|{})
}

fn crawl_google(keyword: &str) -> impl Future<Item=(), Error=()> {
    Client::new().get(&format!("https://www.google.com/search?hl=en&num=10&start=0&q={}", keyword))
        .send().and_then(|mut r|{
            let body = mem::replace(r.body_mut(), Decoder::empty());
            body.concat2().map(Cursor::new).map(|c| (c, r))
        }).and_then(|(mut body, r)| {
            //println!("remote: {}", r.remote_addr().unwrap());

            let opts = ParseOpts {
                tree_builder: TreeBuilderOpts {
                    drop_doctype: true,
                    ..TreeBuilderOpts::default()
                },
                ..ParseOpts::default()
            };

            let result = parse_document(RcDom::default(), opts)
                .from_utf8()
                .read_from(&mut body);

            match result {
                Ok(dom) => {
                    walk_all(0, dom.document);
                },
                Err(e) => {
                    println!("parse error: {}", e);
                }
            }


            ok(())
        }).map_err(|_|{})
}

fn baidu(client: &Client, keyword: &str, count: u32) -> impl Future<Item=Vec<Prey>, Error=SpiderError> {
    client.get(&format!("http://www.baidu.com/s?wd={}", keyword))
        .send()
        .and_then(|mut res| {
            let body = mem::replace(res.body_mut(), Decoder::empty());
            body.concat2()
        }).from_err().map(Cursor::new).and_then(|mut cursor|{
            let opts = ParseOpts {
                tree_builder: TreeBuilderOpts {
                    drop_doctype: true,
                    ..TreeBuilderOpts::default()
                },
                ..ParseOpts::default()
            };

            let result = parse_document(RcDom::default(), opts)
                .from_utf8()
                .read_from(&mut cursor);

            match result {
                Ok(dom) => {
                    walk_all(0, dom.document);
                    ok(Vec::new())
                },
                Err(e) => {
                    eprintln!("parse error: {}", e);
                    err(SpiderError::HttpParseError)
                }
            }
        })
}

fn crawl_baidu(keyword: &str) -> impl Future<Item=(), Error=()> {
    Client::new()
        .get(&format!("http://www.baidu.com/s?wd={}", keyword))
        //.get("http://tokio.rs")
        .send()
        .and_then(|mut res| {
            //println!("{}", res.status());

            let body = mem::replace(res.body_mut(), Decoder::empty());
            body.concat2()
        })
        .map_err(|err| println!("request error: {}", err))
        .map(Cursor::new)
        .and_then(move |mut body| {

            let opts = ParseOpts {
                tree_builder: TreeBuilderOpts {
                    drop_doctype: true,
                    ..TreeBuilderOpts::default()
                },
                ..ParseOpts::default()
            };

            let result = parse_document(RcDom::default(), opts)
                .from_utf8()
                .read_from(&mut body);

            match result {
                Ok(dom) => {
                    let mut prey = vec![];
                    walk(&mut prey, 0, dom.document);

                    prey.iter().for_each(|s| {
                        let parse = json5::from_str::<Prey>(s);
                        match parse {
                            Ok(mut prey) => {
                                //println!("{:?}", prey);
                                tokio::spawn(
                                    Client::new()
                                        .get(&prey.url)
                                        //.get("http://tokio.rs")
                                        .send()
                                        .and_then(|mut res| {
                                            //prey.url = res.url().clone().to_string();

                                            let body = mem::replace(res.body_mut(), Decoder::empty());
                                            body.concat2().map(Cursor::new).map_err(|e|e).and_then(move |mut body|{
                                                let mut content = String::new();
                                                match body.read_to_string(&mut content) {
                                                    Ok(_) => {
                                                        prey.url = res.url().clone().to_string();
                                                        //prey.content = Some(content);


                                                        println!("{}", json5::to_string(&prey).unwrap());

                                                        //return Ok(());
                                                    },
                                                    Err(e) => {
                                                        eprintln!("{}", e);
                                                        //return Err(());
                                                    }
                                                }

                                                Ok(())
                                            })
                                        }).map_err(|err| eprintln!("request error: {}", err))
                                );
                            },
                            Err(e) => eprintln!("json deserialize error: {}", e)
                        }
                    });

                    Ok(())
                },
                Err(_) => {
                    Err(())
                }
            }
        })
}

fn walk_all(indent: usize, handle: Handle) {
    let node = handle;
    // FIXME: don't allocate
    print!("{}", repeat(" ").take(indent).collect::<String>());
    match node.data {
        NodeData::Document => println!("#Document"),

        NodeData::Doctype {
            ref name,
            ref public_id,
            ref system_id,
        } => println!("<!DOCTYPE {} \"{}\" \"{}\">", name, public_id, system_id),

        NodeData::Text { ref contents } => {
            println!("#text: {}", escape_default(&contents.borrow()))
        },

        NodeData::Comment { ref contents } => println!("<!-- {} -->", escape_default(contents)),

        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            print!("<{}", name.local);
            for attr in attrs.borrow().iter() {
                print!(" {}=\"{}\"", attr.name.local, attr.value);
            }
            println!(">");
        },

        _ => {}

        NodeData::ProcessingInstruction { .. } => unreachable!(),
    }

    for child in node.children.borrow().iter() {
        walk_all(indent + 4, child.clone());
    }
}

fn escape_default(s: &str) -> String {
    s.chars().flat_map(|c| c.escape_default()).collect()
}

fn walk(preys: &mut Vec<String>, indent: usize, handle: Handle) {
    let node = handle;
    // FIXME: don't allocate
    //print!("{}", repeat(" ").take(indent).collect::<String>());
    match node.data {
//        NodeData::Document => {},//println!("#Document"),
//
//        NodeData::Doctype {
//            ref name,
//            ref public_id,
//            ref system_id,
//        } => println!("<!DOCTYPE {} \"{}\" \"{}\">", name, public_id, system_id),
//
//        NodeData::Text { ref contents } => {
//            println!("#text: {}", escape_default(&contents.borrow()))
//        },
//
//        NodeData::Comment { ref contents } => println!("<!-- {} -->", escape_default(contents)),

        NodeData::Element {
            //ref name,
            ref attrs,
            ..
        } => {
            //print!("<{}", name.local);
            for attr in attrs.borrow().iter() {
                //print!(" {}=\"{}\"", attr.name.local, attr.value);

                if attr.name.local.to_string() == "data-tools" {
                    preys.push(attr.value.to_string());
                }
            }
            //println!(">");
        },

        _ => {}

        //NodeData::ProcessingInstruction { .. } => unreachable!(),
    }

    for child in node.children.borrow().iter() {
        let mut p = vec![];
        walk(&mut p, indent + 4, child.clone());
        preys.append(&mut p);
    }
}

//fn escape_default(s: &str) -> String {
//    s.chars().flat_map(|c| c.escape_default()).collect()
//}

fn main() {

    //tokio::run(crawl_bing("fuck"));

    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("keywords")
            .short("k")
            .long("keywords")
            .value_name("KEYWORDS")
            .help("keyword to search")
            .takes_value(true))
        .arg(Arg::with_name("google")
            .short("g")
            .long("google")
            .help("search with google")
            .takes_value(false))
        .arg(Arg::with_name("baidu")
            .short("b")
            .long("baidu")
            .help("search with baidu")
            .takes_value(false))
        .arg(Arg::with_name("bing")
            .short("n")
            .long("bing")
            .help("search with bing")
            .takes_value(false))
        .arg(Arg::with_name("proxy")
            .short("x")
            .long("proxy")
            .help("http(s) proxy")
            .takes_value(true));

    let matches = app.get_matches();

    if let Some(keywords) = matches.value_of("keywords") {

        let keywords: Vec<&str> = keywords.split(',').collect();

        let engines = {
            let mut v = vec![];
            if matches.is_present("baidu") {
                v.push("baidu");
            }

            if matches.is_present("google") {
                v.push("google");
            }

            if matches.is_present("bing") {
                v.push("bing");
            }

            v
        };

        if engines.len() == 0 {
            println!("no search engine provided");
            return;
        }

        println!("keywords: {:?}, search engine: {:?}", keywords, engines);

        let spider: Spider = if let Some(proxy_url) = matches.value_of("proxy") {
            match Url::parse(&proxy_url) {
                Ok(_) => {

                    match Proxy::all(proxy_url) {
                        Ok(proxy) => {
                            match Spider::hatch_with_proxy(proxy) {
                                Ok(spider) => spider,
                                Err(e) => {
                                    eprintln!("create client error: {}", e);
                                    return;
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("create proxy error: {}", e);
                            return;
                        }
                    }

                }
                Err(e) => {
                    eprintln!("proxy parse error: {}", e);
                    return;
                }
            }
        } else {
            Spider::hatch()
        };

        tokio::run(baidu(&spider.client, "fuck", 15).map(|_|{}).map_err(|_|{}));

    } else {
        println!("no keyword provided");
    }
}