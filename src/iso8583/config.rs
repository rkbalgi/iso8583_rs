//! This module deals with various configurations that can be applied while creating a iso msg like
//! crypto field F52, F64/128 etc

use crate::crypto::pin::PinFormat;
use crate::crypto::mac::{MacAlgo, PaddingType};

pub struct Config {
    pin_format: Option<PinFormat>,
    pin_key: Option<String>,
    mac_algo: Option<MacAlgo>,
    mac_padding: Option<PaddingType>,
    mac_key: Option<String>,
}


impl Config {
    // Creates a new empty Config
    pub fn new() -> Config {
        Config {
            pin_format: None,
            pin_key: None,
            mac_algo: None,
            mac_key: None,
            mac_padding: None,
        }
    }

    /// Returns the PIN block format associated with this config
    pub fn get_pin_fmt(&self) -> &Option<PinFormat> {
        &self.pin_format
    }

    /// Returns the PIN key associated with this config
    pub fn get_pin_key(&self) -> &Option<String> {
        &self.pin_key
    }

    /// Returns the MAC key associated with this config
    pub fn get_mac_key(&self) -> &Option<String> {
        &self.mac_key
    }

    /// Returns the MAC'ing algorithm associated with this config
    pub fn get_mac_algo(&self) -> &Option<MacAlgo> {
        &self.mac_algo
    }

    /// Returns the MAC padding scheme associated with this config
    pub fn get_mac_padding(&self) -> &Option<PaddingType> {
        &self.mac_padding
    }


    /// Use the Config with a builder pattern
    pub fn with_pin(&mut self, fmt: PinFormat, key: String) -> &mut Config {
        self.pin_format = Some(fmt);
        self.pin_key = Some(key);
        self
    }

    /// Use the Config with a builder pattern
    pub fn with_mac(&mut self, algo: MacAlgo, mac_padding: PaddingType, key: String) -> &mut Config {
        self.mac_algo = Some(algo);
        self.mac_key = Some(key);
        self.mac_padding = Some(mac_padding);
        self
    }
}