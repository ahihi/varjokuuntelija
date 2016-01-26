use std::boxed::Box;
use std::collections::HashMap;
use std::error::Error;

use ::pm;
use ::pm::{InputPort, PortMidiDeviceId};

use error::CustomError;

pub type DeviceId = PortMidiDeviceId;
pub type Channel = u8;
pub type Cc = u8;
pub type Value = u8;

//pub type ChannelCcMap = HashMap<Channel, Cc>;
//pub type DeviceChannelMap = HashMap<DeviceId, ChannelCcMap>;

static INPUT_BUFFER_SIZE: i32 = 1024;
static STATUS_NIBBLE_CC: u8 = 0b1011;

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct CcKey {
    pub device_id: DeviceId,
    pub channel: Channel,
    pub cc: Cc
}

#[derive(Debug)]
pub struct CcEvent {
    pub key: CcKey,
    pub value: Value
}

pub struct MidiInputs {
    inputs: HashMap<DeviceId, InputPort>
}

impl MidiInputs {
    pub fn initialize() -> Result<(), Box<Error>> {
        Ok(try!(pm::initialize()))
    }
    
    pub fn terminate() -> Result<(), Box<Error>> {
        Ok(try!(pm::terminate()))
    }
    
    pub fn new(device_ids: &[DeviceId]) -> Result<MidiInputs, Box<Error>> {
        let mut inputs = HashMap::new();
        
        for &id in device_ids {
            let device_info = try!(
                pm::get_device_info(id)
                    .ok_or(CustomError::new(
                        &format!("Invalid device id: {}", id)
                    ))
            );
            println!("{:?}", device_info);
            inputs.insert(id, InputPort::new(id, INPUT_BUFFER_SIZE));
        }
                
        Ok(MidiInputs {
            inputs: inputs
        })
    }

    pub fn open(&mut self) -> Result<(), Box<Error>> {
        for (_, ref mut input) in self.inputs.iter_mut() {
            try!(input.open());
        }
        Ok(())
    }
    
    pub fn close(&mut self) -> Result<(), Box<Error>> {
        for (_, ref mut input) in self.inputs.iter_mut() {
            try!(input.close());
        }
        Ok(())
    }

    pub fn read_cc(&mut self) -> Result<Vec<CcEvent>, Box<Error>> {
        let mut last_values = HashMap::new();
        
        for (&device_id, ref mut input) in self.inputs.iter_mut() {
            while let Some(event) = try!(input.read()) {
                let msg = event.message;
                
                let status_nibble = msg.status >> 4;
                if status_nibble == STATUS_NIBBLE_CC {
                    let channel = (msg.status & 0b1111) + 1;
                    let cc = msg.data1;
                    let value = msg.data2;
                    
                    last_values.insert((device_id, channel, cc), value);
                }
            }
        }
                
        Ok(
            last_values
                .into_iter()
                .map(|((device_id, channel, cc), value)| {
                    let key = CcKey {
                        device_id: device_id,
                        channel: channel,
                        cc: cc
                    };
                    CcEvent { key: key, value: value }
                })
                .collect()
        )
    }
}
