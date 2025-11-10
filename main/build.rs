use std::env;

fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    println!("cargo:rustc-env=token={}", env::var("token").unwrap());
}
