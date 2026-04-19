# nixcfg-rs

rust driver for [nixcfg](https://github.com/mushrowan/nixcfg): emit
JSON Schema from your config struct, consume it as typed NixOS module
options.

```
rust struct ──→ schemars + #[nixcfg] ──→ JSON Schema ──→ nix lib ──→ mkOption
```

## usage

```toml
[dependencies]
nixcfg = "0.3"
schemars = "1"
serde = { version = "1", features = ["derive"] }
```

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
    // one-liner: wraps NixSchema::from + with_defaults + to_json_pretty
    print!("{}", nixcfg::emit::<Config>("myapp"));
}
```

pipe the output into `schema.json` and consume it from nix:

```nix
{
  imports = [ (nixcfg.lib.mkModule { schema = ./schema.json; }) ];
}
# services.myapp.enable
# services.myapp.dataDir       (str, with default)
# services.myapp.listenPort    (types.port)
# services.myapp.apiTokenPath  (path, secret)
```

## attribute macro flags

`#[nixcfg(...)]` goes *above* `#[derive(JsonSchema)]` (it rewrites to
`#[schemars(extend(...))]` so must run first).

| flag / key-value | effect |
|---|---|
| `secret` | marks as secret (`type → path`, name gets `_path` suffix) |
| `port` | integer becomes `types.port` in nix |
| `path` | string becomes `types.path` |
| `skip` | omit from nix module options entirely |
| `description = "..."` | override description (wins over doc comment) |
| `example = value` | override single example |

for types that can't impl `JsonSchema` (foreign crates), use schemars's
`#[schemars(schema_with = "fn")]` to hand-roll the fragment. any
`x-nixcfg-*` keys in the returned JSON pass through untouched.

see the [schema extensions](https://github.com/mushrowan/nixcfg/blob/main/schema/v1.md)
spec and the [nixcfg README](https://github.com/mushrowan/nixcfg#gotchas)
for the complete mapping and common gotchas.

## example

`example-mycel/` is a reference demo with a `Config` struct, a
`Default` impl, and a checked-in `schema.json`. `nix flake check` runs
the binary and diffs its output against the checked-in file to catch
struct-schema drift.

after editing `example-mycel/src/main.rs`, regenerate the reference:

```
cargo x update-schema
```

## development

```
nix develop              # devshell with toolchain + cargo-nextest + cargo-deny
nix flake check          # package build, clippy, tests, deny, doctest, drift check
cargo nextest run        # fast local test loop
cargo x update-schema    # refresh example-mycel/schema.json
```

## license

dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE),
at your option.
