use clap::{AppSettings, Arg, ArgMatches, Command};
use sha2::Digest;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::string::String;

fn build_app() -> Command<'static> {
    Command::new("hash")
        .author("asingingbird.cb")
        .version("1.0.0")
        .about("Print string or file checksums.")
        .setting(AppSettings::DeriveDisplayOrder)
        .override_usage("hash --[md5|sha256|blake3] --text <text>\n    hash --[md5|sha256|blake3] --file <path>")
        .arg(
            Arg::new("sha256")
                .short('S')
                .long("sha256")
                .help("Compute the hash using sha256 algorithm (Default)")
        )
        .arg(
            Arg::new("md5")
                .short('M')
                .long("md5")
                .help("Compute the hash using md5 algorithm")
                .conflicts_with_all(&["sha256", "blake3"])
        )
        .arg(
            Arg::new("blake3")
                .short('B')
                .long("blake3")
                .help("Compute the hash using blake3 algorithm")
                .conflicts_with_all(&["md5", "sha256"])
        )
        .arg(
            Arg::new("text")
                .short('t')
                .long("text")
                .value_name("text")
                .help("Compute the hash of this text. Can be provided multiple times, compute the hash of each text")
                .takes_value(true)
                .multiple_occurrences(true)
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("file")
                .help("Compute the hash of this file. Can be provided multiple times, compute the hash of each file")
                .takes_value(true)
                .multiple_occurrences(true)
        )
        .arg(
            Arg::new("update")
                .short('u')
                .long("update")
                .help("Instead of computing the hash of each text/file, update on each of them, and print the finalized digest")
        )
        .arg(
            Arg::new("hex")
                .short('H')
                .long("hex")
                .help("Treat the text or file content as hex strings, e.g. '0x19 0xab 0xcd 0xef'")
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Do not print the text/file, just the hash")
        )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HexError<'a> {
    InvalidHexPrefix { hex: &'a str },
    InvalidLength { hex: &'a str },
    InvalidHexCharacter { ch: char, hex: &'a str },
}

impl<'a> std::error::Error for HexError<'a> {}

impl<'a> fmt::Display for HexError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            HexError::InvalidHexPrefix { hex } => {
                write!(
                    f,
                    "Invalid hex prefix '{}', should start with 0x or 0X",
                    hex
                )
            }
            HexError::InvalidLength { hex } => {
                write!(
                    f,
                    "Invalid string length '{}', should be 4, e.g. '0x12'",
                    hex
                )
            }
            HexError::InvalidHexCharacter { ch, hex } => {
                write!(f, "Invalid character '{}' in string '{}'", ch, hex)
            }
        }
    }
}

fn val(ch: char, hex: &str) -> Result<u8, HexError> {
    let chu8 = ch as u8;
    match ch {
        'A'..='F' => Ok(chu8 - b'A' + 10),
        'a'..='f' => Ok(chu8 - b'a' + 10),
        '0'..='9' => Ok(chu8 - b'0'),
        _ => Err(HexError::InvalidHexCharacter { ch, hex }),
    }
}

fn hex_to_byte(hex_string: &str) -> Result<u8, HexError> {
    if hex_string.len() != 4 {
        return Err(HexError::InvalidLength { hex: hex_string });
    } else if !hex_string.starts_with("0x") && !hex_string.starts_with("0X") {
        return Err(HexError::InvalidHexPrefix { hex: hex_string });
    }

    let mut chars = hex_string.chars().skip(2);
    let hi = match chars.next() {
        None => {
            return Err(HexError::InvalidLength { hex: hex_string });
        }
        Some(c) => c,
    };
    let hi = val(hi, hex_string)?;

    let lo = match chars.next() {
        None => {
            return Err(HexError::InvalidLength { hex: hex_string });
        }
        Some(c) => c,
    };
    let lo = val(lo, hex_string)?;

    Ok(hi * 16 + lo)
}

fn hex_to_byte_slice(hex_string: &str) -> Vec<u8> {
    hex_string
        .split(|c: char| c.is_whitespace() || c == ',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| match hex_to_byte(s) {
            Ok(b) => b,
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(exitcode::DATAERR);
            }
        })
        .collect()
}

fn bytes_to_hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[derive(Copy, Clone, Debug)]
pub enum HashAlgorithm {
    MD5,
    SHA256,
    BLAKE3,
}

impl Default for HashAlgorithm {
    fn default() -> Self {
        Self::SHA256
    }
}

#[derive(Clone, Debug, Default)]
pub struct OutputStyle {
    pub entry: String,
    pub len: usize,
    pub entry_type: &'static str,
    pub algo: HashAlgorithm,
    pub hash: String,
}

impl OutputStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, path: &str) {
        let entry_chars: Vec<_> = path.chars().take(40).collect();
        self.len = entry_chars.len();
        self.entry = String::from_iter(entry_chars);
        self.entry_type = "FILE";
    }

    pub fn add_text(&mut self, text: &str) {
        let entry_chars: Vec<_> = text.chars().take(40).collect();
        self.len = entry_chars.len();
        self.entry = String::from_iter(entry_chars);
        self.entry_type = "TEXT";
    }

    pub fn set_algorithm(&mut self, algorithm: HashAlgorithm) {
        self.algo = algorithm;
    }

    pub fn add_hash(&mut self, hash_str: &str) {
        self.hash = hash_str.to_string();
    }

    pub fn summary(&self, action: &str) -> String {
        let etc = if self.len < 40 { "" } else { "..." };
        let surr_line = "=".repeat(80);
        let entry_line = format!("[{} {}] [{}]{}", action, self.entry_type, self.entry, etc);
        let hash_line = format!("[{:?} HASH] [{}]", self.algo, self.hash);
        format!(
            "{}\n{}\n{}\n{}\n",
            surr_line, entry_line, hash_line, surr_line
        )
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

    pub fn update(&mut self, input: &[u8]) {
        self.bytes.extend_from_slice(input);
    }

    pub fn digest(input: &[u8], algo: HashAlgorithm) -> Vec<u8> {
        match algo {
            HashAlgorithm::MD5 => Self::md5hash(input),
            HashAlgorithm::SHA256 => Self::sha256hash(input),
            HashAlgorithm::BLAKE3 => Self::blake3hash(input),
        }
    }

    pub fn hex_digest(&self, algo: HashAlgorithm) -> String {
        bytes_to_hex_string(&Self::digest(&self.bytes, algo))
    }

    pub fn hex_digest_input(input: &[u8], algo: HashAlgorithm) -> String {
        bytes_to_hex_string(&Self::digest(input, algo))
    }

    fn md5hash(input: &[u8]) -> Vec<u8> {
        md5::compute(input).0.to_vec()
    }

    fn sha256hash(input: &[u8]) -> Vec<u8> {
        sha2::Sha256::digest(input).to_vec()
    }

    fn blake3hash(input: &[u8]) -> Vec<u8> {
        blake3::hash(input).as_bytes().to_vec()
    }
}

#[derive(Clone, Debug)]
pub enum HashInput<'a> {
    Text(&'a str),
    File(&'a str),
}

fn get_inputs(matches: &ArgMatches) -> Vec<HashInput> {
    let text_indices;
    let text_values;
    let file_indices;
    let file_values;

    match matches.indices_of("text") {
        None => {
            text_indices = Vec::new();
            text_values = Vec::new()
        }
        Some(v) => {
            text_indices = v.collect();
            text_values = matches.values_of("text").unwrap().collect();
        }
    }
    match matches.indices_of("file") {
        None => {
            file_indices = Vec::new();
            file_values = Vec::new()
        }
        Some(v) => {
            file_indices = v.collect();
            file_values = matches.values_of("file").unwrap().collect();
        }
    }

    let mut i = 0;
    let mut j = 0;
    let mut inputs = Vec::new();
    while i < text_indices.len() && j < file_indices.len() {
        if text_indices[i] < file_indices[j] {
            inputs.push(HashInput::Text(text_values[i]));
            i += 1;
        } else {
            inputs.push(HashInput::File(file_values[j]));
            j += 1;
        }
    }

    if i < text_indices.len() {
        inputs.extend(text_values[i..].iter().map(|x| HashInput::Text(x)));
    } else if j < file_indices.len() {
        inputs.extend(file_values[j..].iter().map(|x| HashInput::File(x)));
    }

    inputs
}

pub fn compute(matches: &ArgMatches, inputs: &[HashInput]) {
    let algo = if matches.is_present("md5") {
        HashAlgorithm::MD5
    } else if matches.is_present("blake3") {
        HashAlgorithm::BLAKE3
    } else {
        HashAlgorithm::SHA256
    };

    let hex_input = matches.is_present("hex");
    let update_on_input = matches.is_present("update");
    let print_hash_only = matches.is_present("quiet");

    let mut hasher = HashImpl::new();

    for input in inputs.iter() {
        let mut style = OutputStyle::new();
        style.set_algorithm(algo);
        let input_bytes = match input {
            HashInput::Text(text) => {
                if !print_hash_only {
                    style.add_text(text);
                }
                if hex_input {
                    hex_to_byte_slice(text)
                } else {
                    text.as_bytes().to_vec()
                }
            }
            HashInput::File(file) => {
                if !print_hash_only {
                    style.add_file(file);
                }
                if hex_input {
                    match std::fs::read_to_string(file) {
                        Ok(s) => hex_to_byte_slice(&s),
                        Err(err) => {
                            eprintln!("Cannot read file {}: {}", file, err);
                            std::process::exit(exitcode::IOERR);
                        }
                    }
                } else {
                    match std::fs::read(file) {
                        Ok(v) => v,
                        Err(err) => {
                            eprintln!("Cannot read file {}: {}", file, err);
                            std::process::exit(exitcode::IOERR);
                        }
                    }
                }
            }
        };
        if update_on_input {
            hasher.update(&input_bytes);
            let digest = hasher.hex_digest(algo);
            if print_hash_only {
                println!("{}", digest);
            } else {
                style.add_hash(&digest);
                println!("{}", style.summary("UPDATE"));
            }
        } else {
            let digest = HashImpl::hex_digest_input(&input_bytes, algo);
            if print_hash_only {
                println!("{}", digest);
            } else {
                style.add_hash(&digest);
                println!("{}", style.summary("COMPUTE"));
            }
        }
    }
}

fn main() {
    let app = build_app();
    let matches = app.get_matches();

    let inputs = get_inputs(&matches);

    compute(&matches, &inputs);
}
