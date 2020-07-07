#[cfg(test)]
mod tests {
    use crate::iso8583::client::ISOTcpClient;
    use crate::iso8583::{iso_spec, IsoError};
    use crate::iso8583::mli::MLIType::MLI2E;
    use crate::crypto::pin::PinFormat::ISO0;
    use crate::iso8583::config::Config;
    use crate::crypto::mac::MacAlgo::RetailMac;
    use crate::crypto::mac::PaddingType::Type1;

    use log;
    use simplelog;
    use std::env::join_paths;
    use std::path::Path;

    #[test]
    #[ignore]
    fn test_send_recv_iso_1100() -> Result<(), IsoError> {
        let path = Path::new(".").join("sample_spec").join("sample_spec.yaml");
        std::env::set_var("SPEC_FILE", path.to_str().unwrap());

        let spec = crate::iso8583::iso_spec::spec("");
        let msg_seg = spec.get_message_from_header("1100").unwrap();


        let mut iso_msg = iso_spec::new_msg(spec, msg_seg);

        iso_msg.set("message_type", "1100").unwrap();
        iso_msg.set_on(2, "4567909845671235").unwrap();
        iso_msg.set_on(3, "004000").unwrap();
        iso_msg.set_on(4, "000000000029").unwrap();
        iso_msg.set_on(11, "779581").unwrap();
        iso_msg.set_on(14, "2204").unwrap();
        iso_msg.set_on(19, "840").unwrap();


        let mut cfg = Config::new();
        cfg.with_pin(ISO0, String::from("e0f4543f3e2a2c5ffc7e5e5a222e3e4d"))
            .with_mac(RetailMac, Type1, String::from("e0f4543f3e2a2c5ffc7e5e5a222e3e4d"));


        //--------- set pin - F52

        //this will compute a pin based on cfg and the supplied pan and set bit position 52
        iso_msg.set_pin("1234", iso_msg.bmp_child_value(2).unwrap().as_str(), &cfg).unwrap();

        // You can also directly set this if there are other means of computing the pin block
        // iso_msg.set_on(52, "0102030405060708").unwrap(); //binary field are represented in their hex encoded format

        //--------- set pin - F52

        iso_msg.set_on(61, "reserved_1").unwrap();
        iso_msg.set_on(62, "reserved-2").unwrap();
        iso_msg.set_on(63, "87877622525").unwrap();
        iso_msg.set_on(96, "1234").unwrap();


        //--------- set mac  - either F64 or F128
        iso_msg.set_mac(&cfg);
        //--------- set mac


        let mut client = ISOTcpClient::new("localhost:6666", &spec, MLI2E);

        match client.send(&iso_msg) {
            Ok(resp_iso_msg) => {
                println!("Received {} \n {}", resp_iso_msg.msg.name(), resp_iso_msg);
            }
            Err(e) => {
                eprintln!("{:?}", e)
            }
        }
        Ok(())
    }


    #[test]
    #[ignore]
    fn test_send_recv_iso_1420() -> Result<(), IsoError> {

        let path = Path::new(".").join("sample_spec").join("sample_spec.yaml");
        std::env::set_var("SPEC_FILE", path.to_str().unwrap());

        let spec = crate::iso8583::iso_spec::spec("");
        let msg_seg = spec.get_message_from_header("1420").unwrap();

        let mut client = ISOTcpClient::new("localhost:6666", &spec, MLI2E);

        //send 10 messages
        for _ in 1..10 {
            let mut iso_msg = iso_spec::new_msg(spec, msg_seg);

            iso_msg.set("message_type", "1420").unwrap();
            iso_msg.set_on(2, "123456789101").unwrap();
            iso_msg.set_on(3, "004000").unwrap();
            iso_msg.set_on(4, "000000000199").unwrap();
            iso_msg.set_on(11, "779581").unwrap();
            iso_msg.set_on(14, "2204").unwrap();
            iso_msg.set_on(19, "840").unwrap();
            iso_msg.set_on(96, "1234").unwrap();
            iso_msg.set_on(160, "5678").unwrap();


            match client.send(&iso_msg) {
                Ok(resp_iso_msg) => {
                    println!("Received {} \n {}", resp_iso_msg.msg.name(), resp_iso_msg);
                }
                Err(e) => {
                    eprintln!("{:?}", e)
                }
            }
        }
        Ok(())
    }
}

