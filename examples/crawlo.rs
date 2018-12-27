
//use spidero::Prey;
use std::{
    mem,
    io::{
        Cursor
    },
    default::Default,
    iter::repeat
};
use futures::{
    Future,
    Stream
};
use reqwest::r#async::{
    Client,
    Decoder
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
        .and_then(move|mut body| {

//            let mut html = String::new();
//            match body.read_to_string(&mut html) {
//                Ok(_) => {
//                    println!("{}", html);
//
//
//
//                    //println!("title: {} ({:?})", dom.at("title").unwrap().content().trim(), start.elapsed().unwrap())
//                },
//                Err(e) => eprintln!("{}", e)
//            }

            let opts = ParseOpts {
                tree_builder: TreeBuilderOpts {
                    drop_doctype: true,
                    ..Default::default()
                },
                ..Default::default()
            };

            let dom = parse_document(RcDom::default(), opts)
                .from_utf8()
                .read_from(&mut body)
                .unwrap();

            let mut prey = vec![];
            walk(&mut prey, 0, dom.document);

            prey.iter().for_each(|s| println!("{}", s));

            Ok(())
        })
}

fn walk(preys: &mut Vec<String>, indent: usize, handle: Handle) {
    let node = handle;
    // FIXME: don't allocate
    print!("{}", repeat(" ").take(indent).collect::<String>());
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

pub fn escape_default(s: &str) -> String {
    s.chars().flat_map(|c| c.escape_default()).collect()
}

fn main() {
    if let Some(keyword) = std::env::args().nth(1) {
        println!("keyword: {}", keyword);
        tokio::run(crawl_baidu(&keyword));
    } else {
        println!("no keyword provided");
    }

}