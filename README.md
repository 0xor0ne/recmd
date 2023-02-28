# REmote CoMmanD Executor

`recmd` is a simple utility that can be used to execute command on a remote host.

## Build

```bash
cargo build --release
```

For building using Docker or for static builds, see [here](./docs/build-docker.md)

## Usage

On the remote host where the command must be executed run `recmd` in server mode:

```bash
remcd srv -p 22000
```

If you prefer to run the server in background, you can enable the daemon mode by
using the option `-d`:

```bash
remcd srv -p 22000 -d
```

with the commands above `recmd` will listen on port 22000 (TCP) for incoming
command requests.

For sending a command request, run `recmd` in send mode (change `1.2.3.4` with
the IP address of the host where the server is running):

```bash
recmd snd -a 1.2.3.4 -p 22000 -c "bash -c 'ls /etc > /tmp/out'"
recmd snd -a 1.2.3.4 -p 22000 -c "bash -c 'cat /tmp/out'"
```

## Encryption Key

Communications between the client and the server are encrypted and authenticated
with `ChaCha20-Poly1305`. `recmd` uses a static password (used to derive the 256
bits `ChaCha20` key) defined in `src/config.rs` with the variable
`PASSWORD_DEF`.

```rust
const PASSWORD_DEF: &str = "1e$tob5UtRi6oFr8jlYO";
```

Be sure to change the value of this variable with your own password before
building your own `recmd` executable.

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
