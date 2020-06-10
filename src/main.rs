//#![feature(vec_drain_as_slice)]
#[macro_use]
extern crate lazy_static;
extern crate hex;
extern crate byteorder;

#[macro_use]
extern crate log;
extern crate simplelog;

use std::time::Duration;
use crate::iso8583::server::IsoServer;

mod iso8583;


fn main() {
    let _ = simplelog::SimpleLogger::init(simplelog::LevelFilter::Debug, simplelog::Config::default());

    let iso_spec = iso8583::iso_spec::spec("SampleSpec");

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
