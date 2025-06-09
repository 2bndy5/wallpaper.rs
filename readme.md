# wallpaper

This Rust library gets and sets the desktop wallpaper/background.

This project is actually a customized fork of [reujab/wallpaper.rs](https://github.com/reujab/wallpaper.rs).

The supported desktops are:

- Windows
- macOS
- GNOME
- KDE
- Cinnamon
- Unity
- Budgie
- XFCE
- LXDE
- MATE
- Deepin
- Most Wayland compositors (set only, requires swaybg)
- i3 (set only, requires feh)

## Examples

```rust
use wallpaper::{DesktopClient, DesktopWallpaper, Mode};

// init interface
let mut client = DesktopWallpaper::new().unwrap();

// Returns the wallpaper of the current desktop.
match client.get_wallpaper() {
    Ok(wallpaper) => println!("{wallpaper:?}"),
    Err(e) => {
        assert!(matches!(e, wallpaper::Error::UnsupportedDesktop))
    }
}

assert!(
    // Sets the wallpaper for the current desktop from a file path.
    client.set_wallpaper(
        "/path/to/picture.png",
        // Also sets the wallpaper mode (crop, center, fit, span, etc).
        Mode::Stretch
    )
    .is_err_and(|e| matches!(e, wallpaper::Error::InvalidPath))
);
```
