use recmd::{snd, srv};
use std::net::{IpAddr, Ipv4Addr};
use std::thread;

fn handle_srv(port: u16) {
    let mut srv = srv::Srv::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
    srv.run().unwrap();
}

#[test]
fn localhost_command() {
    thread::spawn(|| handle_srv(6666));
    thread::sleep(std::time::Duration::from_millis(500));

    let cmd = "echo -n \"test\"";
    let cmd_bytes = cmd.as_bytes().to_vec();
    let snd = snd::Snd::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6666, cmd_bytes);
    match snd.run() {
        Ok(m) => {
            assert_eq!("test", String::from_utf8_lossy(&m));
        }
        _ => {
            assert!(false);
        }
    }
}

#[test]
fn localhost_command_bytes() {
    thread::spawn(|| handle_srv(6666));
    thread::sleep(std::time::Duration::from_millis(500));

    let cmd = "bash -c 'echo -e \"\\x00\\xaa\"'";
    let cmd_bytes = cmd.as_bytes().to_vec();
    let snd = snd::Snd::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6666, cmd_bytes);
    let expected = vec![0x00, 0xaa, 0xa];
    match snd.run() {
        Ok(m) => {
            assert_eq!(expected, m);
        }
        _ => {
            assert!(false);
        }
    }
}
