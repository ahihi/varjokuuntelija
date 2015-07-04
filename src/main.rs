extern crate demonplayer;
extern crate gl;
extern crate glutin;

extern crate glmoi;

use demonplayer::Demonplayer;
use gl::types::*;
use glutin::ElementState::*;
use glutin::Event::*;
use glutin::VirtualKeyCode::*;
use std::mem;
use std::path::Path;
use std::ptr;
use std::ffi::CString;

use glmoi::shaders::{Program, Shader};

static VS_SRC: &'static str = include_str!("glsl/default.vert");
static FS_SRC: &'static str = include_str!("glsl/default.frag");

fn str_ptr(s: &str) -> *const i8 {
    CString::new(s).unwrap().as_ptr()
}

fn gl_version() -> (GLint, GLint) {
    let mut major: GLint = -1;
    unsafe { gl::GetIntegerv(gl::MAJOR_VERSION, &mut major) };
    let mut minor: GLint = -1;
    unsafe { gl::GetIntegerv(gl::MINOR_VERSION, &mut minor) };
    (major, minor)
}

fn main() {
    // Set up audio
    let player = Demonplayer::from_flac(Path::new("music.flac"))
                 .unwrap_or_else(|e| {
                     panic!("demonplayer init failed: {:?}", e);
                 });

    println!("");
    println!("Sample rate: {}", player.sample_rate());
    println!("Bit depth: {}", player.bit_depth());
    println!("Channels: {}", player.channels());
    println!("Samples: {}", player.n_samples());
    println!("Duration: {} s", player.duration());

    println!("");
    println!("Playing");
    let _ = player.play();

    /*let default_host = pa::host::get_default_api();
    println!("PA host: {}", default_host);
    let api_info = pa::host::get_api_info(default_host);
    let api_info_str = match api_info {
        None       => "N/A".to_string(),
        Some(info) => info.name,
    };
    println!("PA API info: {}", api_info_str);*/

    // Get the first available monitor
    let _monitor = glutin::get_available_monitors().nth(0).unwrap();

    // Construct a window
    let wb = glutin::WindowBuilder::new()
             .with_title("glmoi".to_string())
             .with_dimensions(1280, 720)
             //.with_fullscreen(_monitor)
             .with_vsync()
             .with_gl(glutin::GlRequest::Latest)
             .with_gl_profile(glutin::GlProfile::Core)
             ;
    let window = wb.build().unwrap();
    let _ = unsafe { window.make_current() };
    let _ = window.set_cursor_state(glutin::CursorState::Hide);

    // Initialize GL
    gl::load_with(|symbol| window.get_proc_address(symbol));

    let (major, minor) = gl_version();
    println!("OpenGL version: {}.{}", major, minor);

    // Compile and link shaders
    let vs = Shader::new(VS_SRC, gl::VERTEX_SHADER);
    let fs = Shader::new(FS_SRC, gl::FRAGMENT_SHADER);
    let program = Program::new(&vs, &fs);

    let fs_resolution_loc = unsafe {
        gl::GetUniformLocation(program.id, str_ptr("u_resolution"))
    };
    let fs_time_loc = unsafe {
        gl::GetUniformLocation(program.id, str_ptr("u_time"))
    };

    let mut vao = 0;
    let mut vbo = 0;

    let vertices: [GLfloat; 12] = [
        -1.0, -1.0, 0.0,
         1.0, -1.0, 0.0,
         1.0,  1.0, 0.0,
        -1.0,  1.0, 0.0
    ];

    unsafe {
        // Create Vertex Array Object
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        // Create a Vertex Buffer Object and copy the vertex data to it
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       mem::transmute(&vertices[0]),
                       gl::STATIC_DRAW);

        // Use shader program
        program.enable();
        gl::BindFragDataLocation(program.id, 0, str_ptr("out_color"));

        // Specify the layout of the vertex data
        let pos_attr = gl::GetAttribLocation(program.id, str_ptr("position"));
        gl::EnableVertexAttribArray(pos_attr as GLuint);
        gl::VertexAttribPointer(pos_attr as GLuint, 3, gl::FLOAT,
                                gl::FALSE as GLboolean, 0, ptr::null());
    }

    loop {
        let mut end = false;
        for event in window.poll_events() {
            match event {
                Closed
                    => { end = true; },
                KeyboardInput(Pressed, _, Some(Escape))
                    => { end = true; },
                _
                    => println!("{:?}", event)
            };
        }

        if end {
            break;
        }

        let position = match player.position() {
            None    => { break; },
            Some(p) => p
        };

        unsafe {
            let (width, height) = match window.get_inner_size() {
                Some(sz) => sz,
                None     => (0, 0)
            };
            gl::Uniform2f(fs_resolution_loc, width as GLfloat, height as GLfloat);
            gl::Uniform1f(fs_time_loc, position as GLfloat);

            // Clear the screen to black
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw
            gl::DrawArrays(gl::TRIANGLE_FAN, 0, (vertices.len()) as i32);

        };
        let _ = window.swap_buffers();
    }
}

/*fn main2() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // Choose a GL profile that is compatible with OS X 10.7+
    glfw.window_hint(WindowHint::ContextVersion(3, 2));
    glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));

    let (mut window, _) = glfw.create_window(800, 600, "OpenGL", WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    // It is essential to make the context current before calling `gl::load_with`.
    window.make_current();

    unsafe {
    // Cleanup
        gl::DeleteProgram(program);
        gl::DeleteShader(fs);
        gl::DeleteShader(vs);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
    }
}*/
