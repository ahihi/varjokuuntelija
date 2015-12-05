extern crate gl;

extern crate varjokuuntelu;

use gl::types::*;

use varjokuuntelu::Varjokuuntelu;

fn gl_version() -> (GLint, GLint) {
    let mut major: GLint = -1;
    unsafe { gl::GetIntegerv(gl::MAJOR_VERSION, &mut major) };
    let mut minor: GLint = -1;
    unsafe { gl::GetIntegerv(gl::MINOR_VERSION, &mut minor) };
    (major, minor)
}

fn main() {
    let vk = Varjokuuntelu::new();
    
    let (major, minor) = gl_version();
    println!("OpenGL version: {}.{}", major, minor);

    vk.run();
}
