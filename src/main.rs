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
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.commands {
        Commands::Decode(args) => {
            let encoded_value = args.value;
            let decoded_value = decode_bencoded_value(&encoded_value)?;
            println!("{}", decoded_value);
            Ok(())
        }
    }
}

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> anyhow::Result<serde_json::Value> {
    let value: serde_bencode::value::Value = serde_bencode::from_str(encoded_value)?;
    convert(value)
}

fn convert(value: serde_bencode::value::Value) -> anyhow::Result<serde_json::Value> {
    match value {
        serde_bencode::value::Value::Bytes(b) => {
            let s = String::from_utf8(b)?;
            Ok(serde_json::Value::String(s))
        }
        serde_bencode::value::Value::Int(i) => {
            Ok(serde_json::Value::Number(serde_json::Number::from(i)))
        }
        serde_bencode::value::Value::List(l) => {
            let val = l
                .into_iter()
                .map(convert)
                .collect::<anyhow::Result<Vec<serde_json::Value>>>()?;

            Ok(serde_json::Value::Array(val))
        }
        _ => {
            panic!("Unhandled encoded value: {:?}", value)
        }
    }
}
