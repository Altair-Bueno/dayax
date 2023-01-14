# Dayax

Simple and experimental scriptable web server, based on Lua, Rust and Axum

## Usage

## Server

```sh
cargo run -- path/to/lua/script
```

The examples directory includes some examples

## Library

dayax isn't available on crates.io at the moment, but you can point Cargo to the
Github repository. Note: the main branch moves fast and isn't stable

```toml
[dependencies.dayax]
git = "https://github.com/Altair-Bueno/dayax"
```

For usage examples, check [dayax-server](crates/dayax-server)'s source

