#!/bin/sh
# Installs the emmurt CLI from the latest GitHub release.
#
#   curl -fsSL https://raw.githubusercontent.com/bitoesposito/htmlemmurt/main/install.sh | sh
#
# Environment:
#   EMMURT_VERSION      release to install, e.g. v0.1.0 (default: latest)
#   EMMURT_INSTALL_DIR  where to put the binary (default: $HOME/.local/bin)

set -eu

REPO="bitoesposito/htmlemmurt"
INSTALL_DIR="${EMMURT_INSTALL_DIR:-$HOME/.local/bin}"

say() { printf '%s\n' "$*"; }
fail() { printf 'emmurt: %s\n' "$*" >&2; exit 1; }

command -v curl >/dev/null 2>&1 || fail "serve curl pe' scaricà emmurt (curl is required)"
command -v tar >/dev/null 2>&1 || fail "serve tar (tar is required)"

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64 | Linux-amd64) target="x86_64-unknown-linux-gnu" ;;
  Linux-aarch64 | Linux-arm64) target="aarch64-unknown-linux-gnu" ;;
  Darwin-arm64 | Darwin-aarch64) target="aarch64-apple-darwin" ;;
  Darwin-x86_64) target="x86_64-apple-darwin" ;;
  *)
    fail "nun ce sta ancora nu binario pe' $(uname -s) $(uname -m). Compilalo tu:
  cargo install --git https://github.com/$REPO emmurt-cli"
    ;;
esac

version="${EMMURT_VERSION:-}"
if [ -z "$version" ]; then
  # /releases/latest redirects to /releases/tag/<tag>: no API token needed
  latest="$(curl -fsSLI -o /dev/null -w '%{url_effective}' "https://github.com/$REPO/releases/latest")" \
    || fail "nun riesco a truvà l'urdema release"
  version="${latest##*/}"
fi
case "$version" in
  v*) ;;
  *) version="v$version" ;;
esac

name="emmurt-$version-$target"
base="https://github.com/$REPO/releases/download/$version"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

say "Scarico emmurt $version ($target)…"
curl -fsSL "$base/$name.tar.gz" -o "$tmp/$name.tar.gz" || fail "download fallito: $base/$name.tar.gz"
curl -fsSL "$base/SHA256SUMS.txt" -o "$tmp/SHA256SUMS.txt" || fail "nun trovo SHA256SUMS.txt"
expected="$(grep " $name.tar.gz\$" "$tmp/SHA256SUMS.txt" | cut -d' ' -f1)"
if command -v sha256sum >/dev/null 2>&1; then
  actual="$(sha256sum "$tmp/$name.tar.gz" | cut -d' ' -f1)"
else
  actual="$(shasum -a 256 "$tmp/$name.tar.gz" | cut -d' ' -f1)"
fi
[ -n "$expected" ] && [ "$expected" = "$actual" ] || fail "checksum sbagliato pe' $name.tar.gz"

tar -xzf "$tmp/$name.tar.gz" -C "$tmp"
mkdir -p "$INSTALL_DIR"
cp "$tmp/$name/emmurt" "$INSTALL_DIR/emmurt"
chmod +x "$INSTALL_DIR/emmurt"

say "Tutto a posto 🤌 emmurt sta in $INSTALL_DIR/emmurt"
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) say "Aggiungi $INSTALL_DIR al PATH, per esempio: export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac
