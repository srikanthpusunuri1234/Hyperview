pkgname=hyperview
pkgver=0.1.0
pkgrel=1
pkgdesc="A lightweight, polished, open-source image gallery and viewer for Linux"
arch=('x86_64')
url="https://github.com/srikanthpusunuri1234/Hyperview"
license=('GPL3')
depends=('gtk4' 'libadwaita')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/srikanthpusunuri1234/Hyperview/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  # Note the capital H here to match GitHub's extracted folder name
  cd "Hyperview-$pkgver"
  cargo build --release --locked
}

package() {
  # Note the capital H here as well
  cd "Hyperview-$pkgver"
  
  # Install the executable
  install -Dm755 "target/release/hyperview" "$pkgdir/usr/bin/hyperview"
  
  # Install the desktop entry
  install -Dm644 "hyperview.desktop" "$pkgdir/usr/share/applications/hyperview.desktop"
  
  # Install the application icon
  install -Dm644 "hyperview.svg" "$pkgdir/usr/share/icons/hicolor/scalable/apps/hyperview.svg"
}
