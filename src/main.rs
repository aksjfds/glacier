#![allow(unused)]

use std::{fs::File, io::Read};
use tokio::fs::File as AsyncFile;

use glacier::prelude::*;

#[glacier(GET, "/")]
async fn basic(req: Request, res: Response) {
    // println!("{:#?}", "hello");

    // hello(req, res).await;
    res.respond().await;
}

#[glacier(GET, "/1")]
async fn file1(_req: Request, mut res: Response) {
    let mut f = File::open("../readme.md").unwrap();
    let mut data = String::new();
    f.read_to_string(&mut data);

    res.body = data;
    res.respond().await;
}

#[glacier(GET, "/2")]
async fn file2(_req: Request, mut res: Response) {
    match AsyncFile::open("../largefile.txt").await {
        Ok(f) => {
            res.send_file(f).await;
        }
        Err(e) => println!("{:#?}", e),
    }
}

#[main]
async fn main() {
    let glacier = Glacier::bind(3000, routes).await.unwrap();

    println!("{:#?}", "http://localhost:3000");
    glacier.run().await.unwrap();
}
