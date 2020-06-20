#[macro_use]
extern crate lazy_static;
extern crate hex;
extern crate byteorder;

#[macro_use]
extern crate log;
extern crate simplelog;

use std::time::Duration;
use crate::iso8583::server::IsoServer;

pub mod iso8583;

fn main(){


    std::env::set_var("SPEC_FILE","sample_spec\\sample_spec.yaml");

    let _ = simplelog::SimpleLogger::init(simplelog::LevelFilter::Debug, simplelog::Config::default());

    let iso_spec = crate::iso8583::iso_spec::spec("");

    info!("starting iso server for spec {} at port {}", iso_spec.name(), 6666);
    let server: IsoServer = match crate::iso8583::server::new("localhost:6666".to_string(), iso_spec) {
        Ok(server) => {
            server
        }
        Err(e) => {
            panic!(e)
        }
    };
    server.start().join();

}


