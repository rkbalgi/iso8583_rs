//! This module provides implementation of MAC algorithms

//https://en.wikipedia.org/wiki/ISO/IEC_9797-1#Complete_specification_of_the_MAC_calculation

use crate::crypto::{tdes_encrypt_cbc, des_encrypt_cbc};

/// This enum defines various supported algorithms
pub enum MacAlgo {
    //ISO9797 - algo 1
    CbcMac,
    // ISO9797 - algo 3
    RetailMac,
}

/// This enum defines all supported padding types
pub enum PaddingType {
    /// Adding 0 bits
    Type1,
    /// Adding a single 1 bit followed by 0 bits
    Type2,
}

pub struct MacError {
    pub msg: String
}


pub fn verify_mac(algo: &MacAlgo, padding_type: &PaddingType, data: &[u8], key: &Vec<u8>, expected_mac: &Vec<u8>) -> Result<(), MacError> {
    let mac = generate_mac(algo, padding_type, &data.to_vec(), key)?;
    if mac.eq(expected_mac) {
        Ok(())
    } else {
        Err(MacError { msg: format!("computed mac: {} doesn't match expected_mac: {}", hex::encode(mac), hex::encode(expected_mac)) })
    }
}

pub fn generate_mac(algo: &MacAlgo, padding_type: &PaddingType, data: &Vec<u8>, key: &Vec<u8>) -> Result<Vec<u8>, MacError> {
    let new_data = apply_padding(padding_type, data);
    let mut iv = Vec::<u8>::new();
    iv.extend_from_slice(hex::decode("0000000000000000").unwrap().as_slice());

    println!("generating mac on {}", hex::encode(data));

    match algo {
        MacAlgo::CbcMac => {
            let res = tdes_encrypt_cbc(&new_data, key, &iv);
            Ok(res[res.len() - 8..].to_vec())
        }
        MacAlgo::RetailMac => {
            let k = key.as_slice()[0..8].to_vec();

            //if there is a single block
            if data.len() == 8 {
                Ok(tdes_encrypt_cbc(&data, key, &iv))
            } else {

                //else, all but the last block DES and the last block TDES
                let d1 = &new_data[0..new_data.len() - 8].to_vec();
                let d2 = &new_data[new_data.len() - 8..].to_vec();

                let res1 = des_encrypt_cbc(&d1, &k, &iv);
                Ok(tdes_encrypt_cbc(&d2, key, &res1[(res1.len() - 8)..].to_vec()))
            }
        }
    }
}

fn apply_padding(padding_type: &PaddingType, data: &Vec<u8>) -> Vec<u8> {
    let mut new_data = data.clone();
    match padding_type {
        PaddingType::Type1 => {}
        PaddingType::Type2 => {
            new_data.push(0x80);
        }
    };

    while new_data.len() < 8 {
        new_data.push(0x00);
    }

    while new_data.len() % 8 != 0 {
        new_data.push(0x00);
    }

    new_data
}


#[cfg(test)]
mod tests {
    use crate::crypto::mac::{apply_padding, PaddingType, generate_mac, MacAlgo};
    use hex_literal::hex;

    #[test]
    fn test_padding1_shortof8() {
        let data = hex::decode("0102030405").unwrap();
        assert_eq!(hex::encode(apply_padding(&PaddingType::Type1, &data)), "0102030405000000");
    }

    #[test]
    fn test_padding1_exact() {
        let data = hex::decode("0102030405060708").unwrap();
        assert_eq!(hex::encode(apply_padding(&PaddingType::Type1, &data)), "0102030405060708");
    }

    #[test]
    fn test_padding1_typical_short() {
        let data = hex::decode("0102030405060708090a").unwrap();
        assert_eq!(hex::encode(apply_padding(&PaddingType::Type1, &data)), "0102030405060708090a000000000000");
    }


    #[test]
    fn test_padding2_shortof8() {
        let data = hex::decode("0102030405").unwrap();
        assert_eq!(hex::encode(apply_padding(&PaddingType::Type2, &data)), "0102030405800000");
    }

    #[test]
    fn test_padding2_exact() {
        let data = hex::decode("0102030405060708").unwrap();
        assert_eq!(hex::encode(apply_padding(&PaddingType::Type2, &data)), "01020304050607088000000000000000");
    }

    #[test]
    fn test_padding2_typical_short() {
        let data = hex::decode("0102030405060708090a").unwrap();
        assert_eq!(hex::encode(apply_padding(&PaddingType::Type2, &data)), "0102030405060708090a800000000000");
    }


    #[test]
    fn test_gen_mac_cbc_nopads() {
        let res = generate_mac(&MacAlgo::CbcMac, &PaddingType::Type1,
                               &Vec::from(hex!("0102030405060708")), &Vec::from(hex!("e0f4543f3e2a2c5ffc7e5e5a222e3e4d")));
        match res {
            Ok(m) => {
                println!("mac = {}", hex::encode(m.as_slice()));
                assert_eq!("7d34c3071da931b9", hex::encode(m));
            }
            Err(e) => {
                assert!(false, e.msg)
            }
        }
    }

    #[test]
    fn test_gen_mac_cbc_2() {
        let res = generate_mac(&MacAlgo::CbcMac, &PaddingType::Type1,
                               &Vec::from(hex!("01020304050607080102030405060708")), &Vec::from(hex!("e0f4543f3e2a2c5ffc7e5e5a222e3e4d")));
        match res {
            Ok(m) => {
                println!("mac = {}", hex::encode(m.as_slice()));
                assert_eq!("0fe28f4b5537ee79", hex::encode(m));
            }
            Err(e) => {
                assert!(false, e.msg)
            }
        }
    }


    #[test]
    fn test_gen_mac_cbc_3() {
        let res = generate_mac(&MacAlgo::CbcMac, &PaddingType::Type1,
                               &Vec::from(hex!("01020304050607080102030405")), &Vec::from(hex!("e0f4543f3e2a2c5ffc7e5e5a222e3e4d")));
        match res {
            Ok(m) => {
                println!("mac = {}", hex::encode(m.as_slice()));
                assert_eq!("8fb12963d5661a22", hex::encode(m));
            }
            Err(e) => {
                assert!(false, e.msg)
            }
        }
    }

    #[test]
    fn test_gen_mac_cbc_2_paddingtype2() {
        let res = generate_mac(&MacAlgo::CbcMac, &PaddingType::Type2,
                               &Vec::from(hex!("01020304050607080102030405")), &Vec::from(hex!("e0f4543f3e2a2c5ffc7e5e5a222e3e4d")));
        match res {
            Ok(m) => {
                println!("mac = {}", hex::encode(m.as_slice()));
                assert_eq!("8568cd2b7698605f", hex::encode(m));
            }
            Err(e) => {
                assert!(false, e.msg)
            }
        }
    }


    #[test]
    fn test_gen_mac_retail1_nopads() {
        let res = generate_mac(&MacAlgo::RetailMac, &PaddingType::Type1,
                               &Vec::from(hex!("0102030405060708")), &Vec::from(hex!("e0f4543f3e2a2c5ffc7e5e5a222e3e4d")));
        match res {
            Ok(m) => {
                println!("mac = {}", hex::encode(m.as_slice()));
                assert_eq!("7d34c3071da931b9", hex::encode(m));
            }
            Err(e) => {
                assert!(false, e.msg)
            }
        }
    }

    #[test]
    fn test_gen_mac_retail2_padtype1() {
        let res = generate_mac(&MacAlgo::RetailMac, &PaddingType::Type1,
                               &Vec::from(hex!("0102030405060708010203040506070801020304050607080000")), &Vec::from(hex!("e0f4543f3e2a2c5ffc7e5e5a222e3e4d")));
        match res {
            Ok(m) => {
                println!("mac = {}", hex::encode(m.as_slice()));
                assert_eq!(hex::encode(m), "149f99288681d292");
            }
            Err(e) => {
                assert!(false, e.msg)
            }
        }
    }

    #[test]
    fn test_gen_mac_retail_padtype2() {
        let res = generate_mac(&MacAlgo::RetailMac, &PaddingType::Type2,
                               &Vec::from(hex!("0102030405060708010203040506070801020304050607080000")), &Vec::from(hex!("e0f4543f3e2a2c5ffc7e5e5a222e3e4d")));
        match res {
            Ok(m) => {
                println!("mac = {}", hex::encode(m.as_slice()));
                assert_eq!(hex::encode(m), "4689dd5a87015394");
            }
            Err(e) => {
                assert!(false, e.msg)
            }
        }
    }
}