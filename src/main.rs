extern crate varjokuuntelu;

use std::env;
use std::error::Error;
use std::process;

use varjokuuntelu::Varjokuuntelu;
use varjokuuntelu::midi::MidiInputs;

fn main() {
    MidiInputs::initialize().unwrap();
    let args: Vec<String> = env::args().collect();
    
    let exit_code = match Varjokuuntelu::new(&args) {
        Ok(vk) => {
            vk.run();
            0
        },
        Err(e) => {
            println!("{}", e.description());
            1
        }
    };
    MidiInputs::terminate().unwrap();
    process::exit(exit_code);
}
