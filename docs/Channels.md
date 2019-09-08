# Channels

This document is a working specification for the TieDye synthetic channels, we provide no warranty that anything
written here is the case or that it will remain the case.

## Channel Message

The interface for the `close_channel` function in the Channel module looks like the below:

```rust
pub fn close_channel(
    origin,
    message: Vec<u8>,       // Arbitrary length bytes representing the state of the channel.
    public_keys: Vec<u8>,   // 64-bytes;  It is the two parties' public keys appended to each in any order.
    signatures: Vec<u8>     // 128-bytes; The two signatures appended to each other, in the same logical order as the keys.
) -> Result {}
```

We address now how the bytes in the message will be deserialized. 

- The first 6 bytes are the channel id. This gives us `2**48` or `281474976710656` possible concurrent channels.
- Following will be 2 bytes appended to 256 bytes times two for each asset being traded in the synthetic channel:
  this represents the 2-bytes `OracleId`, a `i128` for an amount of that asset that can be either positive or negative,
  repeated twice to represent each member of this channel.
