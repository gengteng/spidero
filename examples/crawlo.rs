use spidero::Prey;
use serde_json::to_string;


fn main() {
    let prey = Prey {
        url: "https://google.com/",
        title: "hahaha",
        content: "wtf"
    };

    match to_string(&prey) {
        Ok(s) =>
            match serde_json::from_str::<Prey>(&s) {
                Ok(p) => println!("{:?}", p),
                Err(e) => eprintln!("deserialize err: {}", e)
            },
        Err(e) => eprintln!("serialize err: {}", e)
    }
}