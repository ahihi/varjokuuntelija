extern crate getopts;

use std::boxed::Box;
use std::error::Error;
use std::fs::File;
use std::io::Read;

use self::getopts::Options;
use ::rustc_serialize::json;

use config::Config;
use error::CustomError;

fn usage(program: &str, opts: Options) -> String {
    let brief = format!("Usage: {} [options] FILE", program);
    format!("{}", opts.usage(&brief))
}

fn get_options_raw(
    args: &[String],
    opts: &mut Options
) -> Result<(Option<Config>, String, Option<(u32, u32)>, Option<usize>), Box<Error>> {
    opts.optopt("c", "config", "configuration file", "FILE");
    opts.optopt("w", "width", "resolution width", "PIXELS");
    opts.optopt("h", "height", "resolution height", "PIXELS");
    opts.optopt("f", "fullscreen", "enable full screen mode on display INDEX", "INDEX");
    
    let matches = try!(opts.parse(args));
    
    let config = match matches.opt_str("c") {
        Some(config_path) => {            
            let json_str = {
                let mut file = try!(File::open(&config_path));
                
                let mut s = String::new();
                try!(file.read_to_string(&mut s));
                s
            };

            let config: Config = try!(json::decode(&json_str));
            
            Some(config)
        },
        
        None =>
            None
    };
    
    let resolution_opt = match (matches.opt_str("w"), matches.opt_str("h")) {
        (Some(w_str), Some(h_str)) => {
            let width = try!(w_str.parse::<u32>());
            let height = try!(h_str.parse::<u32>());            
            Some((width, height))
        },
        
        (Some(_), None) =>
            return Err(From::from(CustomError::new("No -h/--height specified"))),
            
        (None, Some(_)) =>
            return Err(From::from(CustomError::new("No -w/--width specified"))),
        
        _ =>
            None
    };
    
    let fullscreen_monitor_ix_opt = match matches.opt_str("f") {
        Some(ix_str) => {
            let ix = try!(ix_str.parse::<usize>());
            Some(ix)
        },
        
        None =>
            None
    };
    
    let fs_path = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        return Err(From::from(CustomError::new("No file specified")));
    };
    
    Ok((config, fs_path, resolution_opt, fullscreen_monitor_ix_opt))
}

pub fn get_options(args: &[String]) -> Result<(Option<Config>, String, Option<(u32, u32)>, Option<usize>), String> {
    let program = args[0].clone();
    let mut opts = Options::new();
        
    match get_options_raw(&args[1..], &mut opts) {
        Ok(o) => Ok(o),
        Err(e) => Err(
            format!("{}\n\n{}", e.description(), usage(&program, opts))
        )
    }
}
