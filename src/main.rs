use std::io::Read;

use anyhow::Ok;
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
    Info(InfoArgs),
}

#[derive(Args)]
struct DecodeArgs {
    value: String,
}

#[derive(Args)]
struct InfoArgs {
    file_name: String,
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
        Commands::Info(args) => {
            read_torrent_file(args.file_name)?;
            Ok(())
        }
    }
}

fn read_torrent_file(file_name: String) -> anyhow::Result<()> {
    let mut file = std::fs::File::open(file_name)?;
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer)?;

    let content = String::from_utf8_lossy(&buffer);
    let content = decode_bencoded_value(&content)?;
    eprintln!("{}", content);

    let map = content.as_object().unwrap();
    eprintln!("{:#?}", map);
    let tracker_url = map.get("announce").unwrap();
    let file_length = map.get("length").unwrap();

    println!("Tracker URL: {}", tracker_url);
    println!("Length: {}", file_length);
    Ok(())
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
        serde_bencode::value::Value::Dict(d) => {
            let val = d
                .into_iter()
                .map(|(k, v)| {
                    let key = String::from_utf8(k)?;
                    let value = convert(v)?;
                    Ok((key, value))
                })
                .collect::<anyhow::Result<serde_json::Map<String, serde_json::Value>>>()?;
            Ok(serde_json::Value::Object(val))
        }
    }
}
