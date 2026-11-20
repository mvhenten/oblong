#!/bin/sh
set -eu

url="https://github.com/mvhenten/oblong/releases/latest/download/oblong-linux-x86_64"
bin_dir="$HOME/.local/bin"
app_dir="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
bin="$bin_dir/oblong"

if [ "$(uname -s)" != "Linux" ] || [ "$(uname -m)" != "x86_64" ]; then
    echo "oblong: only Linux x86_64 is supported" >&2
    exit 1
fi

mkdir -p "$bin_dir" "$app_dir"

tmp="$bin.download.$$"
trap 'rm -f "$tmp"' EXIT
trap 'exit 1' INT TERM

curl -fsSL --connect-timeout 10 --max-time 300 --retry 2 -o "$tmp" "$url"
chmod 755 "$tmp"
if ! "$tmp" --help >/dev/null 2>&1; then
    echo "oblong: the downloaded binary does not run here, likely a glibc version mismatch; kept the existing install" >&2
    exit 1
fi
mv -f "$tmp" "$bin"

cat > "$app_dir/oblong.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Oblong
Comment=Window management for Sway
Exec=$bin
Terminal=false
Categories=Settings;Utility;
EOF

echo "oblong installed to $bin"

case ":$PATH:" in
    *":$bin_dir:"*) ;;
    *) echo "note: $bin_dir is not on your PATH" ;;
esac
