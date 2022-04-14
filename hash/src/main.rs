use crate::HashAlgorithm::MD5;
use blake3::Hasher;
use clap::{AppSettings, Arg, Command};
use sha2::Digest as Sha2Digest;
use sha2::Sha256;
use std::fs::OpenOptions;
use std::path::Path;

fn build_app() -> Command<'static> {
    Command::new("hash")
        .author("asingingbird.cb")
        .version("1.0.0")
        .about("Print string or file checksums.")
        .setting(AppSettings::DeriveDisplayOrder)
        .override_usage("hash --[md5|sha256|blake3] --text <text>\n    hash --[md5|sha256|blake3] --file <path>")
        .arg(Arg::new("md5").short('M').long("md5").help("Compute the hash using md5 algorithm.").takes_value(false))
        .arg(Arg::new("sha256").short('S').long("sha256").help("Compute the hash using sha256 algorithm.").takes_value(false))
        .arg(Arg::new("blake3").short('B').long("blake3").help("Compute the hash using blake3 algorithm.").takes_value(false))
        .arg(Arg::new("hash_algorithm").short('a').long("algorithm").help("Choose a hash algorithm.").possible_values(["md5", "sha256", "blake3"]).takes_value(true))
        .arg(
            Arg::new("text")
                .short('t')
                .long("text")
                .value_name("text")
                .help("Compute the hash of this text. Can be provided multiple times, compute the hash of each text.")
                .takes_value(true)
                .multiple_occurrences(true)
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("file")
                .help("Compute the hash of this file. Can be provided multiple times, compute the hash of each file.")
                .takes_value(true)
                .multiple_occurrences(true)
        )
        .arg(
            Arg::new("interactive_mode")
                .short('i')
                .long("interactive")
                .value_name("mode")
                .help("Run in interactive mode. Update on each text/file input.")
                .possible_values(["text", "file"])
                .default_value("text")
                .takes_value(true)
        )
        .arg(
            Arg::new("hex")
                .short('H')
                .long("hex")
                .help("Treat the text or file content as hex strings, e.g. \"0x19 0xab 0xcd 0xef\"")
                .takes_value(false)
        )
}

#[derive(Copy, Clone, Debug)]
pub enum HashAlgorithm {
    MD5,
    SHA256,
    BLAKE3,
}

pub struct UpdateHistory {
    pub history: String,
}

impl UpdateHistory {
    pub fn new() -> Self {
        UpdateHistory {
            history: String::new(),
        }
    }

    pub fn add_file(&mut self, path: &str) {
        let entry: String = path.chars().take(40).collect();
        self.history.push_str(&format!("[FILE] [{}]\n", entry));
    }

    pub fn add_text(&mut self, text: &str) {
        let entry: String = text.chars().take(40).collect();
        self.history.push_str(&format!("[TEXT] [{}]\n", entry));
    }

    pub fn summary(&self) -> String {
        let line = "=".repeat(49) + "\n";
        format!("{}{}{}", line, self.history, line)
    }
}

#[derive(Clone, Debug)]
struct HashImpl {
    pub bytes: Vec<u8>,
}

impl HashImpl {
    pub fn new() -> Self {
        HashImpl { bytes: Vec::new() }
    }

    pub fn from(input: &[u8]) -> Self {
        HashImpl {
            bytes: Vec::from(input),
        }
    }

    pub fn reset(&mut self) {
        self.bytes.clear();
    }

    pub fn update(&mut self, input: &[u8]) {
        self.bytes.extend_from_slice(input);
    }

    pub fn digest(&self, algo: HashAlgorithm) -> Vec<u8> {
        match algo {
            MD5 => Self::md5hash(&self.bytes),
            HashAlgorithm::SHA256 => Self::sha256hash(&self.bytes),
            HashAlgorithm::BLAKE3 => Self::blake3hash(&self.bytes),
        }
    }

    pub fn hex_digest(&self, algo: HashAlgorithm) -> String {
        let hash = match algo {
            MD5 => Self::md5hash(&self.bytes),
            HashAlgorithm::SHA256 => Self::sha256hash(&self.bytes),
            HashAlgorithm::BLAKE3 => Self::blake3hash(&self.bytes),
        };
        hash.iter().map(|b| format!("{:02x}", b)).collect()
    }

    pub fn md5hash(input: &[u8]) -> Vec<u8> {
        md5::compute(input).0.to_vec()
    }

    pub fn sha256hash(input: &[u8]) -> Vec<u8> {
        sha2::Sha256::digest(input).to_vec()
    }

    pub fn blake3hash(input: &[u8]) -> Vec<u8> {
        blake3::hash(input).as_bytes().to_vec()
    }
}

fn run_interactive_mode() {}

fn main() {
    let app = build_app();
    let matches = app.get_matches();
    println!("Hello, world!");
}
