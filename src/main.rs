use anyhow::anyhow;
use anyhow::Ok;
use clap::{Args, Parser, Subcommand};

use serde::{Deserialize, Serialize};
use sha1::{self, Digest, Sha1};
use std::collections::HashMap;

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
            let torrent = read_torrent_file(args.file_name)?;

            eprintln!("{torrent:?}");

            println!("Tracker URL: {}", torrent.announce);
            println!("Length: {}", torrent.info.length);
            //Expected hash: 70edcac2611a8829ebf467a6849f5d8408d9d8f4

            println!("Info Hash: {}", hex::encode(torrent.info_hash));

            Ok(())
        }
    }
}

fn read_torrent_file(file_name: String) -> anyhow::Result<Torrent> {
    let content = std::fs::read(file_name)?;

    //let t: Torrent = serde_bencode::from_bytes(content.as_slice())?;

    //println!("t: {:#?}", t);
    let value: serde_bencode::value::Value = serde_bencode::from_bytes(content.as_slice())?;
    //Ok(t)
    match value {
        serde_bencode::value::Value::Dict(d) => {
            let announce = extract_string("announce", &d)?;

            let info = extract_dict("info", &d)?;

            let val = d.get("info".as_bytes()).unwrap();

            // HashMap::from_iter(
            //     info.into_iter()
            //         .map(|(k, v)| (serde_bencode::from_str(String::from_utf8(k)?.as_str()), v))
            //         .into_iter(),
            // );

            //for (k, v) in &info.values().into_iter().collect() {}
            let mut h: HashMap<String, Vec<u8>> = HashMap::new();

            let length = extract_int("length", &info)?;
            let name = extract_string("name", &info)?;
            let piece_length = extract_int("piece length", &info)?;
            let pieces = extract_bytes("pieces", &info)?;
            h.insert("length".to_string(), bincode::serialize(&length).unwrap());
            h.insert("name".to_string(), bincode::serialize(&name).unwrap());
            h.insert(
                "piece_length".to_string(),
                bincode::serialize(&piece_length).unwrap(),
            );
            h.insert("pieces".to_string(), pieces.clone());

            Ok(Torrent {
                announce,
                info: TorrentInfo {
                    length,
                    name,
                    piece_length,
                    pieces,
                },
                info_hash: hash_dict(&h)?,
            })
        }
        _ => Err(anyhow!("Incorrect format, required dict")),
    }
}

fn hash_dict(h: &HashMap<String, Vec<u8>>) -> anyhow::Result<Vec<u8>> {
    let mut hasher = Sha1::new();
    let encoded: Vec<u8> = bincode::serialize(h).unwrap();
    hasher.update(&encoded);

    let result = hasher.finalize();
    Ok(result[..].to_vec())
}

// fn hash_dict(d: &serde_bencode::value::Value) -> anyhow::Result<Vec<u8>> {
//     let mut hasher = Sha1::new();
//     let encoded: Vec<u8> = bincode::serialize(d).unwrap();
//     hasher.update(&encoded);

//     let result = hasher.finalize();
//     Ok(result[..].to_vec())
// }

fn extract_string(
    key: &str,
    d: &HashMap<Vec<u8>, serde_bencode::value::Value>,
) -> anyhow::Result<String> {
    d.get(key.as_bytes())
        .and_then(|v| match v {
            serde_bencode::value::Value::Bytes(b) => String::from_utf8(b.clone()).ok(),
            _ => None,
        })
        .ok_or(anyhow!("Missing field: {}", key))
}

fn extract_dict(
    key: &str,
    d: &HashMap<Vec<u8>, serde_bencode::value::Value>,
) -> anyhow::Result<HashMap<Vec<u8>, serde_bencode::value::Value>> {
    d.get(key.as_bytes())
        .and_then(|v| match v {
            serde_bencode::value::Value::Dict(d) => Some(d.clone()),
            _ => None,
        })
        .ok_or(anyhow!("Missing field: {}", key))
}

fn extract_int(
    key: &str,
    d: &HashMap<Vec<u8>, serde_bencode::value::Value>,
) -> anyhow::Result<i64> {
    d.get(key.as_bytes())
        .and_then(|v| match v {
            serde_bencode::value::Value::Int(i) => Some(*i),
            _ => None,
        })
        .ok_or(anyhow!("Missing filed: {}", key))
}

fn extract_bytes(
    key: &str,
    d: &HashMap<Vec<u8>, serde_bencode::value::Value>,
) -> anyhow::Result<Vec<u8>> {
    d.get(key.as_bytes())
        .and_then(|v| match v {
            serde_bencode::value::Value::Bytes(b) => Some(b.clone()),
            _ => None,
        })
        .ok_or(anyhow!("Missing field: {}", key))
}

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

// fn hash_dict(d: &HashMap<Vec<u8>, serde_bencode::value::Value>) -> anyhow::Result<Vec<u8>> {
//     let mut new_map: HashMap<String, serde_bencode::value::Value> = HashMap::new();

//     for (k, v) in d {
//         let new_key = serde_bencode::to_string(&String::from_utf8(k.to_vec()).unwrap())?;
//         new_map.insert(new_key, v.clone());
//     }
//     println!("Org map: {:#?}", d);
//     println!("New map: {:#?}", new_map);

//     let mut hasher = Sha1::new();
//     let encoded: Vec<u8> = bincode::serialize(&new_map).unwrap();
//     hasher.update(&encoded);

//     let result = hasher.finalize();
//     Ok(result[..].to_vec())
// }

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Torrent {
    announce: String,
    info: TorrentInfo,
    info_hash: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
struct TorrentInfo {
    length: i64,
    name: String,
    #[serde(rename = "piece length")]
    piece_length: i64,
    pieces: Vec<u8>,
}
