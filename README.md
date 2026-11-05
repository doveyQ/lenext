# lenext

A length-extension attack against a naive secret-prefix MAC, built from scratch in Rust.

The project implements a small Merkle–Damgård hash and builds a MAC of the form `tag = H(K || M)` on top of it. It then shows how an attacker can forge a valid tag for an extended message without knowing the key, and explains why a sponge-based hash such as Ascon-Hash256 resists the same attack.

> **Educational project.** The toy hash is deliberately insecure and must not be used for anything real. Only run the attack against challenges you created yourself.

## Project layout

| File | Contents |
| --- | --- |
| `src/lib.rs` | Padding, compression function, hash, MAC, forgery, tests |
| `src/main.rs` | Command-line interface: `issue`, `verify`, `forge` |

## Build and test

```bash
cargo build
cargo test
```

The tests cover the padding, the MAC (valid tags verify, altered messages and tags don't) and the forgery across several message lengths, including messages that end exactly on a block boundary.

The only dependency is [`constant_time_eq`](https://docs.rs/constant_time_eq), used for tag comparison.

## Usage

All keys, messages and tags are passed as hex, because a forged message contains zero bytes that can't be typed as a normal argument. To convert text to hex:

```bash
echo -n 'amount=10' | xxd -p
```

| Command | Role | Output |
| --- | --- | --- |
| `lenext issue <key-hex> <msg-hex>` | Server signs a message | Tag (16 hex digits) |
| `lenext verify <key-hex> <msg-hex> <tag-hex>` | Server checks a tag | `true` or `false` |
| `lenext forge <msg-hex> <tag-hex> <key-len> <extra-hex>` | Attacker extends a message, given only the key's length | Forged message (hex), then forged tag |

Invalid input prints an error to stderr and exits with code 2.

### Running the attack

The example below uses the key `0123456789abcdef`, the message `amount=10` and the extra data `&admin=1`:

```bash
KEY=30313233343536373839616263646566
MSG=616d6f756e743d3130
EXTRA=2661646d696e3d31

# 1. The server issues a tag
cargo run -q -- issue $KEY $MSG

# 2. The attacker forges, knowing only the message, the tag and the key length (16)
cargo run -q -- forge $MSG <tag> 16 $EXTRA

# 3. The server accepts the forged message
cargo run -q -- verify $KEY <forged-msg> <forged-tag>
# → true
```

The forged message is `amount=10`, followed by the glue padding (`0x80`, 14 zero bytes, length 200 bits) and then `&admin=1`. The server never signed it, but the tag verifies.

## The toy hash

| Parameter | Value |
| --- | --- |
| State / output | 64 bits (the tag is the entire state) |
| Block size | 16 bytes, read as two big-endian `u64` words |
| Padding | `0x80`, zero bytes, then the message length in bits as a big-endian `u64` |
| Compression | Davies–Meyer: `F(H, M) = E_M(H) ⊕ H` |
| Block cipher `E` | 32 ARX rounds (add, rotate, XOR), message words as alternating round keys |

Each round adds a round key, mixes the state with a rotated copy of itself, and XORs in the round number so no two rounds are identical. The final XOR with the input state (feed-forward) makes the compression step one-way. Without it, anyone could decrypt backwards, since the "key" is the public message block.

The hash is computed as `hash_from(state, data, total_len)`. It pads `data` according to `total_len` and runs each 16-byte block through the compression function. A normal hash starts from a fixed IV with `total_len = data.len()`. The forgery reuses the same function with a different start state and length.

## The attack

A Merkle–Damgård tag is the hash's complete internal state after the last block. Anyone holding the tag can keep hashing from that point. Given `M`, `tag = H(K || M)` and the length of `K` (but not `K` itself), an attacker:

1. Computes the **glue padding** the server appended to `K || M`. It depends only on the length.
2. Sets the hash state to `tag` instead of the IV.
3. Hashes extra data `x` from that state, with final padding based on the **whole** forged length.
4. Submits `M || glue || x` with the new tag. The server computes `H(K || M || glue || x)` and gets the same value.

The length field that makes Merkle–Damgård collision-resistant is also what makes the glue padding predictable.

### Worked example

16-byte key, `M = amount=10` (9 bytes), `x = &admin=1` (8 bytes):

| Stage | Bytes | Contents |
| --- | --- | --- |
| `K \|\| M` | 25 | Hashed by the server |
| Glue padding | 23 | `0x80`, 14 zero bytes, length 200 bits |
| After glue | 48 | Exactly 3 blocks; state here equals `tag` |
| `x` | 8 | Running total: 56 bytes |
| Final padding | 24 | `0x80`, 15 zero bytes, length 448 bits |
| Forged total | 80 | 5 blocks, of which the attacker computes only the last 2 |

The final padding must use the full 56 bytes, not `len(x)`.

## Why a sponge resists it

Ascon-Hash256 splits its 320-bit state into a 64-bit rate and a 256-bit capacity. Output is squeezed from the rate only, so the tag never reveals the full state needed to continue hashing.

The correct fix for a MAC is not a different hash, though. It's a proper construction such as HMAC or KMAC.

## Notes

- With a 64-bit state, generic birthday collisions take about 2³² work. The round count doesn't change that, and it doesn't affect the length-extension attack either.
- Tags are a single `u64`, so the constant-time comparison is for demonstration. It matters for byte-array tags such as HMAC's.

## References

- [NIST SP 800-232: Ascon-Based Lightweight Cryptography Standards](https://csrc.nist.gov/pubs/sp/800/232/final)
- [Ascon reference implementations and test vectors](https://github.com/ascon/ascon-c)