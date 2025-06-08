mod deepin;
mod gnome;
mod kde;
mod lxde;
mod mate;
mod x_cinnamon;
pub(crate) mod xfce;

use crate::{get_stdout, run, DesktopClient, Error, Mode, Result};
use std::{env, process::Command};

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

impl Drop for DesktopWallpaper {
    fn drop(&mut self) {}
}

impl DesktopClient for DesktopWallpaper {
    fn set_wallpaper(&mut self, path: &str, mode: Mode) -> Result<()> {
        if gnome::is_compliant(&self.distro_flavor) {
            gnome::set_mode(mode)?;
            return gnome::set(path);
        }

        match self.distro_flavor.as_str() {
            "KDE" => {
                kde::set_mode(mode)?;
                kde::set(path)
            }
            "X-Cinnamon" => {
                x_cinnamon::set_mode(mode)?;
                x_cinnamon::set(path)
            }
            "MATE" => {
                mate::set_mode(mode)?;
                mate::set(path)
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
                deepin::set(path)
            }
            _ => {
                // unable to set mode because feature is not supportedon cirrent desktop.
                // just try to set the wallpaper instead.

                if let Ok(mut child) = Command::new("swaybg").args(["-i", path]).spawn() {
                    child.stdout = None;
                    child.stderr = None;
                    return Ok(());
                }

                run("feh", &["--bg-fill", path])
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
