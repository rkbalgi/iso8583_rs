pub mod mac;
pub mod pin;

extern crate rand;
extern crate des;
extern crate block_modes;
extern crate hex_literal;

use des::TdesEde2;
use generic_array::{GenericArray, ArrayLength};
use des::block_cipher::NewBlockCipher;
use des::block_cipher::BlockCipher;


use self::block_modes::BlockMode;
use self::des::Des;
use crate::crypto::mac::{generate_mac, MacAlgo, PaddingType};

pub(crate) fn tdes_ede2_encrypt(data: &Vec<u8>, key: &Vec<u8>) -> Vec<u8> {
    let block_cipher = des::TdesEde2::new(GenericArray::from_slice(key.as_slice()));

    let mut cp_data = data.clone();
    block_cipher.encrypt_block(GenericArray::from_mut_slice(&mut cp_data));
    cp_data
}

pub(crate) fn tdes_ede2_decrypt(data: &Vec<u8>, key: &Vec<u8>) -> Vec<u8> {
    let block_cipher = des::TdesEde2::new(GenericArray::from_slice(key.as_slice()));

    let mut cp_data = data.clone();
    block_cipher.decrypt_block(GenericArray::from_mut_slice(&mut cp_data));
    cp_data
}

type TripleDESCBC = block_modes::Cbc::<des::TdesEde2, block_modes::block_padding::NoPadding>;

pub(crate) fn tdes_encrypt_cbc(data: &Vec<u8>, key: &Vec<u8>, iv: &Vec<u8>) -> Vec<u8> {
    let mut block_cipher = TripleDESCBC::new_var(key.as_slice(), &iv.as_slice()).unwrap();

    let encrypted_data = block_cipher.encrypt_vec(data.as_slice());
    encrypted_data
}


pub(crate) fn des_encrypt_cbc(data: &Vec<u8>, key: &Vec<u8>, iv: &Vec<u8>) -> Vec<u8> {
    let block_cipher = block_modes::Cbc::<des::Des, block_modes::block_padding::NoPadding>::new_var(key.as_slice(), iv.as_slice()).unwrap();

    block_cipher.encrypt_vec(data)
}

pub(crate) fn des_decrypt_cbc(data: &Vec<u8>, key: &Vec<u8>) -> Vec<u8> {
    let block_cipher = des::Des::new(GenericArray::from_slice(key.as_slice()));

    let mut cp_data = data.clone();
    block_cipher.decrypt_block(GenericArray::from_mut_slice(&mut cp_data));
    cp_data
}