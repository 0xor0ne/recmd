// Copyright 2023 0xor0ne <0xor0ne@gmail.com>
use argh::FromArgs;
use std::net::{IpAddr, Ipv4Addr};

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
}

fn main() {
    let args: ReCmdArg = argh::from_env();

    match args.mode {
        ReCmdModeArg::Srv(opts) => {
            println!("Server mode {}", opts.port);
            let mut srv = Srv::new(opts.port);
            srv.run();
        }
        ReCmdModeArg::Snd(opts) => {
            println!("Send mode {} {}", opts.srvip, opts.port);
            let mut snd = Snd::new(IpAddr::V4(opts.srvip), opts.port, 1000);
            snd.run();
        }
    }
}
