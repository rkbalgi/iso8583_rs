#[cfg(test)]
mod tests {
    use std::io::{BufReader, Error, Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    use byteorder::{ReadBytesExt};
    use byteorder::ByteOrder;

    use crate::iso8583::{iso_spec, IsoError};
    use crate::iso8583::server::IsoServer;
    use crate::iso8583::field::Encoding::EBCDIC;
    use crate::iso8583::iso_spec::IsoMsg;
    use std::collections::HashMap;
    use crate::iso8583::mli::{MLI, MLI2E};
    use std::process::exit;


    #[test]
    fn test_send_recv_iso_1100() -> Result<(), Error> {
        std::env::set_var("SPEC_FILE", "sample_spec/sample_spec.yaml");

        let spec = crate::iso8583::iso_spec::spec("");
        let msg_seg = spec.get_message_from_header("1100").unwrap();

        let mut iso_msg = iso_spec::new_msg(spec,msg_seg);

        &iso_msg.set("message_type", "1100").unwrap();
        &iso_msg.set_on(2, "123456789101").unwrap();
        &iso_msg.set_on(3, "004000").unwrap();
        &iso_msg.set_on(4, "000000000199").unwrap();
        &iso_msg.set_on(11, "779581").unwrap();
        &iso_msg.set_on(14, "2204").unwrap();
        &iso_msg.set_on(19, "840").unwrap();
        &iso_msg.set_on(52, "0102030405060708").unwrap();
        &iso_msg.set_on(61, "Raghavendra").unwrap();
        &iso_msg.set_on(62, "Raghavendra Balgi").unwrap();
        &iso_msg.set_on(63, "87877622525").unwrap();
        &iso_msg.set_on(96, "1234").unwrap();
        &iso_msg.set_on(160, "5678").unwrap();


        match iso_msg.assemble() {
            Ok(data) => {
                let mli = &crate::iso8583::mli::MLI2E {};
                let mut buf = mli.create(&data.len()).unwrap();
                buf.extend(data);
                send_recv(&buf)
            }
            Err(e) => {
                println!("Failed to assemble request message: {}", e.msg);
                Ok(())
            }
        }
    }


    #[test]
    fn test_send_recv_iso_1420() -> Result<(), Error> {
        std::env::set_var("SPEC_FILE", "sample_spec/sample_spec.yaml");

        let spec = crate::iso8583::iso_spec::spec("");
        let msg_seg = spec.get_message_from_header("1420").unwrap();

        let mut iso_msg = iso_spec::new_msg(spec,msg_seg);

        &iso_msg.set("message_type", "1420").unwrap();
        &iso_msg.set_on(2, "123456789101").unwrap();
        &iso_msg.set_on(3, "004000").unwrap();
        &iso_msg.set_on(4, "000000000199").unwrap();
        &iso_msg.set_on(11, "779581").unwrap();
        &iso_msg.set_on(14, "2204").unwrap();
        &iso_msg.set_on(19, "840").unwrap();
        &iso_msg.set_on(96, "1234").unwrap();
        &iso_msg.set_on(160, "5678").unwrap();


        match iso_msg.assemble() {
            Ok(data) => {
                let mli = &crate::iso8583::mli::MLI2E {};
                let mut buf = mli.create(&data.len()).unwrap();
                buf.extend(data);
                send_recv(&buf)
            }
            Err(e) => {
                println!("Failed to assemble request message: {}", e.msg);
                Ok(())
            }
        }
    }


    fn send_recv(raw_msg: &Vec<u8>) -> Result<(), Error> {
        println!("raw iso msg = {}", hex::encode(raw_msg.as_slice()));

        let mut client = TcpStream::connect("localhost:6666")?;

        client.write_all(raw_msg.as_slice())?;
        client.flush();

        // read the response

        let mut reader = BufReader::new(&mut client);
        let len = reader.read_u16::<byteorder::BigEndian>().unwrap();


        let mut out_buf = vec![0; len as usize];

        match reader.read_exact(&mut out_buf[..]) {
            Ok(()) => {
                println!("received response: with  {} bytes.", len);
                hexdump::hexdump(&out_buf[..]);
                match iso_spec::spec("SampleSpec").parse(&mut out_buf) {
                    Ok(resp_iso_msg) => {
                        println!("parsed iso-response \"{}\" \n {} \n", resp_iso_msg.msg.name(), resp_iso_msg);
                    }
                    Err(e) => panic!(e.msg)
                }
            }
            Err(e) => {
                panic!(e)
            }
        }
        Ok(())
    }
}

