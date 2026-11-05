use std::env;
use std::process;

use lenext::{forge, mac, verify_mac};

const USAGE: &str = "usage:
  lenext issue  <key-hex> <msg-hex>
  lenext verify <key-hex> <msg-hex> <tag-hex>
  lenext forge  <msg-hex> <tag-hex> <key-len> <extra-hex>";

fn main() {
    let args: Vec<String> = env::args().collect();
    let args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    match args.as_slice() {
        [_, "issue", key, msg] => {
            let key = parse_hex(key, "key");
            let msg = parse_hex(msg, "message");
            println!("{:016x}", mac(&key, &msg));
        }
        [_, "verify", key, msg, tag] => {
            let key = parse_hex(key, "key");
            let msg = parse_hex(msg, "message");
            let tag = parse_tag(tag);
            println!("{}", verify_mac(&key, &msg, tag));
        }
        [_, "forge", msg, tag, key_len, extra] => {
            let msg = parse_hex(msg, "message");
            let tag = parse_tag(tag);
            let key_len: usize = key_len
                .parse()
                .unwrap_or_else(|_| fail("key-len must be a number of bytes"));
            let extra = parse_hex(extra, "extra");

            let (forged_msg, forged_tag) = forge(&msg, tag, key_len, &extra);
            println!("{}", to_hex(&forged_msg));
            println!("{:016x}", forged_tag);
        }
        _ => fail(USAGE),
    }
}

/// Decodes a hex string into bytes. Exits on invalid input
fn parse_hex(s: &str, what: &str) -> Vec<u8> {
    if !s.is_ascii() || s.len() % 2 != 0 {
        fail(&format!("{what} must be an even number of hex digits"));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .unwrap_or_else(|_| fail(&format!("{what} contains a non-hex character")))
        })
        .collect()
}

/// Parses a tag from hex. Exits on invalid input
fn parse_tag(s: &str) -> u64 {
    u64::from_str_radix(s, 16).unwrap_or_else(|_| fail("tag must be up to 16 hex digits"))
}

/// Encodes bytes as a lowercase hex string
fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Prints an error to stderr and exits with code 2
fn fail(msg: &str) -> ! {
    eprintln!("{msg}");
    process::exit(2);
}