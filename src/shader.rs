use std::{
    ffi::CString,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    errors::{GLWError, GLWErrorKind},
    utils,
};

#[derive(Debug, Clone, Copy)]
pub enum ShaderType {
    VertexShader,
    FragmentShader,
}

impl ShaderType {
    pub fn from_path(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        let ext = path.extension().ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No extension found on path.",
        ))?;

        Self::from_ext(&ext.to_string_lossy())
    }

    pub fn from_ext(ext: &str) -> std::io::Result<Self> {
        match ext {
            "fs" => Ok(ShaderType::FragmentShader),
            "vs" => Ok(ShaderType::VertexShader),
            ext => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!("\"{}\" extension not supported.", ext),
            )),
        }
    }
}

impl TryFrom<u32> for ShaderType {
    type Error = std::io::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            gl::VERTEX_SHADER => Ok(Self::VertexShader),
            gl::FRAGMENT_SHADER => Ok(Self::FragmentShader),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "This type of shader is not supported.",
            )),
        }
    }
}

impl From<ShaderType> for u32 {
    fn from(val: ShaderType) -> Self {
        match val {
            ShaderType::FragmentShader => gl::FRAGMENT_SHADER,
            ShaderType::VertexShader => gl::VERTEX_SHADER,
        }
    }
}

pub struct Shader {
    pub shader_id: u32,
    pub shader_type: ShaderType,
}

impl Shader {
    pub fn from_str(source: impl Into<Vec<u8>>, shader_type: ShaderType) -> Result<Self, GLWError> {
        let shader_str = CString::new(source)?;

        let shader_id = unsafe {
            let shader_id = gl::CreateShader(shader_type.into());
            gl::ShaderSource(shader_id, 1, &shader_str.as_ptr(), std::ptr::null());
            gl::CompileShader(shader_id);
            Self::check_succes(shader_id, None)?;
            shader_id
        };

        Ok(Self {
            shader_id,
            shader_type,
        })
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, GLWError> {
        let path = path.as_ref();
        let shader_str = CString::new(fs::read(path)?)?;
        let shader_type = ShaderType::from_path(path)?;

        let shader_id = unsafe {
            let shader_id = gl::CreateShader(shader_type.into());
            gl::ShaderSource(shader_id, 1, &shader_str.as_ptr(), std::ptr::null());
            gl::CompileShader(shader_id);
            Self::check_succes(shader_id, Some(path))?;
            shader_id
        };

        Ok(Self {
            shader_id,
            shader_type,
        })
    }

    pub fn get_uniform_location(&self, name: impl AsRef<str>) -> Result<i32, GLWError> {
        // TODO: copying?
        let c_name = CString::new(name.as_ref())?;

        let location = unsafe { gl::GetUniformLocation(self.shader_id, c_name.as_ptr()) };
        if location == -1 {
            Err(GLWErrorKind::UniformNotFound(name.as_ref().to_string()))?;
        }

        Ok(location)
    }

    /// # Safety
    /// shader_id should be valid
    pub unsafe fn check_succes(shader_id: u32, path: Option<&Path>) -> Result<(), GLWError> {
        utils::check_shader_succes(shader_id, gl::COMPILE_STATUS).map_err(|info| {
            GLWError::new(
                GLWErrorKind::ShaderCompilationFailed(path.map(|p| p.to_path_buf())),
                info,
            )
        })
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.shader_id);
        }
    }
}

pub struct ShaderProgram {
    shader_program_id: u32,
}

impl ShaderProgram {
    pub fn builder<'a>() -> ShaderProgramBuilder<'a> {
        ShaderProgramBuilder::new()
    }

    pub fn use_program(&self) {
        // SAFETY: this can be only done after shader program is created
        // by the builder, so self.shader_program_id is valid
        unsafe {
            gl::UseProgram(self.shader_program_id);
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        // SAFETY: ShaderProgram can only be created by the builder,
        // so self.shader_program_id is valid
        unsafe {
            gl::DeleteProgram(self.shader_program_id);
        }
    }
}

pub struct ShaderProgramBuilder<'a> {
    shader_paths: Vec<PathBuf>,
    shaders: Vec<&'a Shader>,
}

impl<'a> ShaderProgramBuilder<'a> {
    pub fn new() -> Self {
        Self {
            shader_paths: vec![],
            shaders: vec![],
        }
    }

    pub fn attach_shader_path(mut self, path: impl AsRef<Path>) -> Self {
        self.shader_paths.push(path.as_ref().to_path_buf());
        self
    }

    pub fn attach_shader(mut self, shader: &'a Shader) -> Self {
        self.shaders.push(shader);
        self
    }

    pub fn build(self) -> Result<ShaderProgram, GLWError> {
        let owned_shaders: Vec<Shader> = self
            .shader_paths
            .into_iter()
            .map(Shader::from_path)
            .collect::<Result<_, _>>()?;

        let shader_program_id = unsafe {
            let shader_program_id = gl::CreateProgram();
            self.shaders
                .into_iter()
                .chain(owned_shaders.iter())
                .for_each(|shader| gl::AttachShader(shader_program_id, shader.shader_id));

            gl::LinkProgram(shader_program_id);

            utils::check_program_success(shader_program_id, gl::LINK_STATUS)
                .map_err(|info| GLWError::new(GLWErrorKind::ShaderProgramLinkingFailed, info))?;

            shader_program_id
        };

        Ok(ShaderProgram { shader_program_id })
    }
}

impl Default for ShaderProgramBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}
