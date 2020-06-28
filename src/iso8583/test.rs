#[cfg(test)]
mod tests {
    use crate::iso8583::client::ISOTcpClient;
    use crate::iso8583::{iso_spec, IsoError};
    use crate::iso8583::mli::MLIType::MLI2E;

    #[test]
    #[ignore]
    fn test_send_recv_iso_1100() -> Result<(), IsoError> {
        std::env::set_var("SPEC_FILE", "sample_spec/sample_spec.yaml");

        let spec = crate::iso8583::iso_spec::spec("");
        let msg_seg = spec.get_message_from_header("1100").unwrap();

        let mut iso_msg = iso_spec::new_msg(spec, msg_seg);

        iso_msg.set("message_type", "1100").unwrap();
        iso_msg.set_on(2, "123456789101").unwrap();
        iso_msg.set_on(3, "004000").unwrap();
        iso_msg.set_on(4, "000000000199").unwrap();
        iso_msg.set_on(11, "779581").unwrap();
        iso_msg.set_on(14, "2204").unwrap();
        iso_msg.set_on(19, "840").unwrap();
        iso_msg.set_on(52, "0102030405060708").unwrap();
        iso_msg.set_on(61, "reserved_1").unwrap();
        iso_msg.set_on(62, "reserved-2").unwrap();
        iso_msg.set_on(63, "87877622525").unwrap();
        iso_msg.set_on(96, "1234").unwrap();
        iso_msg.set_on(160, "5678").unwrap();

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
        std::env::set_var("SPEC_FILE", "sample_spec/sample_spec.yaml");

        let spec = crate::iso8583::iso_spec::spec("");
        let msg_seg = spec.get_message_from_header("1420").unwrap();

        let mut client = ISOTcpClient::new("localhost:6666", &spec, MLI2E);

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

