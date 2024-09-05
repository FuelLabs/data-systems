use std::env;

use dotenvy::dotenv;

fn main() {
    dotenv().expect(".env file not found");

    if let Ok(value) = env::var("NATS_PUBLIC_PASS") {
        println!("cargo:rustc-env=NATS_PUBLIC_PASS={}", value);
    }
}
