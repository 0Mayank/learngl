use std::{
    ffi::{c_void, CString},
    sync::mpsc::Receiver,
};

use gl::types::{GLchar, GLsizei, GLsizeiptr};
use glfw::Context;

const VERTEX_SHADER_SOURCE: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;

    void main() {
        gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
"#;

const FRAGMENT_SHADER_SOURCE_ORANGE: &str = r#"
    #version 330 core
    out vec4 FragColor;

    void main() {
        FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
    }
"#;

const FRAGMENT_SHADER_SOURCE_YELLOW: &str = r#"
    #version 330 core
    out vec4 FragColor;

    void main() {
        FragColor = vec4(1.0f, 1.0f, 0.0f, 1.0f);
    }
"#;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw
        .create_window(800, 600, "LearnOpenGl", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let vertices_1: [f32; 9] = [
        0.5, 0.5, 0.0, // top right
        0.5, -0.5, 0.0, // bottom right
        -0.5, -0.5, 0.0, // bottom left
    ];

    let vertices_2: [f32; 9] = [
        -0.5, 0.5, 0.0, // top left
        -0.5, -0.5, 0.0, // bottom left
        0.5, 0.5, 0.0, // top right
    ];

    let (shader_program_orange, shader_program_yellow, vaos) = unsafe {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let c_vert = CString::new(VERTEX_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(vertex_shader, 1, &c_vert.as_ptr(), std::ptr::null());
        gl::CompileShader(vertex_shader);

        let fragment_shader_orange = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_frag = CString::new(FRAGMENT_SHADER_SOURCE_ORANGE.as_bytes()).unwrap();
        gl::ShaderSource(
            fragment_shader_orange,
            1,
            &c_frag.as_ptr(),
            std::ptr::null(),
        );
        gl::CompileShader(fragment_shader_orange);

        let fragment_shader_yellow = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_frag = CString::new(FRAGMENT_SHADER_SOURCE_YELLOW.as_bytes()).unwrap();
        gl::ShaderSource(
            fragment_shader_yellow,
            1,
            &c_frag.as_ptr(),
            std::ptr::null(),
        );
        gl::CompileShader(fragment_shader_yellow);

        let mut success = 42;
        let mut info_log = vec![0; 512];

        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE.into() {
            gl::GetShaderInfoLog(
                vertex_shader,
                512,
                std::ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            eprintln!(
                "ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}",
                String::from_utf8_lossy(&info_log)
            );
            info_log.clear();
            return;
        }

        gl::GetShaderiv(fragment_shader_orange, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE.into() {
            gl::GetShaderInfoLog(
                fragment_shader_orange,
                512,
                std::ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            eprintln!(
                "ERROR::SHADER::FRAGMENT::ORANGE::COMPILATION_FAILED\n{}",
                String::from_utf8_lossy(&info_log)
            );
            info_log.clear();
            return;
        }

        gl::GetShaderiv(fragment_shader_yellow, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE.into() {
            gl::GetShaderInfoLog(
                fragment_shader_yellow,
                512,
                std::ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            eprintln!(
                "ERROR::SHADER::FRAGMENT::YELLOW::COMPILATION_FAILED\n{}",
                String::from_utf8_lossy(&info_log)
            );
            info_log.clear();
            return;
        }

        let shader_program_orange = gl::CreateProgram();
        gl::AttachShader(shader_program_orange, vertex_shader);
        gl::AttachShader(shader_program_orange, fragment_shader_orange);
        gl::LinkProgram(shader_program_orange);

        let shader_program_yellow = gl::CreateProgram();
        gl::AttachShader(shader_program_yellow, vertex_shader);
        gl::AttachShader(shader_program_yellow, fragment_shader_yellow);
        gl::LinkProgram(shader_program_yellow);

        gl::GetProgramiv(shader_program_orange, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE.into() {
            gl::GetProgramInfoLog(
                shader_program_orange,
                512,
                std::ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            eprintln!(
                "ERROR::PROGRAM::LINKING_FAILED\n{}",
                String::from_utf8_lossy(&info_log)
            );
            info_log.clear();
            return;
        }

        gl::GetProgramiv(shader_program_yellow, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE.into() {
            gl::GetProgramInfoLog(
                shader_program_yellow,
                512,
                std::ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            eprintln!(
                "ERROR::PROGRAM::LINKING_FAILED\n{}",
                String::from_utf8_lossy(&info_log)
            );
            info_log.clear();
            return;
        }

        // We no longer need shader objects after linking them with the
        // program object
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader_orange);
        gl::DeleteShader(fragment_shader_yellow);

        let mut vaos = [0, 0];
        let mut vbos = [0, 0];

        gl::GenVertexArrays(2, vaos.as_mut_ptr());
        gl::GenBuffers(2, vbos.as_mut_ptr());

        gl::BindVertexArray(vaos[0]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[0]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            std::mem::size_of_val(&vertices_1) as GLsizeiptr,
            &vertices_1[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * std::mem::size_of::<gl::types::GLfloat>() as GLsizei,
            std::ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        gl::BindVertexArray(vaos[1]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[1]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            std::mem::size_of_val(&vertices_2) as GLsizeiptr,
            &vertices_2[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * std::mem::size_of::<gl::types::GLfloat>() as GLsizei,
            std::ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        gl::BindVertexArray(0);

        //gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

        (shader_program_orange, shader_program_yellow, vaos)
    };

    while !window.should_close() {
        // handle events
        process_events(&mut window, &events);

        // rendering commands
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(shader_program_orange);
            gl::BindVertexArray(vaos[0]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);

            gl::UseProgram(shader_program_yellow);
            gl::BindVertexArray(vaos[1]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);

            gl::BindVertexArray(0);
        }

        // check and call events and swap the buffers
        window.swap_buffers();
        glfw.poll_events();
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
                gl::Viewport(0, 0, width, height)
            },
            glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                window.set_should_close(true)
            }
            _ => {}
        }
    }
}
