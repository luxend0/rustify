# Rustify

A fast, lightweight Spotify client for Linux, written in Rust.

The official client is a Chromium app that needs 400-600 MB of RAM and takes seconds to start. Rustify targets:

| Budget | Target |
|---|---|
| First frame | < 100 ms |
| Memory (RSS) | < 40 MB |
| Idle CPU | 0 % |

Built on [librespot](https://github.com/librespot-org/librespot) for playback and [Slint](https://slint.dev) with its software renderer for the UI.

## Status

Early development. Current stage: W0, a probe that checks whether librespot can play audio for your account.

```sh
cargo run --release --example probe
```

## Requirements

- Spotify Premium.
- An account that librespot can still play. Accounts created after an unknown cutoff (around 2024/25) no longer receive audio keys, see [librespot#1649](https://github.com/librespot-org/librespot/issues/1649). For those accounts Rustify will act as a Spotify Connect remote instead.
- PulseAudio or PipeWire (pipewire-pulse).

## Not in scope

Rustify does not download tracks to files and does not circumvent DRM. It streams what your Premium account can stream.

## License

GPL-3.0-only. Rustify is not affiliated with or endorsed by Spotify.
