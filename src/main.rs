extern crate gl;
extern crate glmoi;
extern crate glutin;
extern crate libc;

use gl::types::*;
use std::mem;
use std::ptr;
use std::ffi::CString;
use glmoi::shaders::{Program, Shader};

static VS_SRC: &'static str =
   "#version 150\n\
    in vec3 position;\n\
    void main() {\n\
       gl_Position = vec4(position, 1.0);\n\
    }";

static FS_SRC: &'static str =
   "#version 150\n\
    uniform vec2 u_resolution;\n\
    out vec4 out_color;\n\
    void main() {\n\
       out_color = vec4(gl_FragCoord.x/u_resolution.x, 0.0, gl_FragCoord.y/u_resolution.y, 1.0);\n\
    }";

fn str_ptr(s: &str) -> *const i8 {
    CString::new(s).unwrap().as_ptr()
}

fn main() {
    // Get the first available monitor
    let monitor = glutin::get_available_monitors().nth(0).unwrap();    
    
    // Construct a window
    let wb = glutin::WindowBuilder::new()
             .with_title("glmoi".to_string())
             .with_dimensions(1280, 720)
             //.with_fullscreen(monitor)
             //.with_vsync()
             ;
    let window = wb.build().unwrap();
    
    // Initialize GL
    let _ = unsafe { window.make_current() };
    gl::load_with(|symbol| window.get_proc_address(symbol));
    
    // Compile and link shaders
    let vs = Shader::new(VS_SRC, gl::VERTEX_SHADER);
    let fs = Shader::new(FS_SRC, gl::FRAGMENT_SHADER);
    let program = Program::new(&vs, &fs);

    let fs_resolution_loc = unsafe { gl::GetUniformLocation(program.id, str_ptr("u_resolution")) };

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

    for event in window.wait_events() {
        unsafe {
            let (width, height) = match window.get_inner_size() {
                Some(sz) => sz,
                None     => (0, 0)
            };
            gl::Uniform2f(fs_resolution_loc, width as GLfloat, height as GLfloat);
                        
            // Clear the screen to black
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw
            gl::DrawArrays(gl::TRIANGLE_FAN, 0, (vertices.len()) as i32);
            
        };
        let _ = window.swap_buffers();

        match event {
            glutin::Event::Closed => break,
            glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Escape)) => break,
            _ => println!("{:?}", event)
        }
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
