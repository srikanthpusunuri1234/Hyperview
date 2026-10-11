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

## Installation (Arch Linux)
Clone the repository and build the package using `makepkg`:
```bash
git clone https://github.com/srikanthpusunuri1234/Hyperview.git
cd Hyperview
makepkg -si
