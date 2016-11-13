extern crate gl;
extern crate glutin;
extern crate notify;
extern crate portmidi as pm;
extern crate rustc_serialize;
extern crate time;

pub mod config;
pub mod error;
pub mod midi;
pub mod options;
pub mod shaders;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::default::Default;
use std::error::Error;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::mem;
use std::ptr;
use std::os::raw;
use std::sync::mpsc::channel;

use gl::types::*;
use glutin::Api;
use glutin::ElementState::*;
use glutin::Event::*;
use glutin::VirtualKeyCode::*;
use glutin::Window;
use glutin::WindowBuilder;
use notify::{RecommendedWatcher, Watcher};
use time::SteadyTime;

use config::{Config, MidiConfig};
use error::CustomError;
use midi::{CcKey, DeviceId, MidiInputs};
use shaders::{Program, Shader};

type UniformLoc = i32;
type CcLocMap = HashMap<CcKey, UniformLoc>;

static VERTICES: [GLfloat; 12] = [
    -1.0, -1.0, 0.0,
     1.0, -1.0, 0.0,
     1.0,  1.0, 0.0,
    -1.0,  1.0, 0.0
];

static VERTEX_SHADER_SRC: &'static str = include_str!("glsl/default.vert");

static U_RESOLUTION: &'static str = "u_resolution";
static U_TIME: &'static str = "u_time";

pub fn str_ptr(s: &str) -> *const u8 {
    CString::new(s).unwrap().as_ptr()
}

pub struct Varjokuuntelu {
    config: Config,
    fragment_shader_path: String,
    window: Window,
    program: RefCell<Program>,
    fs_resolution_loc: Cell<UniformLoc>,
    fs_time_loc: Cell<UniformLoc>,
    midi_inputs: RefCell<MidiInputs>,
    midi_locs: RefCell<CcLocMap>,
    midi_state: RefCell<HashMap<UniformLoc, GLfloat>>
}

impl Varjokuuntelu {
    pub fn new(args: &[String]) -> Result<Varjokuuntelu, Box<Error>> {
        let (config_opt, fs_path, dimensions_opt, fullscreen_monitor_opt) = try!(
            options::get_options(args)
                .map_err(|msg| CustomError::new(&msg))
        );
        
        let config = match config_opt {
            Some(c) => c,
            None => Default::default()
        };
                
        let window = try!(init_window(dimensions_opt, fullscreen_monitor_opt));
        
        // Set up Vertex Array Object and Vertex Buffer Object
        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            // Create Vertex Array Object
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            // Create a Vertex Buffer Object and copy the vertex data to it
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (VERTICES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                mem::transmute(&VERTICES[0]),
                gl::STATIC_DRAW
            );
        }
        
        print!("Loading {}... ", fs_path);
        let (program, fs_resolution_loc, fs_time_loc, midi_locs) =
            try!(match load_fragment_shader_raw(&config.midi, &fs_path) {
                Ok(result) => {
                    println!("ok");
                    Ok(result)
                },
                
                Err(e) => {
                    println!("failed");
                    Err(e)
                }
            });
        
        let midi_inputs = {
            let device_ids: Vec<DeviceId> = config.midi.keys().map(|id| *id).collect();
            try!(MidiInputs::new(&device_ids))
        };
            
        Ok(Varjokuuntelu {
            config: config,
            fragment_shader_path: fs_path,
            window: window,
            program: RefCell::new(program),
            fs_resolution_loc: Cell::new(fs_resolution_loc),
            fs_time_loc: Cell::new(fs_time_loc),
            midi_inputs: RefCell::new(midi_inputs),
            midi_locs: RefCell::new(midi_locs),
            midi_state: RefCell::new(HashMap::new())
        })
    }
    
    fn enable_program(&self) {
        let program = self.program.borrow();

        // Enable shader program
        program.enable();
        unsafe {
            gl::BindFragDataLocation(program.id, 0, str_ptr("out_color"));
    
            // Specify the layout of the vertex data
            let pos_attr = gl::GetAttribLocation(program.id, str_ptr("position"));
            gl::EnableVertexAttribArray(pos_attr as GLuint);
            gl::VertexAttribPointer(
                pos_attr as GLuint,
                3,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                0,
                ptr::null()
            );
        };
    }
            
    fn load_fragment_shader(&self) -> Result<(), Box<Error>> {
        let (program, fs_resolution_loc, fs_time_loc, midi_locs) =
            try!(load_fragment_shader_raw(&self.config.midi, &self.fragment_shader_path));
        *self.program.borrow_mut() = program;
        self.enable_program();
        
        self.fs_resolution_loc.set(fs_resolution_loc);
        self.fs_time_loc.set(fs_time_loc);
        {
            let mut self_midi_locs = self.midi_locs.borrow_mut();
            *self_midi_locs = midi_locs;
        }
        
        Ok(())
    }
    
    fn handle_window_events(&self) -> bool {
        let mut end = false;
        for event in self.window.poll_events() {
            match event {
                Closed =>
                    { end = true; },
                KeyboardInput(Pressed, _, Some(Escape)) =>
                    { end = true; },
                _ =>
                    ()
            };
        }
        end
    }
    
    fn handle_midi_events(&self) {
        let midi_events = {
            let mut midi_inputs = self.midi_inputs.borrow_mut();
            match midi_inputs.read_cc() {
                Ok(cc) => cc,
                Err(_) => {
                    //println!("Warning: Failed to read MIDI events");
                    Vec::new()
                }
            }
        };
        
        let midi_locs = self.midi_locs.borrow();
        let mut midi_state = self.midi_state.borrow_mut();
        for event in midi_events.into_iter() {
            if let Some(&loc) = midi_locs.get(&event.key) {
                midi_state.insert(loc, event.value as GLfloat);
            }
        }
    }
    
    fn render(&self, time: GLfloat) {
        let (width, height) = match self.window.get_inner_size() {
            Some(res) => res,
            None      => (0, 0)
        };
        
        unsafe {
            // Pass resolution & time uniforms to shader
            gl::Uniform2f(
                self.fs_resolution_loc.get(),
                width as GLfloat,
                height as GLfloat
            );
            gl::Uniform1f(
                self.fs_time_loc.get(),
                time
            );

            // Pass midi uniforms
            self.handle_midi_events();
            for &loc in self.midi_locs.borrow().values() {
                let value = match self.midi_state.borrow().get(&loc) {
                    Some(&v) => v,
                    None => 0 as GLfloat
                };
                gl::Uniform1f(
                    loc,
                    value
                );
            }

            // Clear the screen to black
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw
            gl::DrawArrays(gl::TRIANGLE_FAN, 0, (VERTICES.len()) as i32);
        };
    }
    
    pub fn run(&self) {
        let (major, minor) = gl_version();
        println!("OpenGL version: {}.{}", major, minor);
        
        let (tx, rx) = channel();
        
        let mut watcher: RecommendedWatcher = Watcher::new(tx).unwrap();
        watcher.watch(&self.fragment_shader_path).unwrap();
        
        match self.midi_inputs.borrow_mut().open() {
            Ok(()) => {},
            Err(e) => {
                println!("Failed to open MIDI inputs: {}", e.description());
            }
        }
        
        self.enable_program();
        let start_time = SteadyTime::now();

        loop {
            let end = self.handle_window_events();
            if end {
                break;
            }
            
            match rx.try_recv() {
                Ok(_) => {
                    print!("Reloading {}... ", self.fragment_shader_path);
                    match self.load_fragment_shader() {
                        Ok(_) => println!("ok"),
                        Err(e) => println!("failed\n{}", e.description()),
                    }
                },
                    
                Err(_) => ()
            };


            let time = {
                let diff = SteadyTime::now() - start_time;
                0.001 * diff.num_milliseconds() as GLfloat
            };
            self.render(time);
        
            let _ = self.window.swap_buffers();
        }

        match self.midi_inputs.borrow_mut().close() {
            Ok(()) => {},
            Err(e) => {
                println!("Failed to close MIDI inputs: {}", e.description());
            }
        }
    }
}

fn init_window(
    dimensions_opt: Option<(u32, u32)>,
    fullscreen_monitor_ix_opt: Option<usize>
) -> Result<Window, Box<Error>> {
    // Construct a window
    let mut wb = WindowBuilder::new()
        .with_title("varjokuuntelija".to_string())
        .with_vsync()
        .with_gl(glutin::GlRequest::Specific(Api::OpenGlEs, (2, 0)))
        //.with_gl_profile(glutin::GlProfile::Core)
        .with_srgb(Some(true))
        ;

    // Add dimensions if specified
    if let Some((width, height)) = dimensions_opt {
        wb = wb.with_dimensions(width, height);
    }

    // Add fullscreen monitor if specified
    if let Some(fullscreen_monitor_ix) = fullscreen_monitor_ix_opt {
        let monitors = glutin::get_available_monitors();
        
        println!("Monitors:");
        let mut monitor_opt = None;
        for (i, m) in monitors.enumerate() {
            let name = m.get_name().unwrap_or("<Unknown>".to_string());
    
            if i == fullscreen_monitor_ix {
                monitor_opt = Some(m);
                print!("* ");
            } else {
                print!("  ");
            }
    
            println!("[{}] {}", i, name);
        }
        
        let fullscreen_monitor = try!(
            monitor_opt.ok_or(CustomError::new(
                &format!("Unable to get monitor {}", fullscreen_monitor_ix))
            )
        );
        
        wb = wb.with_fullscreen(fullscreen_monitor);
    }
    
    let window = try!(wb.build_strict());
    let _ = unsafe { window.make_current() };

    // Initialize GL
    gl::load_with(
        |symbol| window.get_proc_address(symbol) as *const raw::c_void
    );
    
    Ok(window)
}

fn gl_version() -> (GLint, GLint) {
    let mut major: GLint = -1;
    unsafe { gl::GetIntegerv(gl::MAJOR_VERSION, &mut major) };
    let mut minor: GLint = -1;
    unsafe { gl::GetIntegerv(gl::MINOR_VERSION, &mut minor) };
    (major, minor)
}

fn get_fragment_shader(path: &str) -> Result<Shader, Box<Error>> {
    let fragment_shader_src = {
        let mut file = try!(
            File::open(path)
                .map_err(|e| CustomError::new(
                    &format!("Failed to open file ({:?})", e.kind()))
                )
        );
        let mut src = String::new();
        try!(
            file.read_to_string(&mut src)
                .map_err(|_| CustomError::new("Failed to read file"))
        );
        src
    };

    Shader::new(&fragment_shader_src, gl::FRAGMENT_SHADER)
}

fn load_fragment_shader_raw(midi_config: &MidiConfig, path: &str) -> Result<(Program, UniformLoc, UniformLoc, CcLocMap), Box<Error>> {
    let vertex_shader = try!(Shader::new(VERTEX_SHADER_SRC, gl::VERTEX_SHADER));
    
    let fragment_shader = try!(get_fragment_shader(path));
    
    let midi_mappings = {
        let mut mappings = Vec::new();
        
        for (&device_id, channel_to_cc_to_uniform_map) in midi_config {
            for (&channel, cc_to_uniform_map) in channel_to_cc_to_uniform_map {
                for (&cc, uniform) in cc_to_uniform_map {
                    let key = CcKey { device_id: device_id, channel: channel, cc: cc };
                    mappings.push((key, uniform));
                }
            }
        }
        
        mappings
    };
    
    let uniforms = {
        let mut uniforms = vec![U_RESOLUTION, U_TIME];
        
        for &(_, uniform) in &midi_mappings {
            uniforms.push(uniform);
        }
        
        uniforms
    };
    
    let program = Program::new(
        vertex_shader,
        fragment_shader,
        &uniforms
    );
    
    let fs_resolution_loc = try!(program.get_fragment_uniform(U_RESOLUTION));
    let fs_time_loc = try!(program.get_fragment_uniform(U_TIME));
    
    let midi_locs = {
        let mut locs = HashMap::new();
        
        for (key, uniform) in midi_mappings {
            locs.insert(
                key,
                try!(program.get_fragment_uniform(uniform))
            );
        }
        
        locs
    };    

    Ok((program, fs_resolution_loc, fs_time_loc, midi_locs))
}
