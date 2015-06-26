extern crate gl;

use self::gl::types::*;
use std::ptr;
use std::str;
use std::ffi::CString;

pub struct Shader {
    pub id: GLuint,
    pub kind: GLenum
}

impl Shader {
    pub fn new(src: &str, kind: GLenum) -> Shader {
        unsafe {
            let shader = gl::CreateShader(kind);
            
            // Attempt to compile the shader
            let c_str = CString::new(src.as_bytes()).unwrap();
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            // Get the compile status
            let mut status = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
                panic!("{}", str::from_utf8(&buf).ok().expect("ShaderInfoLog not valid utf8"));
            }
            
            Shader {
                id: shader,
                kind: kind
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id); }
    }
}

pub struct Program<'a> {
    pub id: GLuint,
    pub vertex_shader: &'a Shader,
    pub fragment_shader: &'a Shader
}

impl<'a> Program<'a> {
    pub fn new(vertex_shader: &'a Shader, fragment_shader: &'a Shader) -> Program<'a> {
        unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader.id);
            gl::AttachShader(program, fragment_shader.id);
            gl::LinkProgram(program);
            
            // Get the link status
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                let mut len: GLint = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
                panic!("{}", str::from_utf8(&buf).ok().expect("ProgramInfoLog not valid utf8"));
            }
            
            Program {
                id: program,
                vertex_shader: vertex_shader,
                fragment_shader: fragment_shader
            }
        }
    }
    
    pub fn enable(&self) {
        unsafe { gl::UseProgram(self.id); }
    }
}

impl<'a> Drop for Program<'a> {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id); };
    }
}
