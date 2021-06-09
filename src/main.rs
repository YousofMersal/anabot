use dotenv;
use std::env;

fn main() {
    dotenv::dotenv().ok();
    let _ans = env::var("TOKEN").expect("Expected token");

    println!("Hello, world!");
}
