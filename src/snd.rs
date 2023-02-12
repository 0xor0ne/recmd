use std::io::Write;
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::time::Duration;

#[derive(Debug)]
pub struct Snd {
    srv_ip: IpAddr,
    port: u16,
    timeout: Duration,
}

impl Snd {
    pub fn new(srv_ip: IpAddr, port: u16, to_ms: u64) -> Self {
        Snd {
            srv_ip,
            port,
            timeout: Duration::from_millis(to_ms),
        }
    }

    pub fn run(&mut self) {
        self.send_simple().unwrap();
    }

    fn send_simple(&self) -> std::io::Result<usize> {
        match TcpStream::connect_timeout(&SocketAddr::new(self.srv_ip, self.port), self.timeout) {
            Ok(mut stream) => {
                stream.set_write_timeout(Some(self.timeout))?;
                let wbn = stream.write(b"Hello!");
                drop(stream);
                wbn
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
                Err(e)
            }
        }
    }
}
