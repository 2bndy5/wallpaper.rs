mod deepin;
mod gnome;
mod kde;
mod lxde;
mod mate;
mod x_cinnamon;
pub(crate) mod xfce;

use crate::{get_stdout, run, DesktopClient, Error, Mode, Result};
use std::{env, path::Path, process::Command};

pub struct DesktopWallpaper {
    distro_flavor: String,
}

impl DesktopWallpaper {
    pub fn new() -> Self {
        Self {
            distro_flavor: env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
        }
    }
}

pub struct Monitor(usize);

impl From<usize> for Monitor {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl DesktopClient for DesktopWallpaper {
    fn set_wallpaper<M>(&mut self, monitor: M, path: &str, mode: Mode) -> Result<()>
    where
        M: Into<Monitor>,
    {
        let _ = monitor;
        let path = PathBuf::from(path);

        if gnome::is_compliant(&self.distro_flavor) {
            gnome::set_mode(mode)?;
            return gnome::set(&path);
        }

        match self.distro_flavor.as_str() {
            "KDE" => {
                kde::set_mode(mode)?;
                kde::set(&path)
            }
            "X-Cinnamon" => {
                x_cinnamon::set_mode(mode)?;
                x_cinnamon::set(&path)
            }
            "MATE" => {
                mate::set_mode(mode)?;
                mate::set(&path)
            }
            "XFCE" => {
                xfce::set_mode(mode)?;
                xfce::set(path)
            }
            "LXDE" => {
                lxde::set_mode(mode)?;
                lxde::set(path)
            }
            "Deepin" => {
                deepin::set_mode(mode)?;
                deepin::set(&path)
            }
            _ => {
                // unable to set mode because feature is not supportedon cirrent desktop.
                // just try to set the wallpaper instead.

                if let Ok(mut child) = Command::new("swaybg")
                    .args(["-i", path.as_ref().to_str().ok_or(Error::InvalidPath)?])
                    .spawn()
                {
                    child.stdout = None;
                    child.stderr = None;
                    return Ok(());
                }

                run(
                    "feh",
                    &[
                        "--bg-fill",
                        path.as_ref().to_str().ok_or(Error::InvalidPath)?,
                    ],
                )
            }
        }
    }

    fn get_wallpaper<M>(&self, monitor: M) -> Result<String>
    where
        M: Into<Monitor>,
    {
        let _ = monitor;

        if gnome::is_compliant(self.distro_flavor.as_str) {
            return gnome::get();
        }

        match self.distro_flavor.as_str() {
            "KDE" => kde::get(),
            "X-Cinnamon" => x_cinnamon::get(),
            "MATE" => mate::get(),
            "XFCE" => xfce::get(),
            "LXDE" => lxde::get(),
            "Deepin" => deepin::get(),
            _ => Err(Error::UnsupportedDesktop),
        }
    }
}

/// Sets the wallpaper for the current desktop from a file path.
pub fn set_from_path<P>(path: P) -> Result<()>
where
    P: AsRef<Path> + std::fmt::Display,
{
    let desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();

    if gnome::is_compliant(&desktop) {
        return gnome::set(&path);
    }

    match desktop.as_str() {
        "KDE" => kde::set(&path),
        "X-Cinnamon" => run(
            "dconf",
            &[
                "write",
                "/org/cinnamon/desktop/background/picture-uri",
                &enquote::enquote('"', &format!("file://{}", &path)),
            ],
        ),
        "MATE" => run(
            "dconf",
            &[
                "write",
                "/org/mate/desktop/background/picture-filename",
                &enquote::enquote('"', path.as_ref().to_str().ok_or(Error::InvalidPath)?),
            ],
        ),
        "XFCE" => xfce::set(path),
        "LXDE" => lxde::set(path),
        "Deepin" => run(
            "dconf",
            &[
                "write",
                "/com/deepin/wrap/gnome/desktop/background/picture-uri",
                &enquote::enquote('"', &format!("file://{}", &path)),
            ],
        ),
        _ => {
            if let Ok(mut child) = Command::new("swaybg")
                .args(["-i", path.as_ref().to_str().ok_or(Error::InvalidPath)?])
                .spawn()
            {
                child.stdout = None;
                child.stderr = None;
                return Ok(());
            }

            run(
                "feh",
                &[
                    "--bg-fill",
                    path.as_ref().to_str().ok_or(Error::InvalidPath)?,
                ],
            )
        }
    }
}

fn parse_dconf(command: &str, args: &[&str]) -> Result<String> {
    let mut stdout = enquote::unquote(&get_stdout(command, args)?)?;
    // removes file protocol
    if stdout.starts_with("file://") {
        stdout = stdout[7..].into();
    }
    Ok(stdout)
}
