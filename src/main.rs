#[macro_use]
extern crate lazy_static;
extern crate hex;
extern crate byteorder;

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

    let iso_msg = iso_spec.parse(hex::decode("00010203040506070809000101020304050607080900").expect("failed to decode hex")).expect("parsing failed!");
    println!("{:?}",iso_msg.fd_map)
}
