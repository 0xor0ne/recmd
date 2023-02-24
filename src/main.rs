// Copyright 2023 0xor0ne <0xor0ne@gmail.com>
use argh::FromArgs;
use std::fmt;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr};
use std::str;

use recmd::snd::Snd;
use recmd::srv::Srv;

#[derive(FromArgs, PartialEq, Debug)]
/// Top-level command.
struct ReCmdArg {
    #[argh(subcommand)]
    mode: ReCmdModeArg,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum ReCmdModeArg {
    Srv(ReCmdModeSrvArg),
    Snd(ReCmdModeSndArg),
}

#[derive(FromArgs, PartialEq, Debug)]
/// First subcommand.
#[argh(subcommand, name = "srv", description = "server mode")]
struct ReCmdModeSrvArg {
    #[argh(option, short = 'p', default = "3666", description = "listening port")]
    port: u16,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Second subcommand.
#[argh(subcommand, name = "snd", description = "send mode")]
struct ReCmdModeSndArg {
    #[argh(option, short = 'i', description = "server ip")]
    srvip: Ipv4Addr,
    #[argh(option, short = 'p', default = "3666", description = "server port")]
    port: u16,
    #[argh(option, short = 'c', description = "command")]
    cmd: String,
}

#[derive(Debug)]
struct ReCmdError;

impl std::error::Error for ReCmdError {}

impl fmt::Display for ReCmdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TCP error")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: ReCmdArg = argh::from_env();

    match args.mode {
        ReCmdModeArg::Srv(opts) => {
            println!("Server mode {}", opts.port);
            let mut srv = Srv::new(opts.port);
            srv.run()?;
        }
        ReCmdModeArg::Snd(opts) => match opts.cmd.len() {
            0 => {
                println!("Command can not be empty!");
            }
            _ => {
                println!("Send mode {} {} \"{}\"", opts.srvip, opts.port, opts.cmd,);
                let snd = Snd::new(
                    IpAddr::V4(opts.srvip),
                    opts.port,
                    opts.cmd.as_bytes().to_vec(),
                );
                let output = snd.run()?;

                match str::from_utf8(&output) {
                    Ok(s) => {
                        println!("{}", s);
                    }
                    _ => {
                        std::io::stdout().write_all(&output)?;
                    }
                }
            }
        },
    }

    Ok(())
}
