use super::parse_dconf;
use crate::{run, Mode, Result};

pub fn get() -> Result<String> {
    parse_dconf(
        "dconf",
        &[
            "read",
            "/com/deepin/wrap/gnome/desktop/background/picture-uri",
        ],
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
            "/com/deepin/wrap/gnome/desktop/background/picture-uri",
            &enquote::enquote('"', &format!("file://{}", &path)),
        ],
    )
}

pub fn set_mode(mode: Mode) -> Result<()> {
    run(
        "dconf",
        &[
            "write",
            "/com/deepin/wrap/gnome/desktop/background/picture-options",
            &mode.get_gnome_string(),
        ],
    )
}
