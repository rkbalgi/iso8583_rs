#[macro_use]
extern crate lazy_static;
extern crate hex;
extern crate byteorder;

use std::io::{Read, Write};

mod iso8583;


fn main() {
    //let testvec = vec![1, 2, 3];

    println!("Hello, world!");
    // println!("{:?} {:?}", testvec, iso8583::bitmap::new_bmp());

    let bmp = &iso8583::bitmap::new_bmp(0x7e02030400000001, 0, 0x8000000000000001);
    for i in 1..193 {
        println!("{} {}", i, bmp.is_on(i))
    }
    println!("{}", bmp.hex_string());

    let iso_spec = iso8583::iso_spec::Spec("SampleSpec");

    //31313030 6024000000000000 123456789101 004000 779581 2204
    let mut raw_msg: Vec<u8> = Vec::new();
    "1100".as_bytes().read_to_end(&mut raw_msg);
    raw_msg.write_all(hex::decode("6024000000000000").expect("failed to decode bmp").as_slice());
    "123456789101".as_bytes().read_to_end(&mut raw_msg);
    "004000".as_bytes().read_to_end(&mut raw_msg);
    "779581".as_bytes().read_to_end(&mut raw_msg);
    "2204".as_bytes().read_to_end(&mut raw_msg);

    println!("raw iso msg = {}", hex::encode(raw_msg.as_slice()));
    println!("raw iso msg = {}", hex::encode(raw_msg.as_slice()));

    match iso_spec.parse(raw_msg) {
        Ok(iso_msg) => println!("{:?}", iso_msg.fd_map),
        Err(e) => panic!(e),
    }

}
