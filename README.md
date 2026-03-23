# ryth

Scriptable interface to [iwd](https://iwd.wiki.kernel.org/). Wraps [iwdrs](https://github.com/pythops/iwdrs). All output is JSON.

Built for scripts and status widgets. Not a replacement for `iwctl`. Felt a need for it since quickshell doesn't support iwd as a backend, yet.

> For those who care. Yes. AI-assisted.

## Commands

```
ryth status [--watch]
ryth list [--watch]
ryth scan
ryth connect <ssid> [--password <pw>]
ryth disconnect
ryth autoconnect <ssid> <on|off>
ryth forget <ssid>
ryth known
ryth power <on|off>
```

`--watch` on `status` streams updates on state change. `--watch` on `list` re-outputs after each scan.

## Build

```sh
cargo build --release
```

Docs (man + html):

```sh
cargo run --bin gen-docs
```

## License

Unlicense
