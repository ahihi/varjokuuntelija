use std::boxed::Box;
use std::collections::HashMap;
use std::error::Error;

use ::pm;

use error::CustomError;

pub type DeviceId = pm::PortMidiDeviceId;
pub type Channel = u8;
pub type Cc = u8;
pub type Value = u8;

//pub type ChannelCcMap = HashMap<Channel, Cc>;
//pub type DeviceChannelMap = HashMap<DeviceId, ChannelCcMap>;

static INPUT_BUFFER_SIZE: usize = 1024;
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
    portmidi: pm::PortMidi,
    inputs: HashMap<DeviceId, pm::InputPort>
}

impl MidiInputs {
    pub fn new(device_ids: &[DeviceId]) -> Result<MidiInputs, Box<Error>> {
        let portmidi = try!(pm::PortMidi::new());
        
        let mut inputs = HashMap::new();
        
        for &id in device_ids {
            let device_info = try!(
                portmidi.device(id)
                    /*.ok_or(CustomError::new(
                        &format!("Invalid device id: {}", id)
                    ))*/
            );
            println!("{:?}", device_info);
            let input_port = try!(portmidi.input_port(device_info, INPUT_BUFFER_SIZE));
            inputs.insert(id, input_port);
        }
                
        Ok(MidiInputs {
            portmidi: portmidi,
            inputs: inputs
        })
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
