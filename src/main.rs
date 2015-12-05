extern crate gl;
extern crate glutin;
extern crate time;

extern crate varjokuuntelu;

use std::mem;
use std::ptr;
use std::os::raw;

use gl::types::*;
use glutin::ElementState::*;
use glutin::Event::*;
use glutin::VirtualKeyCode::*;
use glutin::Window;
use glutin::WindowBuilder;
use time::Duration;
use time::SteadyTime;

use varjokuuntelu::options;
use varjokuuntelu::str_ptr;
use varjokuuntelu::shaders::{Program, Shader};

static VS_SRC: &'static str = include_str!("glsl/default.vert");
static FS_SRC: &'static str = include_str!("glsl/default.frag");

static U_RESOLUTION: &'static str = "u_resolution";
static U_TIME: &'static str = "u_time";

fn gl_version() -> (GLint, GLint) {
    let mut major: GLint = -1;
    unsafe { gl::GetIntegerv(gl::MAJOR_VERSION, &mut major) };
    let mut minor: GLint = -1;
    unsafe { gl::GetIntegerv(gl::MINOR_VERSION, &mut minor) };
    (major, minor)
}

fn init_window(
    dimensions_opt: Option<(u32, u32)>,
    fullscreen_monitor_ix_opt: Option<usize>
) -> Window {
    // Construct a window
    let mut wb = WindowBuilder::new()
        .with_title("varjokuuntelija".to_string())
        .with_vsync()
        .with_gl(glutin::GlRequest::Latest)
        .with_gl_profile(glutin::GlProfile::Core)
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
        
        let fullscreen_monitor = monitor_opt.unwrap();
        
        wb = wb.with_fullscreen(fullscreen_monitor);
    }
    
    let window = wb.build_strict().unwrap();
    let _ = unsafe { window.make_current() };

    // Initialize GL
    gl::load_with(
        |symbol| window.get_proc_address(symbol) as *const raw::c_void
    );
    
    window
}

fn load_shaders() -> (Shader, Shader) {
    let vs = Shader::new(VS_SRC, gl::VERTEX_SHADER);
    let fs = Shader::new(FS_SRC, gl::FRAGMENT_SHADER);
    (vs, fs)
}

fn init_rendering<'a>(
    vs: &'a Shader,
    fs: &'a Shader,
    vertices: &[GLfloat]
) -> Program<'a> {
    // Link shaders
    let program = Program::new(&vs, &fs, &vec![U_RESOLUTION, U_TIME]);

    // Set up Vertax Array Object and Vertex Buffer Object
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
            (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            mem::transmute(&vertices[0]),
            gl::STATIC_DRAW
        );
   }

   unsafe {
        // Use shader program
        program.enable();
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
    }

    program
}

fn handle_window_events(window: &Window) -> bool {
    let mut end = false;
    for event in window.poll_events() {
        match event {
            Closed =>
                { end = true; },
            KeyboardInput(Pressed, _, Some(Escape)) =>
                { end = true; },
            _ =>
                //println!("{:?}", event)
                ()
        };
    }
    end
}

fn render(
    vertices: &[GLfloat],
    fs_resolution_loc: i32,
    resolution: (u32, u32),
    fs_time_loc: i32,
    time: Duration
) {
    let (width, height) = resolution;
    let time_secs = 0.001 * (time.num_milliseconds() as GLfloat);
    unsafe {
        // Pass resolution & time uniforms to shader
        gl::Uniform2f(fs_resolution_loc, width as GLfloat, height as GLfloat);
        gl::Uniform1f(fs_time_loc, time_secs as GLfloat);

        // Clear the screen to black
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        // Draw
        gl::DrawArrays(gl::TRIANGLE_FAN, 0, (vertices.len()) as i32);
    };
}

fn main() {
    let (dimensions_opt, fullscreen_monitor_opt) =
        match options::get_options() {
            Ok(opts) => opts,
            Err(msg) => {
                println!("{}", msg);
                return;
            }
        };
    
    let vertices: [GLfloat; 12] = [
        -1.0, -1.0, 0.0,
         1.0, -1.0, 0.0,
         1.0,  1.0, 0.0,
        -1.0,  1.0, 0.0
    ];
    
    let window = init_window(dimensions_opt, fullscreen_monitor_opt);
    
    let (major, minor) = gl_version();
    println!("OpenGL version: {}.{}", major, minor);

    let (vs, fs) = load_shaders();
    let program = init_rendering(&vs, &fs, &vertices);
    let fs_resolution_loc = program.get_fragment_uniform(U_RESOLUTION).unwrap();
    let fs_time_loc = program.get_fragment_uniform(U_TIME).unwrap();
    
    let start_time = SteadyTime::now();

    loop {
        let end = handle_window_events(&window);
        if end {
            break;
        }

        let resolution = match window.get_inner_size() {
            Some(res) => res,
            None      => (0, 0)
        };
        
        let time = SteadyTime::now() - start_time;
        render(
            &vertices,
            fs_resolution_loc, resolution,
            fs_time_loc, time
        );
        
        let _ = window.swap_buffers();
    }
}
