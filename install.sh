#!/bin/sh
# Install loam from a GitHub release, without Rust.
#
#   curl -fsSL https://raw.githubusercontent.com/oddurs/loam/main/install.sh | sh
#
# LOAM_VERSION   a release tag, as v0.3.0; the latest release when unset
# LOAM_DIR       where to put the binary; ~/.local/bin when unset
#
# Linux (x86_64, aarch64) and macOS (Apple silicon, Intel). Anywhere else:
# cargo install --locked --git https://github.com/oddurs/loam
set -eu

repo=oddurs/loam
version=${LOAM_VERSION:-latest}
dir=${LOAM_DIR:-"$HOME/.local/bin"}

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) target=x86_64-unknown-linux-musl ;;
  Linux-aarch64 | Linux-arm64) target=aarch64-unknown-linux-musl ;;
  Darwin-arm64) target=aarch64-apple-darwin ;;
  Darwin-x86_64) target=x86_64-apple-darwin ;;
  *)
    echo "loam: no release is built for $(uname -s) $(uname -m);" >&2
    echo "      cargo install --locked --git https://github.com/$repo" >&2
    exit 1
    ;;
esac

if [ "$version" = latest ]; then
  url=https://github.com/$repo/releases/latest/download/loam-$target.tar.gz
else
  url=https://github.com/$repo/releases/download/$version/loam-$target.tar.gz
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
fetch() {
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$1" -o "$2"
  else
    wget -q "$1" -O "$2"
  fi
}
fetch "$url" "$tmp/loam.tar.gz" || {
  echo "loam: could not download $url" >&2
  exit 1
}
# The checksum published beside the archive, when there is a tool to check it.
if fetch "$url.sha256" "$tmp/loam.tar.gz.sha256" 2>/dev/null; then
  want=$(cut -d' ' -f1 "$tmp/loam.tar.gz.sha256")
  if command -v sha256sum >/dev/null 2>&1; then
    have=$(sha256sum "$tmp/loam.tar.gz" | cut -d' ' -f1)
  elif command -v shasum >/dev/null 2>&1; then
    have=$(shasum -a 256 "$tmp/loam.tar.gz" | cut -d' ' -f1)
  else
    have=$want
  fi
  if [ "$have" != "$want" ]; then
    echo "loam: the download does not match its checksum; nothing installed" >&2
    exit 1
  fi
fi

tar xzf "$tmp/loam.tar.gz" -C "$tmp"
mkdir -p "$dir"
mv "$tmp/loam-$target/loam" "$dir/loam"
chmod +x "$dir/loam"
echo "installed $("$dir/loam" --version) to $dir/loam"
case ":$PATH:" in
  *":$dir:"*) ;;
  *) echo "note: $dir is not on your PATH" ;;
esac
