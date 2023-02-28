// fn main() {
//     let test_string = String::from("test");
//     println!("Hello, world!");
//
//
// }
// fn main() {
//     let result = 10.00;        //f64 by default
//     let interest:f32 = 8.35;
//     let cost:f64 = 15000.600;  //double precision
//
//     println!("result value is {}",result);
//     println!("interest is {}",interest);
//     println!("cost is {}",cost);
// }
//==========================================================
// use serde::Deserialize;
// use reqwest::Error;
//
// #[derive(Deserialize, Debug)]
// struct User {
//     login: String,
//     id: u32,
// }
//
// #[tokio::main]
// async fn main() -> Result<(), Error> {
//     let request_url = format!("https://api.github.com/repos/{owner}/{repo}/stargazers",
//                               owner = "rust-lang-nursery",
//                               repo = "rust-cookbook");
//     println!("{}", request_url);
//     let response = reqwest::get(&request_url).await?;
//
//     let users: Vec<User> = response.json().await?;
//     println!("{:?}", users);
//     Ok(())
// }

//========================================================================================
use reqwest;

use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;

#[derive(Serialize, Debug)]
struct Track {
    name: String,
    href: String,
}

fn print_tracks(tracks: Vec<&Track>) {
    println!("total tracks {}", tracks.len())
}

// This is using the `tokio` runtime. You'll need the following dependency:
//
// `tokio = { version = "1", features = ["full"] }`
// #[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
// async fn main() -> Result<(), reqwest::Error> {
async fn main() {
    // Some simple CLI args requirements...
    let url = match std::env::args().nth(1) {
        Some(url) => url,
        None => {
            println!("No CLI URL provided, using default.");
            "https://hyper.rs".into()
        }
    };

    let auth: Vec<String> = std::env::args().collect();
    println!("env {:?}", auth);
    eprintln!("Fetching {:?}...", url);

    // reqwest::get() is a convenience function.
    //
    // In most cases, you should create/build a reqwest::Client and reuse
    // it for all requests.
    // let res = reqwest::get(url).await?;

    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", &auth[2]))
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .send()
        .await
        .unwrap();

    // eprintln!("Response: {:?} {}", res.version(), res.status());
    // eprintln!("Headers: {:#?}\n", res.headers());

    match response.status() {
        reqwest::StatusCode::OK => {
            match response.text().await {
                Ok(parsed) => println!("these are the tracks {}", parsed), //print_tracks(parsed),
                Err(_) => println!("Hm, the response didn't match the shape we expected."),
            };
        }
        reqwest::StatusCode::UNAUTHORIZED => {
            println!("Need to grab a new token");
        }
        other => {
            panic!("Uh oh! Something unexpected happened: {:?}", other);
        }
    };

    // let body = res.text().await?;

    // println!("{}", body);

    // Ok(())
}

// The [cfg(not(target_arch = "wasm32"))] above prevent building the tokio::main function
// for wasm32 target, because tokio isn't compatible with wasm32.
// If you aren't building for wasm32, you don't need that line.
// The two lines below avoid the "'main' function not found" error when building for wasm32 target.
// #[cfg(target_arch = "wasm32")]
// fn main() {}
