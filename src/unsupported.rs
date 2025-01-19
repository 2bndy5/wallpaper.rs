use error::Error;
use Result;

pub fn get() -> Result<String> {
    Err(Error::UnsupportedDesktop)
}

pub fn set_from_path(_: &str) -> Result<()> {
    Err(Error::UnsupportedDesktop)
}

pub fn set_mode(_: Mode) -> Result<()> {
    Err(Error::UnsupportedDesktop)
}
