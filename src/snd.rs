//! Sender module.
//!
//! This module is in charge to send the command to a server and way the response

use crate::message::{
    Message, MessageError, ReCmdMsg, ReCmdMsgPayload, ReCmdMsgType, HDR_LEN_ON_WIRE,
};
use bytes::BytesMut;
use std::fmt;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Config;
use crate::crypt::Crypt;

#[derive(Debug)]
pub struct Snd {
    srv_ip: IpAddr,
    port: u16,
    data: Vec<u8>,
    config: Config,
}

#[derive(Debug)]
pub enum SndError {
    TcpError,
}

impl std::error::Error for SndError {}

impl fmt::Display for SndError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SndError::TcpError => write!(f, "TCP error"),
        }
    }
}

impl From<std::io::Error> for SndError {
    fn from(_e: std::io::Error) -> Self {
        SndError::TcpError
    }
}

impl From<MessageError> for SndError {
    fn from(_e: MessageError) -> Self {
        SndError::TcpError
    }
}

impl Snd {
    pub fn new(srv_ip: IpAddr, port: u16, data: Vec<u8>) -> Self {
        Snd {
            srv_ip,
            port,
            data,
            config: Config::init(),
        }
    }

    pub fn run(&self) -> Result<Vec<u8>, SndError> {
        match TcpStream::connect_timeout(
            &SocketAddr::new(self.srv_ip, self.port),
            self.config.get_tcp_connect_to(),
        ) {
            Ok(mut stream) => {
                stream.set_write_timeout(Some(self.config.get_tcp_write_to()))?;
                stream.set_read_timeout(Some(self.config.get_tcp_resp_to()))?;

                let ts: u64 = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let data_to_send = self.encrypt_serialize(ts)?;
                stream.write_all(&data_to_send)?;
                let mut data_res = Vec::new();
                Snd::read_message(&mut stream, &mut data_res)?;
                let msg_dec: ReCmdMsg = self.deserialize_decrypt(&data_res)?;
                drop(stream);

                if msg_dec.hdr.msg_type == ReCmdMsgType::DirectCmdRes {
                    if let ReCmdMsgPayload::DirectCmdRes {
                        ts: ts_dec,
                        m: m_dec,
                        ..
                    } = &msg_dec.payload
                    {
                        if ts == *ts_dec {
                            Ok(m_dec.to_vec())
                        } else {
                            Err(SndError::TcpError)
                        }
                    } else {
                        Err(SndError::TcpError)
                    }
                } else {
                    Err(SndError::TcpError)
                }
            }
            Err(_) => Err(SndError::TcpError),
        }
    }

    fn read_message(stream: &mut TcpStream, buf: &mut Vec<u8>) -> Result<usize, std::io::Error> {
        let mut hdrdata = [0u8; HDR_LEN_ON_WIRE];
        stream.read_exact(&mut hdrdata)?;

        match Message::parse_hdr(&hdrdata) {
            Ok((_, (_, len, _))) => {
                let len = len.try_into();

                match len {
                    Ok(len) => {
                        let mut payloaddata: Vec<u8> = vec![0; len];
                        let npayload = stream.read(&mut payloaddata)?;

                        if npayload == len {
                            buf.append(&mut hdrdata.to_vec());
                            buf.append(&mut payloaddata);
                            Ok(HDR_LEN_ON_WIRE + npayload)
                        } else {
                            Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Payload too short",
                            ))
                        }
                    }
                    _ => Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Conversion error",
                    )),
                }
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Hdr decoding error",
            )),
        }
    }

    fn deserialize_decrypt(&self, data: &[u8]) -> Result<ReCmdMsg, MessageError> {
        let cipher = Box::new(Crypt::new(self.config.get_key()));
        let msg = Message::new(cipher);
        let mut msg_enc = BytesMut::with_capacity(0);
        msg_enc.extend_from_slice(data);
        msg.deserialize_decrypt(&msg_enc)
    }

    fn encrypt_serialize(&self, ts: u64) -> Result<Vec<u8>, std::io::Error> {
        let cipher = Box::new(Crypt::new(self.config.get_key()));
        let msg = Message::new(cipher);
        if let Ok(msg_enc) = msg.encrypt_serialize(ReCmdMsgType::DirectCmdReq, &self.data, ts) {
            Ok(msg_enc.to_vec())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Encrypt error",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Snd;
    use crate::message::{ReCmdMsg, ReCmdMsgPayload, ReCmdMsgType};
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn encrypt_serialize_deserialize_decrypt() {
        let cmd_str = "echo test";
        let ts: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let snd = Snd::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            6666u16,
            cmd_str.as_bytes().to_vec(),
        );

        let data = snd.encrypt_serialize(ts).unwrap();
        let msg_dec: ReCmdMsg = snd.deserialize_decrypt(&data).unwrap();

        assert_eq!(msg_dec.hdr.msg_type, ReCmdMsgType::DirectCmdReq);
        if let ReCmdMsgPayload::DirectCmdReq { ts: ts_dec, .. } = &msg_dec.payload {
            assert_eq!(ts, *ts_dec);
        }
    }
}
