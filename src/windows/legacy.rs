use super::DesktopWallpaper;
use crate::{Error, Mode, Result};
use std::{
    ffi::{c_void, OsStr},
    iter, mem,
    os::windows::ffi::OsStrExt,
    path::PathBuf,
};
use windows::{
    core,
    Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_GETDESKWALLPAPER,
        SPI_SETDESKWALLPAPER, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    },
};
use windows_registry::CURRENT_USER;

impl DesktopWallpaper {
    pub fn new() -> core::Result<Self> {
        Ok(Self {})
    }
}

pub fn get() -> Result<String> {
    unsafe {
        let buffer: [u16; 260] = mem::zeroed();
        SystemParametersInfoW(
            SPI_GETDESKWALLPAPER,
            buffer.len() as u32,
            Some(buffer.as_ptr() as *mut c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
        .map_err(|e| windows::core::Error::from_hresult(e.into()))?;

        let path = String::from_utf16(&buffer)
            .map_err(|_| Error::InvalidPath)?
            // removes trailing zeroes from buffer
            .trim_end_matches('\x00')
            .into();
        Ok(path)
    }
}

pub fn set(path: &PathBuf, mode: Mode) -> Result<()> {
    // set wallpaper mode
    let hkcu = CURRENT_USER
        .create("Control Panel\\Desktop")
        .map_err(|_e| Error::UnsupportedDesktop)?;
    hkcu.set_string(
        "TileWallpaper",
        &match mode {
            Mode::Tile => "1",
            _ => "0",
        }
        .to_string(),
    )
    .map_err(|e| windows::core::Error::from_hresult(e.into()))?;

    // copied from https://searchfox.org/mozilla-central/rev/5e955a47c4af398e2a859b34056017764e7a2252/browser/components/shell/nsWindowsShellService.cpp#493
    hkcu.set_string(
        "WallpaperStyle",
        &match mode {
            // does not work with integers
            Mode::Center | Mode::Tile => "0",
            Mode::Fit => "6",
            Mode::Span => "22",
            Mode::Stretch => "2",
            Mode::Crop => "10",
        }
        .to_string(),
    )
    .map_err(|e| windows::core::Error::from_hresult(e.into()))?;

    // set wallpaper
    unsafe {
        let path = OsStr::new(&path)
            .encode_wide()
            // append null byte
            .chain(iter::once(0))
            .collect::<Vec<u16>>();
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(path.as_ptr() as *mut c_void),
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        )
        .map_err(|e| windows::core::Error::from_hresult(e.into()))?;
    }
    Ok(())
}
