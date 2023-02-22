use bytes::BytesMut;
use sha2::{
    digest::crypto_common::generic_array::{typenum::U32, GenericArray},
    Digest, Sha256,
};
use std::boxed::Box;
use std::fmt;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::cmd::Cmd;
use crate::config::Config;
use crate::crypt::Crypt;
use crate::message::{Message, MessageError, ReCmdMsg, ReCmdMsgPayload, ReCmdMsgType};

pub struct Srv {
    port: u16,
    config: Arc<Mutex<Config>>,
    history: Arc<Mutex<Vec<GenericArray<u8, U32>>>>,
}

#[derive(Debug)]
pub enum SrvError {
    TcpError,
}

impl std::error::Error for SrvError {}

impl fmt::Display for SrvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SrvError::TcpError => write!(f, "TCP error"),
        }
    }
}

impl From<std::io::Error> for SrvError {
    fn from(_e: std::io::Error) -> Self {
        SrvError::TcpError
    }
}

impl Srv {
    pub fn new(port: u16) -> Self {
        Srv {
            port,
            config: Arc::new(Mutex::new(Config::init())),
            history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn run(&mut self) -> Result<(), SrvError> {
        let listener = TcpListener::bind(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            self.port,
        ))?;

        println!("Server listening on port {}", self.port);
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection from {}", stream.peer_addr().unwrap());
                    let c = Arc::clone(&self.config);
                    let h = Arc::clone(&self.history);
                    thread::spawn(move || Srv::handle_connection(stream, c, h));
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            };
        }

        drop(listener);

        Ok(())
    }

    fn handle_connection(
        mut stream: TcpStream,
        config: Arc<Mutex<Config>>,
        history: Arc<Mutex<Vec<GenericArray<u8, U32>>>>,
    ) -> Result<(), std::io::Error> {
        let mut data = Vec::new();

        match stream.read_to_end(&mut data) {
            Ok(n) if n > 0 => {
                match config.lock() {
                    Ok(c) => {
                        stream.set_write_timeout(Some(c.get_tcp_write_to()))?;

                        let hash = Srv::get_data_sha256(&data);

                        // Just ignore messages already in history
                        if let Some(_) = Srv::exist_in_history(&history, &hash) {
                            let msg_dec: ReCmdMsg = Srv::deserialize_decrypt(c.get_key(), &data)?;

                            // Update history
                            Srv::update_history(&history, hash, c.get_history_depth())?;

                            // Consider only direct command req ignore silently all the rest
                            if msg_dec.hdr.msg_type == ReCmdMsgType::DirectCmdReq {
                                let cmdo = Srv::execute_command(&msg_dec)?;

                                let data_to_send =
                                    Srv::encrypt_serialize(c.get_key(), &msg_dec, &cmdo)?;

                                stream.write_all(&data_to_send)?;
                            }
                        }

                        Ok(())
                    }
                    Err(_) => Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Config lock failed",
                    )),
                }
            }
            Ok(_) => {
                stream.shutdown(Shutdown::Both).unwrap();
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "No bytes on TCP stream",
                ))
            }
            Err(e) => {
                stream.shutdown(Shutdown::Both).unwrap();
                Err(e)
            }
        }
    }

    fn execute_command(msg: &ReCmdMsg) -> Result<Vec<u8>, std::io::Error> {
        if let ReCmdMsgPayload::DirectCmdReq {
            ts: _,
            m_len: _,
            m: m_dec,
        } = &msg.payload
        {
            let c = Cmd::new(&String::from_utf8_lossy(&m_dec));
            match c.run() {
                Ok(o) => Ok(o),
                Err(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "Err")),
            }
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Wrong msg type",
            ))
        }
    }

    fn encrypt_serialize(
        key: GenericArray<u8, U32>,
        req: &ReCmdMsg,
        data: &Vec<u8>,
    ) -> Result<Vec<u8>, std::io::Error> {
        if let ReCmdMsgPayload::DirectCmdReq {
            ts: ts_dec,
            m_len: _,
            m: _,
        } = req.payload
        {
            let cipher = Box::new(Crypt::new(key));
            let msg = Message::new(cipher);
            if let Ok(msg_enc) = msg.encrypt_serialize(ReCmdMsgType::DirectCmdRes, data, ts_dec) {
                Ok(msg_enc.to_vec())
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Encrypt error",
                ))
            }
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Wrong msg type",
            ))
        }
    }

    fn deserialize_decrypt(
        key: GenericArray<u8, U32>,
        data: &Vec<u8>,
    ) -> Result<ReCmdMsg, MessageError> {
        let cipher = Box::new(Crypt::new(key));
        let msg = Message::new(cipher);
        let mut msg_enc = BytesMut::with_capacity(0);
        msg_enc.extend_from_slice(&data);
        msg.deserialize_decrypt(&msg_enc.to_vec())
    }

    fn get_data_sha256(data: &Vec<u8>) -> GenericArray<u8, U32> {
        let mut hasher = Sha256::new();
        hasher.update(&data);
        hasher.finalize()
    }

    fn exist_in_history(
        history: &Arc<Mutex<Vec<GenericArray<u8, U32>>>>,
        hash: &GenericArray<u8, U32>,
    ) -> Option<()> {
        match history.lock() {
            Ok(h) => {
                if h.contains(hash) {
                    Some(())
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    fn update_history(
        history: &Arc<Mutex<Vec<GenericArray<u8, U32>>>>,
        hash: GenericArray<u8, U32>,
        history_depth: usize,
    ) -> Result<(), std::io::Error> {
        match history.lock() {
            Ok(mut h) => {
                h.insert(0, hash);

                if h.len() > history_depth {
                    h.pop();
                }

                Ok(())
            }
            Err(_) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Config lock failed",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Srv;
    use crate::message::{Message, ReCmdMsg, ReCmdMsgHdr, ReCmdMsgPayload, ReCmdMsgType};
    use crate::{config::Config, crypt::Crypt};
    use bytes::BytesMut;
    use hex;
    use sha2::digest::crypto_common::generic_array::{typenum::U32, GenericArray};
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn check_sha256() {
        let data: Vec<u8> = b"a".to_vec();
        let data_sha256_expected = GenericArray::clone_from_slice(
            &hex::decode("ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb")
                .unwrap(),
        );

        let data_sha256 = Srv::get_data_sha256(&data);

        assert_eq!(data_sha256_expected, data_sha256);
    }

    #[test]
    fn exist_in_history_true() {
        let data0: Vec<u8> = b"0".to_vec();
        let data1: Vec<u8> = b"1".to_vec();
        let data2: Vec<u8> = b"2".to_vec();

        let data0_sha256 = Srv::get_data_sha256(&data0);
        let data1_sha256 = Srv::get_data_sha256(&data1);
        let data2_sha256 = Srv::get_data_sha256(&data2);

        let history_depth = 4;
        let history = Arc::new(Mutex::new(Vec::<GenericArray<u8, U32>>::new()));

        Srv::update_history(&history, data0_sha256, history_depth).unwrap();
        Srv::update_history(&history, data1_sha256, history_depth).unwrap();
        Srv::update_history(&history, data2_sha256, history_depth).unwrap();

        assert_eq!(Some(()), Srv::exist_in_history(&history, &data1_sha256));
    }

    #[test]
    fn exist_in_history_false() {
        let data0: Vec<u8> = b"0".to_vec();
        let data1: Vec<u8> = b"1".to_vec();
        let data2: Vec<u8> = b"2".to_vec();
        let data3: Vec<u8> = b"3".to_vec();

        let data0_sha256 = Srv::get_data_sha256(&data0);
        let data1_sha256 = Srv::get_data_sha256(&data1);
        let data2_sha256 = Srv::get_data_sha256(&data2);
        let data3_sha256 = Srv::get_data_sha256(&data3);

        let history_depth = 4;
        let history = Arc::new(Mutex::new(Vec::<GenericArray<u8, U32>>::new()));

        Srv::update_history(&history, data0_sha256, history_depth).unwrap();
        Srv::update_history(&history, data1_sha256, history_depth).unwrap();
        Srv::update_history(&history, data2_sha256, history_depth).unwrap();

        assert_eq!(None, Srv::exist_in_history(&history, &data3_sha256));
    }

    #[test]
    fn exist_in_history_saturation_false() {
        let data0: Vec<u8> = b"0".to_vec();
        let data1: Vec<u8> = b"1".to_vec();
        let data2: Vec<u8> = b"2".to_vec();
        let data3: Vec<u8> = b"3".to_vec();
        let data4: Vec<u8> = b"4".to_vec();

        let data0_sha256 = Srv::get_data_sha256(&data0);
        let data1_sha256 = Srv::get_data_sha256(&data1);
        let data2_sha256 = Srv::get_data_sha256(&data2);
        let data3_sha256 = Srv::get_data_sha256(&data3);
        let data4_sha256 = Srv::get_data_sha256(&data4);

        let history_depth = 4;
        let history = Arc::new(Mutex::new(Vec::<GenericArray<u8, U32>>::new()));

        Srv::update_history(&history, data0_sha256, history_depth).unwrap();
        Srv::update_history(&history, data1_sha256, history_depth).unwrap();
        Srv::update_history(&history, data2_sha256, history_depth).unwrap();
        Srv::update_history(&history, data3_sha256, history_depth).unwrap();
        Srv::update_history(&history, data4_sha256, history_depth).unwrap();

        assert_eq!(None, Srv::exist_in_history(&history, &data0_sha256));
        assert_eq!(Some(()), Srv::exist_in_history(&history, &data1_sha256));
    }

    fn deserialize_decrypt(corrupt: bool) {
        let cmds = "ls -al";
        let cfg = Config::init();
        let key = cfg.get_key();
        // Create cipher
        let cipher = Box::new(Crypt::new(key));
        // Create command string
        let m = String::from(cmds);
        let m_bytes = m.into_bytes();
        // Create, encrypt and serialize message
        let msg = Message::new(cipher);
        // Timestamp
        let ts: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut msg_enc: BytesMut = msg
            .encrypt_serialize(ReCmdMsgType::DirectCmdReq, &m_bytes, ts)
            .unwrap();
        let msg_dec: ReCmdMsg = Srv::deserialize_decrypt(key, &msg_enc.to_vec()).unwrap();

        if corrupt {
            // 1 byte for type
            // 4 bytes for length
            // 24 bytes for nonce
            match msg_enc[30] {
                255 => {
                    msg_enc[30] = 0;
                }
                _ => {
                    msg_enc[30] = msg_enc[30] + 1;
                }
            }

            match msg.deserialize_decrypt(&msg_enc.to_vec()) {
                Err(_) => assert!(true),
                _ => assert!(false),
            }
        } else {
            if let ReCmdMsgPayload::DirectCmdReq {
                ts: ts_dec,
                m_len: m_len_dec,
                m: m_dec,
            } = msg_dec.payload
            {
                assert_eq!(ts, ts_dec);
                assert_eq!(m_bytes.len(), m_len_dec as usize);
                assert_eq!(m_bytes, m_dec);
            } else {
                assert!(false);
            }
        }
    }

    #[test]
    fn deserialize_decrypt_ok() {
        deserialize_decrypt(false);
    }

    #[test]
    fn deserialize_decrypt_corrupt() {
        deserialize_decrypt(true);
    }

    #[test]
    fn run_command() {
        let cmds = "echo \"test\"";
        let cfg = Config::init();
        let key = cfg.get_key();
        // Create cipher
        let cipher = Box::new(Crypt::new(key));
        // Create command string
        let m = String::from(cmds);
        // Create, encrypt and serialize message
        let msg = Message::new(cipher);
        // Timestamp
        let ts: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let msg_enc: BytesMut = msg
            .encrypt_serialize(ReCmdMsgType::DirectCmdReq, &m.into_bytes(), ts)
            .unwrap();
        let msg_dec: ReCmdMsg = Srv::deserialize_decrypt(key, &msg_enc.to_vec()).unwrap();

        let cmdo = Srv::execute_command(&msg_dec).unwrap();
        let ostr = String::from_utf8_lossy(&cmdo);
        assert_eq!("test\n", ostr);
    }

    #[test]
    fn encrypt_serialize_ok() {
        let cfg = Config::init();
        let key = cfg.get_key();
        let ts: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let data = "home".as_bytes().to_vec();

        let req: ReCmdMsg = ReCmdMsg {
            hdr: ReCmdMsgHdr {
                msg_type: ReCmdMsgType::DirectCmdReq,
                len: 14,
                nonce: GenericArray::clone_from_slice(&[0; 24]),
            },
            payload: {
                ReCmdMsgPayload::DirectCmdReq {
                    ts,
                    m_len: 6,
                    m: "ls -la".as_bytes().to_vec(),
                }
            },
        };
        //fn encrypt_serialize(
        //    key: GenericArray<u8, U32>,
        //    req: &ReCmdMsg,
        //    data: &Vec<u8>,
        //) -> Result<Vec<u8>, std::io::Error> {

        let data_to_send = Srv::encrypt_serialize(key, &req, &data).unwrap();
        let cipher = Box::new(Crypt::new(key));
        let msg = Message::new(cipher);
        let msg_dec: ReCmdMsg = msg.deserialize_decrypt(&data_to_send).unwrap();

        if let ReCmdMsgPayload::DirectCmdRes {
            ts: ts_dec,
            m_len: m_len_dec,
            m: m_dec,
        } = msg_dec.payload
        {
            assert_eq!(ts, ts_dec);
            assert_eq!(data.len(), m_len_dec as usize);
            assert_eq!(data, m_dec);
        } else {
            assert!(false);
        }
    }
}
