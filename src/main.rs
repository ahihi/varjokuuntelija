extern crate varjokuuntelu;

use std::error::Error;
use std::process;

use varjokuuntelu::Varjokuuntelu;

fn main() {
    match Varjokuuntelu::new() {
        Ok(vk) => vk.run(),
        Err(e) => {
            println!("{}", e.description());
            process::exit(1);
        }
    }
}
