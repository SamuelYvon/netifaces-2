use pyo3::exceptions::PyTypeError;
use pyo3::PyErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetifacesError {
    #[error("Failed to use a system function (module {0}, error code {1})")]
    SystemErrorCode(String, u32),
}

/// Controls what is the interface name returned to the user.
pub enum InterfaceDisplay {
    HumanReadable = 0,
    MachineReadable = 1,
}

impl TryFrom<i32> for InterfaceDisplay {
    type Error = PyErr;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(InterfaceDisplay::HumanReadable),
            1 => Ok(InterfaceDisplay::MachineReadable),
            _ => Err(PyTypeError::new_err("Error message")),
        }
    }
}
