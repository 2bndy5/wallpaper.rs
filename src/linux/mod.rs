mod deepin;
mod gnome;
mod kde;
mod lxde;
mod mate;
mod x_cinnamon;
pub(crate) mod xfce;

use crate::{get_stdout, run, DesktopClient, Error, Mode, Result};
use std::{env, path::PathBuf, process::Command};

pub struct DesktopWallpaper {
    distro_flavor: String,
}

impl DesktopWallpaper {
    pub fn new() -> Result<Self> {
        Ok(Self {
            distro_flavor: env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
        })
    }
}

impl DesktopClient for DesktopWallpaper {
    fn set_wallpaper(&mut self, path: &str, mode: Option<Mode>) -> Result<()> {
        let _ = PathBuf::from(path)
            .canonicalize()
            .map_err(|_| Error::InvalidPath)?;

        if gnome::is_compliant(&self.distro_flavor) {
            if let Some(mode) = mode {
                gnome::set_mode(mode)?;
            }
            return gnome::set(path);
        }

        match self.distro_flavor.as_str() {
            "KDE" => {
                if let Some(mode) = mode {
                    kde::set_mode(mode)?;
                }
                kde::set(path)
            }
            "X-Cinnamon" => {
                if let Some(mode) = mode {
                    x_cinnamon::set_mode(mode)?;
                }
                x_cinnamon::set(path)
            }
            "MATE" => {
                if let Some(mode) = mode {
                    mate::set_mode(mode)?;
                }
                mate::set(path)
            }
            "XFCE" => {
                if let Some(mode) = mode {
                    xfce::set_mode(mode)?;
                }
                xfce::set(path)
            }
            "LXDE" => {
                if let Some(mode) = mode {
                    lxde::set_mode(mode)?;
                }
                lxde::set(path)
            }
            "Deepin" => {
                if let Some(mode) = mode {
                    deepin::set_mode(mode)?;
                }
                deepin::set(path)
            }
            _ => {
                // unable to set mode because feature is not supported on current desktop.
                // just try to set the wallpaper instead with `swaybg`;
                // fallback to `feh` if `swaybg` somehow fails.

                if Command::new("swaybg").args(["-i", path]).spawn().is_err() {
                    return run("feh", &["--bg-fill", path]);
                }
                Ok(())
            }
        }
    }

    fn get_wallpaper(&self) -> Result<String> {
        if gnome::is_compliant(self.distro_flavor.as_str()) {
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

fn parse_dconf(command: &str, args: &[&str]) -> Result<String> {
    let mut stdout = enquote::unquote(&get_stdout(command, args)?)?;
    // removes file protocol
    if stdout.starts_with("file://") {
        stdout = stdout[7..].into();
    }
    Ok(stdout)
}
