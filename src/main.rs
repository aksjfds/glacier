#![allow(unused)]

use std::{io::Write, time};
use tokio::fs::File as AsyncFile;

use glacier::prelude::*;

// #[glacier(GET, "/1")]
// async fn file1(_req: Request, mut res: Response) {
//     let start = time::Instant::now();
//     match AsyncFile::open("/home/aksjfds/codes/glacier/largefile.txt").await {
//         Ok(f) => {
//             res.send_file(f).await.unwrap();
//         }
//         Err(e) => println!("{:#?}", e),
//     }

//     println!("传输耗时: {:#?}ms", start.elapsed().as_millis());
// }

#[glacier(GET, "/")]
async fn basic(req: Request, res: Response) {
    // println!("{:#?}", "hello");

    // println!("{:#?}", req.headers.last().unwrap());

    res.respond().await;
}

#[main]
async fn main() {
    let glacier = Glacier::bind(3000, routes).await.unwrap();

    println!("{:#?}", "http://localhost:3000");
    glacier.run().await.unwrap();
}
