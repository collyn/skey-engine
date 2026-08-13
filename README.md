# skey-engine

Stateless Vietnamese input method engine written in Rust — used by
[skey](https://github.com/collyn/skey), a fcitx5 Vietnamese input method addon.

## Why

skey only needs a stateless `input → output` transform for Vietnamese text
processing. Most existing engines carry unnecessary state-machine complexity.
skey-engine provides the same functionality in ~700 lines of straightforward Rust,
with a simple Letter-based accumulator model.

## Design

- **Stateless** — `input string → output string`, no internal mutable state
- **Unikey-compatible transforms** — `dd→đ`, `aa→â`, `aw→ă`, `ow→ơ`, `uw→ư`,
  `vaya→vây`, `toiws→tới`, tone replacement, and more
- **String-based `is_valid()`** — CVC validation works on the composed string
  directly, not tied to engine state
- **4 input methods** — Telex, VNI, VIQR, and combined Telex+VNI
- **Thin FFI** — 8 C functions covering lifecycle, config, transform, and validation

## API

```c
// Lifecycle
SkeyEngine *skey_engine_new(int32_t method);        // 0=Telex, 1=VNI, 2=VIQR, 3=TeipVni
void        skey_engine_free(SkeyEngine *e);

// Configuration
void skey_engine_set_method(SkeyEngine *e, int32_t method);
void skey_engine_set_tone_style(SkeyEngine *e, int32_t modern);    // 0=traditional, 1=modern
void skey_engine_set_short_w(SkeyEngine *e, int32_t enabled);      // standalone w→ư
void skey_engine_set_free_marking(SkeyEngine *e, int32_t free);    // reserved
void skey_engine_set_bracket_uo(SkeyEngine *e, int32_t enabled);   // reserved

// Core
char   *skey_engine_transform(SkeyEngine *e, const char *input);
int32_t skey_engine_is_valid(const char *s);
void    skey_free_string(char *s);
```

## Build

```bash
cargo build --release
# → target/release/libskey_engine.a
```

## Release

Bump version and create a tag:

```bash
# 1. Edit version in Cargo.toml
sed -i 's/^version = ".*"/version = "0.2.0"/' Cargo.toml

# 2. Commit & tag
git add Cargo.toml
git commit -m "chore: bump version to 0.2.0"
git tag v0.2.0
git push origin main --tags
```

Pushing a `v*` tag triggers the [release workflow](.github/workflows/release.yml)
which builds `libskey_engine.a`, runs tests, and publishes the artifact to
GitHub Releases.

## License

[GPL-3.0](LICENSE)
