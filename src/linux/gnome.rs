use super::parse_dconf;
use crate::{get_stdout, run, Mode, Result};

#[inline]
pub fn is_compliant(desktop: &str) -> bool {
    desktop.contains("GNOME") || desktop == "Unity" || desktop == "Pantheon"
}

pub fn get() -> Result<String> {
    parse_dconf(
        "gsettings",
        &["get", "org.gnome.desktop.background", "picture-uri"],
    )
}

pub fn set<P>(path: P) -> Result<()>
where
    P: AsRef<std::path::Path> + std::fmt::Display,
{
    let uri = enquote::enquote('"', &format!("file://{}", &path));
    run(
        "gsettings",
        &["set", "org.gnome.desktop.background", "picture-uri", &uri],
    )?;

    // Check if a separate dark mode background URI is available and set it too
    let is_dark_mode_supported = get_stdout(
        "gsettings",
        &[
            "writable",
            "org.gnome.desktop.background",
            "picture-uri-dark",
        ],
    );
    if is_dark_mode_supported.is_ok_and(|v| v.to_lowercase() == "true") {
        // In Gnome < 42 this cmd could fail since key "picture-uri-dark" does not exists
        run(
            "gsettings",
            &[
                "set",
                "org.gnome.desktop.background",
                "picture-uri-dark",
                &uri,
            ],
        )?;
    }
    Ok(())
}

pub fn set_mode(mode: Mode) -> Result<()> {
    run(
        "gsettings",
        &[
            "set",
            "org.gnome.desktop.background",
            "picture-options",
            &mode.get_gnome_string(),
        ],
    )
}

impl Mode {
    pub(crate) fn get_gnome_string(self) -> String {
        enquote::enquote(
            '"',
            match self {
                Mode::Center => "centered",
                Mode::Crop => "zoom",
                Mode::Fit => "scaled",
                Mode::Span => "spanned",
                Mode::Stretch => "stretched",
                Mode::Tile => "wallpaper",
            },
        )
    }
}
