# wallpaper

This Rust library gets and sets the desktop wallpaper/background.

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

fn main() {
    // init interface
    let mut client = DesktopWallpaper::new().unwrap();

    // Returns the wallpaper of the current desktop.
    println!("{:?}", client.get_wallpaper().unwrap());

    assert!(
        // Sets the wallpaper for the current desktop from a file path.
        client.set_wallpaper(
            "/path/to/picture.png",
            // Also sets the wallpaper mode (crop, center, fit, span, etc).
            Mode::Stretch
        )
        .is_err_and(|e| matches!(e, wallpaper::Error::InvalidPath))
    );

    drop(client);
}
```
