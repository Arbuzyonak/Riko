#!/bin/sh
command -v gpu-screen-recorder >/dev/null 2>&1 || exit 0
pgrep -f 'Vortex\.exe' >/dev/null 2>&1 || exit 0
out_dir="${XDG_VIDEOS_DIR:-$HOME/Videos}/riko-replays"
mkdir -p "$out_dir"
gpu-screen-recorder -w screen -f 60 -c mp4 -r 60 -o "$out_dir" >/dev/null 2>&1 &
recorder=$!
while pgrep -f 'Vortex\.exe' >/dev/null 2>&1; do
    kill -0 "$recorder" 2>/dev/null || exit 0
    sleep 5
done
kill -INT "$recorder" 2>/dev/null
