#!/bin/sh
log_dir="${RIKO_DATA_DIR:-$HOME/.local/share/riko}"
mkdir -p "$log_dir"
log="$log_dir/ping.log"
while pgrep -f 'Vortex\.exe' >/dev/null 2>&1; do
    ms=$(ping -c 1 -W 2 playvortex.io 2>/dev/null | sed -n 's/.*time=\([0-9.]*\).*/\1/p')
    printf '%s %s\n' "$(date -Is)" "${ms:-timeout}" >> "$log"
    sleep 5
done
