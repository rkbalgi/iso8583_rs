//#![feature(vec_drain_as_slice)]
#[macro_use]
extern crate lazy_static;
extern crate hex;
extern crate byteorder;

use std::io::{Read, Write};
use std::time::Duration;
use crate::iso8583::server::IsoServer;

mod iso8583;


fn main() {
    let iso_spec = iso8583::iso_spec::spec("SampleSpec");


    // build a raw iso message as per "SampleSpec"
    let mut raw_msg: Vec<u8> = Vec::new();

    // message type
    "1100".as_bytes().read_to_end(&mut raw_msg);
    //bitmap
    raw_msg.write_all(hex::decode("6024000000000000").expect("failed to decode bmp").as_slice());

    //pan - with length indicator and data
    "12".as_bytes().read_to_end(&mut raw_msg);
    "123456789101".as_bytes().read_to_end(&mut raw_msg);

    //proc code
    "004000".as_bytes().read_to_end(&mut raw_msg);
    //stan
    "779581".as_bytes().read_to_end(&mut raw_msg);
    //expiration date
    "2204".as_bytes().read_to_end(&mut raw_msg);

    println!("raw iso msg = {}", hex::encode(raw_msg.as_slice()));


    match iso_spec.parse(raw_msg) {
        Ok(iso_msg) => println!("{}", iso_msg),
        Err(e) => panic!(e),
    }

    let server: IsoServer = match crate::iso8583::server::new("localhost:6666".to_string()) {
        Ok(server) => {
            server
        }
        Err(e) => {
            panic!(e)
        }
    };
    server.start();


    std::thread::sleep(Duration::from_secs(1000));
}
