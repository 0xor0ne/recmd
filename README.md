# REmote CoMmanD

`recmd` is a simple utility that can be used to execute command on a remote host.

## Build

```bash
cargo build --release
```

## Usage

On the remote host where the command must be executed run:

```bash
remcd srv -p 22000
```

with the command above `recmd` will listen on port 22000 (TCP) for incoming
command requests.

For sending a command request, run:

```bash
recmd snd -a 1.2.3.4 -p 22000 -c "ls /"
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
