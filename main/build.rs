use std::env;

const ENVIRONMENTVARIABLES: [&str; 6] = [
    "TOKEN",
    "APPLICATION_ID",
    "BACKEND_URL",
    "LAVALINK_HOST",
    "LAVALINK_PORT",
    "LAVALINK_PASSWORD",
];

fn main() {
    // Check for ENVIRONMENT variable to determine which .env file to load
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    let env_file = match environment.as_str() {
        "staging" => ".env.staging",
        "production" => ".env.production",
        _ => ".env.dev", // Default to .env for development or unknown environments
    };

    // Load the appropriate .env file
    if let Err(e) = dotenv::from_filename(env_file) {
        eprintln!("Warning: Failed to load {} file: {}", env_file, e);
        // Fallback to default .env file if the specific one doesn't exist
        if env_file != ".env" {
            if let Err(e) = dotenv::dotenv() {
                panic!("Failed to load any .env file: {}", e);
            }
        } else {
            panic!("Failed to load .env file: {}", e);
        }
    }

    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=.env.staging");
    println!("cargo:rerun-if-changed=.env.production");
    println!("cargo:rerun-if-env-changed=ENVIRONMENT");

    for &var in &ENVIRONMENTVARIABLES {
        let value = env::var(var).unwrap_or_default();
        println!("cargo:rustc-env={}={}", var, value);
    }
}
