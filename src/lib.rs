#[macro_use]
extern crate glium;
extern crate notify;
extern crate portmidi as pm;
extern crate rustc_serialize;
extern crate time;

pub mod config;
pub mod error;
pub mod midi;
pub mod options;
//pub mod shaders;

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
use std::time::{Duration};

use glium::{Display, Program, Surface, VertexBuffer};
use glium::glutin;
use glium::glutin::{Api, ElementState, GlRequest, VirtualKeyCode, Window};
use glium::index::{NoIndices, PrimitiveType};
use glium::program::{ProgramCreationError};
use glium::uniforms::{EmptyUniforms, Sampler};
use glium::texture::texture2d::{Texture2d};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use time::{SteadyTime};

use config::{Config, MidiConfig};
use error::CustomError;
use midi::{CcKey, DeviceId, MidiInputs};

static VERTEX_SHADER_SRC: &'static str = include_str!("glsl/default.vert");

static U_RESOLUTION: &'static str = "u_resolution";
static U_TIME: &'static str = "u_time";

#[derive(Copy,Clone)]
struct Vertex {
    position: [f32; 2]
}

impl Vertex {
    fn new(x: f32, y: f32) -> Vertex {
        Vertex { position: [x, y] }
    }
}

implement_vertex!(Vertex, position);

#[derive(Debug)]
struct VarjoUniforms<'a> {
    resolution: [f32; 2],
    time: f32,
    midi: &'a HashMap<String, f32>
}

impl glium::uniforms::Uniforms for VarjoUniforms<'_> {
    fn visit_values<'a, F: FnMut(&str, glium::uniforms::UniformValue<'a>)>(&'a self, mut output: F) {
        use glium::uniforms::AsUniformValue;
        
        output(U_RESOLUTION, self.resolution.as_uniform_value());
        output(U_TIME, self.time.as_uniform_value());
        for (name, value) in self.midi.iter() {
            output(name, value.as_uniform_value());
        }
    }
}

pub struct Varjokuuntelu {
    config: Config,
    fragment_shader_path: String,
    display: glium::Display,
    events_loop: glutin::EventsLoop,
    vbuf: VertexBuffer<Vertex>,
    ixs: NoIndices,
    program: RefCell<Program>,
    midi_inputs: RefCell<MidiInputs>,
    midi_state: RefCell<HashMap<String, f32>>
}

impl Varjokuuntelu {
    pub fn new(args: &[String]) -> Result<Varjokuuntelu, Box<Error>> {
        let (config_opt, fs_path, dimensions_opt, fullscreen_monitor_opt) = options::get_options(args)
            .map_err(|msg| CustomError::new(&msg))?;
        
        let config = match config_opt {
            Some(c) => c,
            None => Default::default()
        };
                
        let (display, events_loop) = init_display(dimensions_opt, fullscreen_monitor_opt)?;

        let vbuf = VertexBuffer::new(&display, &vec![
            Vertex::new(-1.0, -1.0), Vertex::new( 1.0, -1.0),
            Vertex::new( 1.0,  1.0), Vertex::new(-1.0,  1.0)
        ])?;

        let ixs = NoIndices(PrimitiveType::TriangleFan);
        
        print!("Loading {}... ", fs_path);
        let program =
            (match load_fragment_shader_raw(&display, &config.midi, &fs_path) {
                Ok(result) => {
                    println!("ok");
                    Ok(result)
                },
                
                Err(e) => {
                    println!("failed");
                    Err(e)
                }
            })?;
        
        let midi_inputs = {
            let device_ids: Vec<DeviceId> = config.midi.keys().map(|id| *id).collect();
            MidiInputs::new(&device_ids)?
        };
            
        Ok(Varjokuuntelu {
            config: config,
            fragment_shader_path: fs_path,
            display: display,
            events_loop: events_loop,
            vbuf: vbuf,
            ixs: ixs,
            program: RefCell::new(program),
            midi_inputs: RefCell::new(midi_inputs),
            midi_state: RefCell::new(HashMap::new())
        })
    }
            
    fn load_fragment_shader(&self) -> Result<(), Box<Error>> {
        let program =
            load_fragment_shader_raw(&self.display, &self.config.midi, &self.fragment_shader_path)?;
        *self.program.borrow_mut() = program;
        
        Ok(())
    }
    
    fn handle_window_events(&mut self) -> bool {
        let mut end = false;
        
        self.events_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent { event: glutin::WindowEvent::CloseRequested, .. } =>
                    { end = true; },
                glutin::Event::WindowEvent { event: glutin::WindowEvent::KeyboardInput { input: glutin::KeyboardInput { virtual_keycode: Some(glutin::VirtualKeyCode::Escape), .. }, .. }, .. } =>
                    { end = true; },
                _ => {}
            };
        });
        
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
                midi_state.insert(loc, event.value as f32);
            }
        }
    }
    
    fn render(&self, time: f32) {
        let mut target = self.display.draw();

        let (width, height) = target.get_dimensions();

        let unis = VarjoUniforms {
            resolution: [width as f32, height as f32],
            time: time,
            midi: &self.midi_state.borrow()
        };

        target.clear_color(0.0, 0.0, 0.0, 1.0);
        
        if let Err(e) = target.draw(&self.vbuf, &self.ixs, &self.program.borrow(), &unis, &Default::default()) {
            println!("Failed to render: {}", e);
        }

        if let Err(e) = target.finish() {
            println!("Failed to finish target: {}", e);
        }
    }
    
    pub fn run(&mut self) {
        let (major, minor) = gl_version();
        println!("OpenGL version: {}.{}", major, minor);
        
        let (tx, rx) = channel();
        
        let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_millis(0)).unwrap();
        watcher.watch(&self.fragment_shader_path, RecursiveMode::NonRecursive).unwrap();
        
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
                0.001 * diff.num_milliseconds() as f32
            };
            self.render(time);
        }
    }
}

fn init_display(
    dimensions_opt: Option<(u32, u32)>,
    fullscreen_monitor_ix_opt: Option<usize>
) -> Result<(glium::Display, glutin::EventsLoop), Box<Error>> {
    let events_loop = glutin::EventsLoop::new();
    let mut wb = glutin::WindowBuilder::new()
        .with_title("varjokuuntelija".to_string());
    let cb = glutin::ContextBuilder::new()
        //.with_gl(GlRequest::Specific(Api::OpenGlEs, (2, 0)))
        //.with_gl_profile(glutin::GlProfile::Core)
        .with_gl(GlRequest::Latest)
        .with_vsync(true)
        .with_srgb(true);

    // add dimensions if specified
    if let Some((width, height)) = dimensions_opt {
        wb = wb.with_dimensions(glutin::dpi::LogicalSize::new(width as f64, height as f64));
    }

    // add fullscreen monitor if specified
    if let Some(fullscreen_monitor_ix) = fullscreen_monitor_ix_opt {
        let monitors = events_loop.get_available_monitors();
        
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
        
        let fullscreen_monitor = monitor_opt.ok_or(CustomError::new(
            &format!("Unable to get monitor {}", fullscreen_monitor_ix))
        )?;
        
        wb = wb.with_fullscreen(Some(fullscreen_monitor));
    }

    let display = glium::Display::new(wb, cb, &events_loop)?;
        
    Ok((display, events_loop))
}

fn gl_version() -> (i32, i32) {
    /*let mut major: GLint = -1;
    unsafe { gl::GetIntegerv(gl::MAJOR_VERSION, &mut major) };
    let mut minor: GLint = -1;
    unsafe { gl::GetIntegerv(gl::MINOR_VERSION, &mut minor) };
    (major, minor)*/
    (-666, -666)
}

fn get_fragment_shader(path: &str) -> Result<String, Box<Error>> {
    let fragment_shader_src = {
        let mut file = File::open(path)
            .map_err(|e| CustomError::new(
                &format!("Failed to open file ({:?})", e.kind()))
            )?;
        let mut src = String::new();
        file.read_to_string(&mut src)
            .map_err(|_| CustomError::new("Failed to read file"))?;
        src
    };

    Ok(fragment_shader_src)
}

fn load_fragment_shader_raw(
    display: &glium::Display,
    midi_config: &MidiConfig,
    path: &str
) -> Result<Program, Box<Error>> {
    let fragment_shader_src = get_fragment_shader(path)?;
    
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

    let program_result = Program::from_source(display, VERTEX_SHADER_SRC, &fragment_shader_src, None);

    if let Err(ProgramCreationError::CompilationError(e)) = program_result {
        return Err(Box::new(CustomError::new(&e)));
    }

    let program = program_result?;
    
    Ok(program)
}
