# nixcfg

rust bridge for [nixcfg](https://github.com/mushrowan/nixcfg): derive
JSON Schema from your config struct, consume it as typed NixOS module
options in downstream nix.

```rust
use nixcfg::{JsonSchema, nixcfg};
use serde::Serialize;

/// my service configuration
#[nixcfg]
#[derive(JsonSchema, Serialize, Default)]
struct Config {
    /// data directory
    data_dir: String,

    /// listen port
    #[nixcfg(port)]
    listen_port: u16,

    /// API authentication token
    #[nixcfg(secret)]
    api_token: String,
}

fn main() {
    print!("{}", nixcfg::emit::<Config>("myapp"));
}
```

see the [nixcfg-rs README](https://github.com/mushrowan/nixcfg-rs) for
the full attribute reference and examples, and the [nixcfg schema
spec](https://github.com/mushrowan/nixcfg/blob/main/schema/v1.md) for
the set of `x-nixcfg-*` extensions this library produces.

## license

dual-licensed under MIT or Apache-2.0, at your option.
