# Hyperview

Hyperview is a lightweight, polished, open-source image gallery and viewer for Linux, built natively for Wayland with Rust and GTK4/Libadwaita.

## Features
- **Native Wayland/GTK4 UI:** Premium Libadwaita styling with seamless dark mode support.
- **Smart Directory Scanning:** Open an image and instantly navigate through siblings in the same folder.
- **Slideshow:** Built-in auto-advancing slideshow with looping.
- **Distraction-Free:** Fullscreen mode and a minimalist UI.
- **Pointer-Centric Zoom:** Smooth Ctrl+Scroll and Double-Click zoom that tracks your cursor.
- **Safe File Operations:** Reveal in folder, copy path, and safely delete images.

## Keyboard Shortcuts
- `Left Arrow` / `Right Arrow`: Previous / Next image
- `Ctrl + O`: Open file dialog
- `F`: Toggle Fullscreen
- `S`: Toggle Slideshow (3-second interval)
- `G`: Toggle between Gallery Grid and Image Viewer
- `0`: Reset Zoom (Fit to window)
- `Ctrl + Mouse Scroll`: Zoom In / Out
- `Double Click`: Toggle Zoom / Fit

## Installation on Arch Linux

## Download or copy PKGBUILD into a clean build directory
```
mkdir -p /tmp/hyperview-build
curl -sSL [https://raw.githubusercontent.com/srikanthpusunuri1234/Hyperview/main/PKGBUILD](https://raw.githubusercontent.com/srikanthpusunuri1234/Hyperview/main/PKGBUILD) -o /tmp/hyperview-build/PKGBUILD
```
## Build and install the Arch package
```
cd /tmp/hyperview-build
makepkg -si
```
