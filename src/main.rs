use clap::{Args, Parser, Subcommand};
use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    // If encoded_value starts with a digit, it's a number
    if encoded_value.chars().next().unwrap().is_ascii_digit() {
        // Example: "5:hello" -> "hello"
        let colon_index = encoded_value.find(':').unwrap();
        let number_string = &encoded_value[..colon_index];
        let number = number_string.parse::<i64>().unwrap();
        let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
        serde_json::Value::String(string.to_string())
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}

#[derive(Parser)]
#[clap(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Decode(DecodeArgs),
}

#[derive(Args)]
struct DecodeArgs {
    value: String,
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let cli = Cli::parse();

    match cli.commands {
        Commands::Decode(args) => {
            let encoded_value = args.value;
            //let decoded_value = decode_bencoded_value(&encoded_value);
            let decoded_value: i32 = serde_bencode::from_str(&encoded_value).unwrap();
            println!("{}", decoded_value);
        }
    }
}
