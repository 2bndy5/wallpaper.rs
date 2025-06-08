use super::parse_dconf;
use crate::{run, Error, Mode, Result};

pub fn get() -> Result<String> {
    parse_dconf(
        "dconf",
        &["read", "/org/mate/desktop/background/picture-filename"],
    )
}

pub fn set<P>(path: P) -> Result<()>
where
    P: AsRef<std::path::Path> + std::fmt::Display,
{
    run(
        "dconf",
        &[
            "write",
            "/org/mate/desktop/background/picture-filename",
            &enquote::enquote('"', path.as_ref().to_str().ok_or(Error::InvalidPath)?),
        ],
    )
}

pub fn set_mode(mode: Mode) -> Result<()> {
    run(
        "dconf",
        &[
            "write",
            "/org/mate/desktop/background/picture-options",
            &mode.get_gnome_string(),
        ],
    )
}
