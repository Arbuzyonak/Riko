#!/bin/sh
set -eu
if [ $# -lt 1 ]; then
    echo "usage: $0 <plugin-dir> [out-dir]" >&2
    echo "  zips a plugin folder and prints the catalog entry stub (name, size, sha256)" >&2
    exit 2
fi

plugin_dir=${1%/}
out_dir=${2:-.}
name=$(basename "$plugin_dir")

if [ ! -f "$plugin_dir/plugin.toml" ]; then
    echo "error: $plugin_dir has no plugin.toml" >&2
    exit 1
fi

mkdir -p "$out_dir"
archive="$out_dir/$name.zip"
rm -f "$archive"
( cd "$(dirname "$plugin_dir")" && zip -r -q -X "$OLDPWD/$archive" "$name" )

size=$(wc -c < "$archive")
sum=$(sha256sum "$archive" | cut -d' ' -f1)

echo "packaged $archive"
echo "--- catalog entry ---"
printf '{\n  "name": "%s",\n  "download_url": "REPLACE_ME",\n  "sha256": "%s",\n  "size_bytes": %s\n}\n' \
    "$name" "$sum" "$size"
