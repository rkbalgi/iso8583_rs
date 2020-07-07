//! This module implements various PIN block types

/// More info here - https://www.eftlab.com/knowledge-base/261-complete-list-of-pin-blocks-in-payments/

use rand;
use super::rand::Rng;
use generic_array::GenericArray;
use des::block_cipher::NewBlockCipher;
use des::block_cipher::BlockCipher;
use crate::crypto::{tdes_ede2_decrypt, tdes_ede2_encrypt};


#[derive(Debug)]
pub enum PinFormat {
    //ANSI X9.8, ECI-4
    ISO0,
    ISO1,
    ISO2,
    ISO3,
    ISO4,
}

pub struct PinError {
    pub msg: String
}

pub fn generate_pin_block(fmt: &PinFormat, c_pin: &str, pan: &str, key: &str) -> Result<Vec<u8>, PinError> {
    match fmt {
        PinFormat::ISO0 => {
            let mut b1 = format!("0{:X}{}", c_pin.len(), c_pin);
            pad_8(&mut b1);
            println!("= {}", b1);

            //rightmost 12 not including check digit
            let mut b2 = String::from("0000");
            b2.push_str(&pan[pan.len() - 13..pan.len() - 1]);

            let res = xor_hexstr(b1.as_str(), b2.as_str());
            let res = tdes_ede2_encrypt(&res, &hex::decode(key).unwrap().to_vec());

            Ok(res.to_vec())
        }
        PinFormat::ISO1 => {
            let mut b1 = format!("1{:X}{}", c_pin.len(), c_pin);
            pad_8(&mut b1);
            match hex::decode(b1) {
                Ok(res) => {
                    let res = tdes_ede2_encrypt(&res, &hex::decode(key).unwrap().to_vec());
                    Ok(res)
                }
                Err(e) => {
                    Err(PinError { msg: e.to_string() })
                }
            }
        }

        PinFormat::ISO2 => {
            let mut b1 = format!("2{:X}{}", c_pin.len(), c_pin);
            while b1.len() != 16 {
                b1.push('F');
            }
            println!("= {}", b1);
            match hex::decode(b1) {
                Ok(res) => {
                    let res = tdes_ede2_encrypt(&res, &hex::decode(key).unwrap().to_vec());
                    Ok(res)
                }
                Err(e) => {
                    Err(PinError { msg: e.to_string() })
                }
            }
        }

        PinFormat::ISO3 => {
            let mut b1 = format!("3{:X}{}", c_pin.len(), c_pin);
            pad_8_a2f(&mut b1);
            println!("= {}", b1);

            //rightmost 12 not including check digit
            let mut b2 = String::from("0000");
            b2.push_str(&pan[pan.len() - 13..pan.len() - 1]);

            let res = xor_hexstr(b1.as_str(), b2.as_str());
            let res = tdes_ede2_encrypt(&res, &hex::decode(key).unwrap().to_vec());

            Ok(res.to_vec())
        }

        _ => {
            Err(PinError { msg: format!("{:?} is not supported yet.", fmt) })
        }
    }
}

/// Verifies the pin in the 'pin_block' against expected_pin and returns a boolean to indicate if there was
/// was a successful match
pub fn verify_pin(fmt: &PinFormat, expected_pin: &str, pin_block: &Vec<u8>, pan: &str, key: &str) -> Result<bool, PinError> {
    debug!("verifying pin - expected_pin: {},  block: {}, pan:{}, key:{}", expected_pin, hex::encode(pin_block), pan, key);
    match fmt {
        PinFormat::ISO0 => {
            let mut b2 = String::from("0000");
            b2.push_str(&pan[pan.len() - 13..pan.len() - 1]);

            let res = tdes_ede2_decrypt(&pin_block, &hex::decode(key).unwrap().to_vec());
            let res = xor_hexstr(hex::encode(res.as_slice()).as_str(), b2.as_str());
            let pin_len = res.get(0).unwrap();
            let b1 = hex::encode(&res);
            let actual_pin = b1.get(2 as usize..(2 + pin_len) as usize).unwrap().clone();
            if expected_pin == actual_pin {
                Ok(true)
            } else {
                Ok(false)
            }
        }

        PinFormat::ISO1 => {
            let res = tdes_ede2_decrypt(&pin_block, &hex::decode(key).unwrap().to_vec());

            let pin_len = res.get(0).unwrap();
            let b1 = hex::encode(&res);
            let actual_pin = b1.get(2 as usize..(2 + (pin_len - 16)) as usize).unwrap().clone();
            if expected_pin == actual_pin {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        PinFormat::ISO2 => {
            let res = tdes_ede2_decrypt(&pin_block, &hex::decode(key).unwrap().to_vec());

            let pin_len = res.get(0).unwrap();
            let b1 = hex::encode(&res);
            let actual_pin = b1.get(2 as usize..(2 + (pin_len - 32)) as usize).unwrap().clone();
            if expected_pin == actual_pin {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        PinFormat::ISO3 => {
            let mut b2 = String::from("0000");
            b2.push_str(&pan[pan.len() - 13..pan.len() - 1]);

            let res = tdes_ede2_decrypt(&pin_block, &hex::decode(key).unwrap().to_vec());
            let res = xor_hexstr(hex::encode(res.as_slice()).as_str(), b2.as_str());
            let pin_len = res.get(0).unwrap();
            let b1 = hex::encode(&res);
            let actual_pin = b1.get(2 as usize..(2 + (pin_len - 48)) as usize).unwrap().clone();
            if expected_pin == actual_pin {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        _ => {
            Err(PinError { msg: format!("{:?} is not supported yet.", fmt) })
        }
    }
}



/// XOR the contents of 2 hex string (of equal length) and return the result
/// as a Vec<u8>
fn xor_hexstr(b1: &str, b2: &str) -> Vec<u8> {
    assert_eq!(b1.len(), b2.len());
    hex::decode(b1).unwrap().iter().
        zip(hex::decode(b2).
            unwrap().iter()).
        map(|f| f.0 ^ f.1).collect::<Vec<u8>>()
}


/// Pad a random hex string to'data' to make it 8 bytes
fn pad_8(data: &mut String) {
    let padding: [u8; 8] = rand::thread_rng().gen();
    data.push_str(hex::encode(padding).as_str());
    data.truncate(16);
}

/// Pad a random hex string  (only from A to F) to 'data' to make it 8 bytes
fn pad_8_a2f(data: &mut String) {
    let mut padding: [u8; 8] = rand::thread_rng().gen();
    padding.iter_mut().for_each(|f: &mut u8| {
        //just ensure a min of A for each :-)
        *f = *f | (0xAA as u8);
    });
    data.push_str(hex::encode(padding).as_str());
    data.truncate(16);
}


#[cfg(test)]
mod tests {
    use crate::crypto::pin::{generate_pin_block, verify_pin};
    use crate::crypto::pin::PinFormat::{ISO0, ISO1, ISO2, ISO3};

    #[test]
    fn test_iso0() {
        match generate_pin_block(&ISO0, "1234", "4111111111111111", "e0f4543f3e2a2c5ffc7e5e5a222e3e4d") {
            Ok(p) => {
                //assert_eq!(hex::encode(&p), "6042012526a9c2e0");

                match verify_pin(&ISO0, "1234", &p, "4111111111111111", "e0f4543f3e2a2c5ffc7e5e5a222e3e4d") {
                    Ok(res) => {
                        assert_eq!(res, true)
                    }
                    Err(e) => {
                        assert!(false, e.msg.to_string());
                    }
                }
            }
            Err(e) => {
                assert!(false, e.msg.to_string());
            }
        }

        match generate_pin_block(&ISO0, "12341123456", "4111111111111111", "e0f4543f3e2a2c5ffc7e5e5a222e3e4d") {
            Ok(p) => {
                //assert_eq!(hex::encode(&p), "6042012526a9c2e0");

                match verify_pin(&ISO0, "12341123456", &p, "4111111111111111", "e0f4543f3e2a2c5ffc7e5e5a222e3e4d") {
                    Ok(res) => {
                        assert_eq!(res, true)
                    }
                    Err(e) => {
                        assert!(false, e.msg.to_string());
                    }
                }
            }
            Err(e) => {
                assert!(false, e.msg.to_string());
            }
        }
    }

    #[test]
    fn test_iso1() {
        match generate_pin_block(&ISO1, "8976", "4111111111111111", "e0f4543f3e2a2c5ffc7e5e5a222e3e4d") {
            Ok(p) => {
                match verify_pin(&ISO1, "8976", &p, "4111111111111111", "e0f4543f3e2a2c5ffc7e5e5a222e3e4d") {
                    Ok(res) => {
                        assert_eq!(res, true)
                    }
                    Err(e) => {
                        assert!(false, e.msg.to_string());
                    }
                }
            }
            Err(e) => {
                assert!(false, e.msg.to_string());
            }
        }
    }

    #[test]
    fn test_iso2() {
        match generate_pin_block(&ISO2, "8976", "4111111111111111", "e0f4543f3e2a2c5ffc7e5e5a222e3e4d") {
            Ok(p) => {
                assert_eq!(hex::encode(&p), "795e511357332491");

                match verify_pin(&ISO2, "8976", &p, "4111111111111111", "e0f4543f3e2a2c5ffc7e5e5a222e3e4d") {
                    Ok(res) => {
                        assert_eq!(res, true)
                    }
                    Err(e) => {
                        assert!(false, e.msg.to_string());
                    }
                }
            }
            Err(e) => {
                assert!(false, e.msg.to_string());
            }
        }
    }

    #[test]
    fn test_iso3() {
        match generate_pin_block(&ISO3, "1234", "4111111111111111", "e0f4543f3e2a2c5ffc7e5e5a222e3e4d") {
            Ok(p) => {
                match verify_pin(&ISO3, "1234", &p, "4111111111111111", "e0f4543f3e2a2c5ffc7e5e5a222e3e4d") {
                    Ok(res) => {
                        assert_eq!(res, true)
                    }
                    Err(e) => {
                        assert!(false, e.msg.to_string());
                    }
                }
            }
            Err(e) => {
                assert!(false, e.msg.to_string());
            }
        }

        match generate_pin_block(&ISO3, "12341123456", "4111111111111111", "e0f4543f3e2a2c5ffc7e5e5a222e3e4d") {
            Ok(p) => {
                match verify_pin(&ISO3, "12341123456", &p, "4111111111111111", "e0f4543f3e2a2c5ffc7e5e5a222e3e4d") {
                    Ok(res) => {
                        assert_eq!(res, true)
                    }
                    Err(e) => {
                        assert!(false, e.msg.to_string());
                    }
                }
            }
            Err(e) => {
                assert!(false, e.msg.to_string());
            }
        }
    }
}