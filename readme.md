# wallpaper

This Rust library gets and sets the desktop wallpaper/background.

This project is actually a customized fork of [reujab/wallpaper.rs](https://github.com/reujab/wallpaper.rs).

The supported desktops are:

- Windows
- macOS (cannot change wallpaper mode/style)
- GNOME
- KDE
- Cinnamon
- Unity
- Budgie
- XFCE
- LXDE
- MATE
- Deepin
- Most Wayland compositors (set only, requires `swaybg`)
- i3 (set only, requires `feh`)

## Examples

```rust
use wallpaper::{DesktopClient, DesktopWallpaper, Mode};

// init interface
let mut client = DesktopWallpaper::new().unwrap();

// Returns the wallpaper of the current desktop.
match client.get_wallpaper() {
    Ok(img_path) => println!("{img_path:?}"),
    Err(e) => {
        assert!(matches!(e, wallpaper::Error::UnsupportedDesktop))
    }
}

assert!(
    // Sets the wallpaper for the current desktop from a file path.
    client.set_wallpaper(
        "/path/to/picture.png",
        // Also sets the wallpaper mode (crop, center, fit, span, etc).
        Some(Mode::Stretch)
    )
    .is_err_and(|e| matches!(e, wallpaper::Error::InvalidPath))
);
```

## Features

### `"winrs"`

This feature uses the Windows SDK to access wallpaper information.
Applies to Windows platform only.
This feature is enabled by default.

When the [`"winreg"` feature](#winreg) is enabled,
the default `"winrs"` feature should be explicitly disabled with `default-features = false`.
Otherwise needless `windows` crate features are enabled (unused dependencies).

### `"winreg"`

This feature uses the Windows system Registry to access wallpaper information.
Applies to Windows platform only.
This restores the legacy behavior in v4.0.0 (unreleased upstream) and earlier.
However, this approach often

- had trouble with virtual desktops (a different registry key perhaps)
- didn't stick after reboot (probably Windows theme sync related)

By default, this feature is disabled.
With this feature disabled, the problems listed above no longer occur.
Instead, using the Windows SDK (the [`"winrs"` feature](#winrs) enabled by default),
the given image is now set as the desktop wallpaper for every monitor.
