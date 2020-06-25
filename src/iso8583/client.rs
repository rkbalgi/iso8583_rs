//! This module contains implementation of a ISO TCP client

use crate::iso8583::iso_spec::{Spec, IsoMsg};
use crate::iso8583::IsoError;
use std::net::TcpStream;
use crate::iso8583::mli::{MLI, MLIType};
use std::io::{Write, BufReader, Read};
use byteorder::ReadBytesExt;
use crate::iso8583::server::get_hexdump;



/// This struct represents a ISO8583 TCP client
pub struct ISOTcpClient {
    server_addr: String,
    mli_type: MLIType,
    spec: &'static Spec,
    _tcp_stream: Option<TcpStream>,
}


impl ISOTcpClient {
    /// Creates a new ISOTcpClient
    pub fn new(server_addr: &str, spec: &'static Spec, mli_type: MLIType) -> ISOTcpClient {
        ISOTcpClient {
            server_addr: server_addr.to_string(),
            spec,
            mli_type,
            _tcp_stream: None,
        }
    }

    /// Sends a ISO message to the server and returns the response from server on success
    /// or a IsoError on failure
    pub fn send(&mut self, iso_msg: &IsoMsg) -> Result<IsoMsg, IsoError> {
        match iso_msg.assemble() {
            Ok(data) => {
                let mli = &crate::iso8583::mli::MLI2E {};
                let mut buf = mli.create(&data.len()).unwrap();
                buf.extend(data);
                self.send_recv(&buf)
            }
            Err(e) => {
                Err(IsoError { msg: format!("Failed to assemble request message: {}", e.msg) })
            }
        }
    }

    fn send_recv(&mut self, raw_msg: &Vec<u8>) -> Result<IsoMsg, IsoError> {
        println!("raw iso msg = {}", hex::encode(raw_msg.as_slice()));

        if self._tcp_stream.is_none() {
            self._tcp_stream = match TcpStream::connect(&self.server_addr) {
                Err(e) => return Err(IsoError { msg: e.to_string() }),
                Ok(c) => {
                    println!("connected to server @ {:?}", c.local_addr());
                    Option::Some(c)
                }
            }
        }

        let mut client = (self._tcp_stream.as_mut()).unwrap();

        client.write_all(raw_msg.as_slice());
        client.flush();

        // read the response

        let mut reader = BufReader::new(&mut client);
        let mut len: u32;

        match &self.mli_type {
            MLIType::MLI2E => {
                len = reader.read_u16::<byteorder::BigEndian>().unwrap() as u32;
            }
            MLIType::MLI2I => {
                len = reader.read_u16::<byteorder::BigEndian>().unwrap() as u32;
                len -= 2;
            }
            MLIType::MLI4E => {
                len = reader.read_u32::<byteorder::BigEndian>().unwrap() as u32;
            }
            MLIType::MLI4I => {
                len = reader.read_u32::<byteorder::BigEndian>().unwrap() as u32;
                len -= 4;
            }
        }


        let mut out_buf = vec![0; len as usize];

        match reader.read_exact(&mut out_buf[..]) {
            Ok(()) => {
                println!("received response: with  {} bytes. \n {}\n", len, get_hexdump(&out_buf));
                match self.spec.parse(&mut out_buf) {
                    Ok(resp_iso_msg) => {
                        Ok(resp_iso_msg)
                    }
                    Err(e) => {
                        Err(IsoError { msg: e.msg })
                    }
                }
            }
            Err(e) => {
                Err(IsoError { msg: e.to_string() })
            }
        }
    }
}