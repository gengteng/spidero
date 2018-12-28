
#[macro_use]
extern crate serde_derive;

use std::{
    mem,
    io::{
        Cursor,
        Read
    },
    default::Default,
//    iter::repeat
};
use futures::{
    Future,
    Stream
};
use reqwest::{
    r#async::{
        Client,
        Decoder,
    }
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

#[macro_use]
extern crate clap;

use clap::{
    App,
    Arg,
    //SubCommand
};

#[derive(Debug, Serialize, Deserialize)]
struct Prey {
    url: String,
    title: String,
    content: Option<String>
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

    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("keyword")
            .short("k")
            .long("keyword")
            .value_name("KEYWORD")
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
            .short("m")
            .long("bing")
            .help("search with bing")
            .takes_value(false));

    let matches = app.get_matches();

    if let Some(keyword) = matches.value_of("keyword") {

        if matches.is_present("baidu") {
            println!("keyword: {}, engine: baidu", keyword);
            tokio::run(crawl_baidu(&keyword));
        }

        if matches.is_present("google") {
            println!("keyword: {}, engine: google", keyword);
        }

        if matches.is_present("bing") {
            println!("keyword: {}, engine: bing", keyword);
        }

    } else {
        println!("no keyword provided");
    }

}