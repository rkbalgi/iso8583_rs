#[cfg(test)]
mod tests {
    use std::net::TcpStream;
    use std::io::Write;
    use std::time::Duration;
    use byteorder::WriteBytesExt;
    use byteorder::ByteOrder;

    #[test]
    fn test_client() -> std::io::Result<()> {
        let mut client = TcpStream::connect("localhost:6666")?;
        client.write_u16::<byteorder::BigEndian>(11 as u16);
        client.write_all("hello world".as_bytes())?;

        client.flush();

        std::thread::sleep(Duration::from_secs(5));
        client.write_u16::<byteorder::BigEndian>(22 as u16
        );
        client.write_all("hello world again ... ".as_bytes())?;
        client.flush();
        client.shutdown(std::net::Shutdown::Both);
        Ok(())
    }
}
