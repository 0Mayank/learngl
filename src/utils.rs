use gl::types::GLsizei;

/// # Safety
/// shader_id should be valid
pub unsafe fn check_shader_succes(shader_id: u32, pname: gl::types::GLenum) -> Result<(), String> {
    let mut success = 42;
    let mut info_log = vec![0; 512];
    let mut length: i32 = 42;

    gl::GetShaderiv(shader_id, pname, &mut success);
    if success != gl::TRUE.into() {
        gl::GetShaderInfoLog(
            shader_id,
            512,
            &mut length as *mut i32 as *mut GLsizei,
            info_log.as_mut_ptr() as *mut gl::types::GLchar,
        );
        return Err(String::from_utf8_lossy(&info_log[..length as usize]).to_string());
    }

    Ok(())
}

/// # Safety
/// pid should be valid
pub unsafe fn check_program_success(pid: u32, pname: gl::types::GLenum) -> Result<(), String> {
    let mut success = 42;
    let mut info_log = vec![0; 512];
    let mut length: i32 = 42;

    gl::GetProgramiv(pid, pname, &mut success);
    if success != gl::TRUE.into() {
        gl::GetProgramInfoLog(
            pid,
            512,
            &mut length as *mut i32 as *mut GLsizei,
            info_log.as_mut_ptr() as *mut gl::types::GLchar,
        );
        return Err(String::from_utf8_lossy(&info_log[..length as usize]).to_string());
    }

    Ok(())
}
