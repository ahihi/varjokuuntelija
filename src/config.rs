use std::collections::HashMap;

use midi::{Cc, Channel, DeviceId};

#[derive(Debug, Default, RustcDecodable, RustcEncodable)]
pub struct Config {
    pub midi: MidiConfig
}

pub type MidiConfig = HashMap<DeviceId, HashMap<Channel, HashMap<Cc, String>>>;
