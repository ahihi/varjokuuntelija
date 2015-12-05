extern crate getopts;

use std::boxed::Box;
use std::env;
use std::error::Error;
use std::fmt;

use self::getopts::Options;

fn usage(program: &str, opts: Options) -> String {
    let brief = format!("Usage: {} [options] FILE", program);
    format!("{}", opts.usage(&brief))
}

#[derive(Debug)]
struct CustomError {
    pub message: String
}

impl CustomError {
    fn new(message: &str) -> CustomError {
        CustomError { message: message.to_string() }
    } 
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for CustomError {
    fn description(&self) -> &str {
        &self.message
    }
    
    fn cause(&self) -> Option<&Error> {
        None
    }
}

fn get_options_raw(
    args: &[String],
    opts: &mut Options
) -> Result<(String, Option<(u32, u32)>, Option<usize>), Box<Error>> {    
    opts.optopt("w", "width", "resolution width", "PIXELS");
    opts.optopt("h", "height", "resolution height", "PIXELS");
    opts.optopt("f", "fullscreen", "enable full screen mode on display INDEX", "INDEX");
    
    let matches = try!(
        opts.parse(args)
    );
    
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
    
    Ok((fs_path, resolution_opt, fullscreen_monitor_ix_opt))
}

pub fn get_options() -> Result<(String, Option<(u32, u32)>, Option<usize>), String> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
        
    match get_options_raw(&args[1..], &mut opts) {
        Ok(o) => Ok(o),
        Err(e) => Err(
            format!("{}\n\n{}", e.description(), usage(&program, opts))
        )
    }
}