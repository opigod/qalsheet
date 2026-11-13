# Maintainer: QalSheet
pkgname=qalsheet
pkgver=2.0.0
pkgrel=1
pkgdesc="Native Rust scientific calculator, grapher, algebra solver and converter"
arch=('x86_64' 'aarch64')
url="https://github.com/opigod/qalsheet"
license=('MIT')
depends=('glibc' 'libgcc' 'libx11' 'libxcb' 'libxkbcommon' 'wayland' 'libglvnd' 'fontconfig')
makedepends=('rust')
source=()
sha256sums=()

build() {
  cd "$startdir"
  cargo build --release --locked -p qalsheet-desktop
}

package() {
  cd "$startdir"
  install -Dm755 target/release/qalsheet "$pkgdir/usr/bin/qalsheet"
  install -Dm644 packaging/linux/qalsheet.desktop "$pkgdir/usr/share/applications/qalsheet.desktop"
  install -Dm644 packaging/linux/qalsheet.png "$pkgdir/usr/share/icons/hicolor/512x512/apps/qalsheet.png"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  install -Dm644 licenses/nunito-OFL.txt "$pkgdir/usr/share/licenses/$pkgname/OFL.txt"
}
