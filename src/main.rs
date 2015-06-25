extern crate gl;
extern crate glutin;
extern crate libc;

fn main() {
    let window = glutin::Window::new().unwrap();

    unsafe { window.make_current() };

    unsafe {
        gl::load_with(|symbol| window.get_proc_address(symbol));

        gl::ClearColor(0.0, 1.0, 0.0, 1.0);
    }

    for event in window.wait_events() {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT) };
        window.swap_buffers();

        match event {
            glutin::Event::Closed => break,
            _ => ()
        }
    }
}
