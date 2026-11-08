# b1.7.3-modding-sdk

Everything you need to make mods for [b1.7.3-rust-rewrite](https://github.com/qad3n/b1.7.3-rust-rewrite), the Minecraft Beta 1.7.3 client and server reconstructed and rewritten in Rust. You don't need the game source to build a mod.

The game itself vendors this repo at `vendor/b1.7.3-modding-sdk` and loads mods with its built-in loader (`crates/modloader` in the game repo).

This is a hobby project and a proof of concept. All credit for Minecraft goes to Mojang and its original authors. Not affiliated with Mojang or Microsoft.

## Layout

```
mod_protocol/     wire format, data specs and wasm ABI shared by the game's loader and the SDK
mod_tool/         cargo b173: new, check, build, run
b173_api/         the SDK mods are written against
b173_api_macros/  #[mod_main] and the behavior attribute macros
examples/
  ruby/           data-only mod: blocks, items, recipes, ore generation, a tag
  testmod/        uses every part of the API
  energy_api/     shared API crate for the energy service
  energy/         provides the energy service
  machines/       uses the energy service
```

## Quick start

```bash
rustup target add wasm32-unknown-unknown
```

```bash
cargo install --path mod_tool
```

```bash
B173_SDK=/path/to/b1.7.3-modding-sdk/b173_api cargo b173 new mymod
```

```bash
cargo b173 build --path mymod
```

The result is `mymod/target/mymod-0.1.0.b173mod`. Drop it in the game's `mods/` folder, on the client and on the server.

- `cargo b173 check` checks `mod.toml` and every data file with the loader's own rules.
- `cargo b173 run --client` or `--server` builds the mod, copies it into `run/client/mods` or `run/server/mods` (or `B173_GAME_DIR`, `B173_SERVER_DIR`) and starts `beta173` or `beta173_server` from `PATH` or `B173_BIN_DIR`.

## A mod in short

- `mod.toml` names the mod: `id`, `version`, `api`, `side` (`both`, `client` or `server`), and other mods under `[depends]` or `[optional]`.
- `data/<modid>/` holds content as TOML: `blocks/`, `items/`, `recipes/`, `worldgen/`, `tags/`. Blocks and items copy a vanilla template and change what they need.
- `assets/<modid>/` holds textures (`textures/<path>.png`, named `<modid>:<path>`), sounds and `lang/en_US.lang`.
- `src/lib.rs` is optional code. `#[mod_main]` runs once at load and registers blocks, items, events, commands, keybinds, screens and more. Block, item, entity and tile entity logic are trait impls marked `#[block_behavior]`, `#[item_behavior]`, `#[entity_behavior]` or `#[tile_behavior]`.

Check `ctx.isAuthority()` before you change the world, and check everything a client sends to the server.

## Compatibility

- `api = "major.minor"` in `mod.toml`: a mod built for 1.x loads on every 1.y with y >= x.
- The wasm ABI is versioned (`ABI_VERSION` in `mod_protocol::abi`). Messages only grow by new variants at the end within a major.
- Deprecated SDK items stay for at least one major and warn at build time.

## Building this repo

```bash
cargo clippy --workspace --all-targets
```

```bash
cargo clippy --target wasm32-unknown-unknown --workspace --exclude mod_tool
```

Example textures are not in the repo. Add PNGs under `examples/<mod>/assets/<mod>/textures/` before you run the examples in game.
