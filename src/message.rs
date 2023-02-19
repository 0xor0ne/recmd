//! ReCmd Messages module
//!
//! Handles serialization/deserialization and encryption/decryption of messages
//!
//! Each message is composed by an header followed by a payload.
//! On wire the header is composed by:
//! - type (1 byte)
//!   - Direct Command Request: from client to server for requesting a command execution
//!   - Direct Command Response: from server to client containing the command output
//! - payload length (4 byte in big endian order)
//! - 24 bytes nonce (XChaCha20Poly1305)
//!
//! The payload is transmitted as a sequence of bytes.
//! In the clear text form the payload is composed by:
//! - Direct Command Request message:
//!   - timestamp (8 bytes in big endian form)
//!   - command (utf-8 string)
//! - Direct Command Response message:
//!   - timestamp (must match the one used in the request)
//!   - command output (utf-8 string)

use crate::crypt::Crypt;
use byteorder::{BigEndian, ByteOrder, WriteBytesExt};
use bytes::{buf::BufMut, BytesMut};
use chacha20poly1305::{
    aead::generic_array::{typenum::U24, GenericArray},
    XNonce,
};
use nom::{
    bytes::complete::take,
    combinator::{map_opt, map_res},
    number::complete::be_u8,
    IResult,
};
use std::fmt;
use std::io::Write;
use std::rc::Rc;

/// Message types:
/// - DirectCmdReq: request remote command
/// - DirectCmdRes: response to DirectCmdReq
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum ReCmdMsgType {
    DirectCmdReq = 0,
    DirectCmdRes = 1,
    Unknown(u8),
}

/// ReCmd Message header:
#[derive(Debug)]
struct ReCmdMsgHdr {
    msg_type: ReCmdMsgType,
    len: u32,
    nonce: XNonce,
}

#[derive(Debug)]
enum ReCmdMsgPayload {
    DirectCmdReq { ts: u64, m_len: u32, m: String },
    DirectCmdRes { ts: u64, m_len: u32, m: String },
}

#[derive(Debug)]
pub struct ReCmdMsg {
    hdr: ReCmdMsgHdr,
    payload: ReCmdMsgPayload,
}

#[derive(Debug)]
pub enum MessageError {
    ParseError,
    CryptError,
}

impl std::error::Error for MessageError {}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MessageError::ParseError => write!(f, "Parsing error"),
            MessageError::CryptError => write!(f, "Crypt error"),
        }
    }
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for MessageError {
    fn from(e: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        MessageError::ParseError
    }
}

pub struct Message {
    cipher: Rc<Crypt>,
}

impl From<u8> for ReCmdMsgType {
    fn from(t: u8) -> Self {
        match t {
            0 => ReCmdMsgType::DirectCmdReq,
            1 => ReCmdMsgType::DirectCmdRes,
            n => ReCmdMsgType::Unknown(n),
        }
    }
}

impl From<ReCmdMsgType> for u8 {
    fn from(t: ReCmdMsgType) -> u8 {
        match t {
            ReCmdMsgType::DirectCmdReq => 0,
            ReCmdMsgType::DirectCmdRes => 1,
            ReCmdMsgType::Unknown(n) => n,
        }
    }
}

impl Message {
    pub fn new(cipher: Rc<Crypt>) -> Self {
        Message { cipher }
    }

    pub fn encrypt_serialize(
        &self,
        t: ReCmdMsgType,
        m: &str,
        ts: u64,
    ) -> Result<BytesMut, MessageError> {
        let mut b = BytesMut::new();

        // Create temporary BytesMut for payload
        let mut b_payload_clear = vec![];

        b_payload_clear.write_u64::<BigEndian>(ts).unwrap();
        b_payload_clear
            .write_u32::<BigEndian>(m.len().try_into().unwrap())
            .unwrap();
        b_payload_clear.write(m.as_bytes()).unwrap();
        if let Ok((nonce, b_payload_enc)) = self.cipher.encrypt(&b_payload_clear) {
            b.put_u8(t.into());
            let payload_len = b_payload_enc.len() as u32;
            let mut tmp = [0; 4];
            BigEndian::write_u32(&mut tmp, payload_len);
            b.extend_from_slice(&tmp);
            b.put_slice(&nonce);
            b.put_slice(&b_payload_enc);
        } else {
            return Err(MessageError::CryptError);
        }

        Ok(b)
    }

    fn type_from_u8(t: u8) -> Option<ReCmdMsgType> {
        Some(t.into())
    }

    fn parse_type(input: &[u8]) -> IResult<&[u8], ReCmdMsgType> {
        map_opt(be_u8, Self::type_from_u8)(input)
    }

    fn parse_length(input: &[u8]) -> IResult<&[u8], u32> {
        let (i, obe) = take(std::mem::size_of::<u32>())(input)?;
        let one = BigEndian::read_u32(obe);

        Ok((i, one))
    }

    fn nonce_from_slice(s: &[u8]) -> Option<XNonce> {
        Some(GenericArray::<u8, U24>::clone_from_slice(s))
    }

    fn parse_nonce(input: &[u8]) -> IResult<&[u8], XNonce> {
        map_opt(take(24usize), Self::nonce_from_slice)(input)
    }

    fn payload_from_slice(p: &[u8]) -> Option<Vec<u8>> {
        Some(p.to_vec())
    }

    fn parse_payload_enc(input: &[u8], len: usize) -> IResult<&[u8], Vec<u8>> {
        map_opt(take(len), Self::payload_from_slice)(input)
    }

    fn parse_ts(input: &[u8]) -> IResult<&[u8], u64> {
        let (i, tsbe) = take(std::mem::size_of::<u64>())(input)?;
        let tsne = BigEndian::read_u64(tsbe);

        Ok((i, tsne))
    }

    fn parse_m_len(input: &[u8]) -> IResult<&[u8], u32> {
        let (i, obe) = take(std::mem::size_of::<u32>())(input)?;
        let one = BigEndian::read_u32(obe);

        Ok((i, one))
    }

    fn parse_m(input: &[u8], len: usize) -> IResult<&[u8], &str> {
        map_res(take(len), |s| std::str::from_utf8(s))(input)
    }

    fn parse_payload_dec(input: &[u8]) -> IResult<&[u8], (u64, u32, String)> {
        let (i, ts) = Self::parse_ts(input)?;
        let (i, m_len) = Self::parse_m_len(i)?;
        let (i, m) = Self::parse_m(i, m_len.try_into().unwrap())?;
        Ok((i, (ts, m_len, String::from(m))))
    }

    pub fn deserialize_decrypt(&self, data: BytesMut) -> Result<ReCmdMsg, MessageError> {
        // parse type
        let (i, t) = Self::parse_type(&data)?;
        // parse len
        let (i, len) = Self::parse_length(i)?;
        // parse nonce
        let (i, nonce) = Self::parse_nonce(i)?;
        // parse payload enc
        let (i, payload_enc) = Self::parse_payload_enc(i, usize::try_from(len).unwrap())?;
        // decrypt
        if let Ok(payload_dec) = self.cipher.decrypt(nonce, &payload_enc) {
            let (_i, (ts, m_len, m)) = Self::parse_payload_dec(&payload_dec)?;

            match t {
                ReCmdMsgType::DirectCmdReq => Ok(ReCmdMsg {
                    hdr: ReCmdMsgHdr {
                        msg_type: t.into(),
                        len: len,
                        nonce: nonce,
                    },
                    payload: ReCmdMsgPayload::DirectCmdReq { ts, m_len, m },
                }),
                ReCmdMsgType::DirectCmdRes => Ok(ReCmdMsg {
                    hdr: ReCmdMsgHdr {
                        msg_type: t.into(),
                        len: len,
                        nonce: nonce,
                    },
                    payload: ReCmdMsgPayload::DirectCmdRes { ts, m_len, m },
                }),
                _ => Err(MessageError::ParseError),
            }
        } else {
            return Err(MessageError::CryptError);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Message, ReCmdMsg, ReCmdMsgPayload, ReCmdMsgType};
    use crate::{config::Config, crypt::Crypt};
    use bytes::BytesMut;
    use std::rc::Rc;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn encrypt_serialize_deserialize_decrypt(t: ReCmdMsgType, m_orig: &str, corrupt: bool) {
        // Build key
        let cfg = Config::init();
        let key = cfg.get_key();
        // Create cipher
        let cipher = Rc::new(Crypt::new(key));
        // Create command string
        let m = String::from(m_orig);
        // Create, encrypt and serialize message
        let msg = Message::new(cipher);
        // Timestamp
        let ts: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut msg_enc: BytesMut = msg.encrypt_serialize(t, &m, ts).unwrap();

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

            match msg.deserialize_decrypt(msg_enc) {
                Err(_) => assert!(true),
                _ => assert!(false),
            }
        } else {
            // Deserialize and decrypt message
            let msg_dec: ReCmdMsg = msg.deserialize_decrypt(msg_enc).unwrap();

            match t {
                ReCmdMsgType::DirectCmdReq => {
                    if let ReCmdMsgPayload::DirectCmdReq {
                        ts: ts_dec,
                        m_len: m_len_dec,
                        m: m_dec,
                    } = msg_dec.payload
                    {
                        assert_eq!(ts, ts_dec);
                        assert_eq!(m.len(), m_len_dec as usize);
                        assert_eq!(m, m_dec);
                    }
                }
                ReCmdMsgType::DirectCmdRes => {
                    if let ReCmdMsgPayload::DirectCmdRes {
                        ts: ts_dec,
                        m_len: m_len_dec,
                        m: m_dec,
                    } = msg_dec.payload
                    {
                        assert_eq!(ts, ts_dec);
                        assert_eq!(m.len(), m_len_dec as usize);
                        assert_eq!(m, m_dec);
                    }
                }
                _ => {
                    assert!(false);
                }
            }
        }
    }

    #[test]
    fn encrypt_serialize_deserialize_decrypt_request() {
        encrypt_serialize_deserialize_decrypt(ReCmdMsgType::DirectCmdReq, "ls -la", false);
    }

    #[test]
    fn encrypt_serialize_deserialize_decrypt_response() {
        encrypt_serialize_deserialize_decrypt(ReCmdMsgType::DirectCmdRes, "ls -la", false);
    }

    #[test]
    fn encrypt_serialize_corr_deserialize_decrypt_request() {
        encrypt_serialize_deserialize_decrypt(ReCmdMsgType::DirectCmdReq, "ls -la", true);
    }

    #[test]
    fn encrypt_serialize_corr_deserialize_decrypt_response() {
        encrypt_serialize_deserialize_decrypt(ReCmdMsgType::DirectCmdRes, "ls -la", true);
    }
}
