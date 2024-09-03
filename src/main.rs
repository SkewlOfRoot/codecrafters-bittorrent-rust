use clap::{Args, Parser, Subcommand};

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
            let decoded_value = decode_bencoded_value(&encoded_value);
            println!("{}", decoded_value);
        }
    }
}

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    // If encoded_value starts with a digit, it's a number
    let first_char = encoded_value.chars().next().unwrap();
    // Example: "5:hello" -> "hello"
    if first_char.is_ascii_digit() {
        let decoded: String = serde_bencode::from_str(encoded_value).unwrap();
        serde_json::Value::String(decoded)
    // Example: i52e -> 52
    } else if first_char == 'i' {
        let decoded: i64 = serde_bencode::from_str(encoded_value).unwrap();
        serde_json::Value::Number(serde_json::Number::from(decoded))
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}
