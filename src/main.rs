use glacier::prelude::*;

#[glacier(GET, "/")]
async fn hello(_req: Request, mut res: Response) {
    // println!("{:#?}", "hello");

    res.respond().await;
}

#[main]
async fn main() {
    let glacier = Glacier::bind(3000, routes).await.unwrap();

    println!("{:#?}", "http://localhost:3000");
    glacier.run().await;
}
