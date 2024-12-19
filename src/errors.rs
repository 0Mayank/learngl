use std::path::PathBuf;
use thiserror::Error;

pub type Result<T, E = GLWError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
#[error("{kind}\ninfo:\n{info:?}")]
pub struct GLWError {
    kind: GLWErrorKind,
    info: Option<String>,
}

impl GLWError {
    pub fn new(kind: GLWErrorKind, info: impl Into<Option<String>>) -> Self {
        Self {
            kind,
            info: info.into(),
        }
    }

    pub fn info(self, info: String) -> Self {
        Self {
            kind: self.kind,
            info: Some(info),
        }
    }
}

#[derive(Debug, Error)]
pub enum GLWErrorKind {
    #[error("Io Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Shader Compilation Failed for: {0:?}")]
    ShaderCompilationFailed(Option<PathBuf>),
    #[error("Shader Linking Failed")]
    ShaderProgramLinkingFailed,
    #[error(transparent)]
    CStringNulError(#[from] std::ffi::NulError),
}

impl<T> From<T> for GLWError
where
    T: Into<GLWErrorKind>,
{
    fn from(value: T) -> Self {
        Self {
            kind: value.into(),
            info: None,
        }
    }
}

pub trait GLWErrorExt {
    type Ok;

    fn info(self, info: String) -> Result<Self::Ok, GLWError>;
}

impl<O, E> GLWErrorExt for Result<O, E>
where
    E: Into<GLWError>,
{
    type Ok = O;

    fn info(
        self,
        info: String,
    ) -> Result<<std::result::Result<O, E> as GLWErrorExt>::Ok, GLWError> {
        self.map_err(|e| e.into().info(info))
    }
}
