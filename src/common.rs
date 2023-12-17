use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetifacesError {
    #[error("Failed to use a system function (module {0}, error code {1})")]
    SystemErrorCode(String, u32),
}
