#!/bin/sh
set -eu

doc=$1
work=/work
runtime=/tmp/runtime
shots=/tmp/shots
output=HEADLESS-1
ticks_per_second=$(getconf CLK_TCK 2>/dev/null || printf '100')

mkdir -p "$runtime" "$shots" /tmp/.X11-unix
chmod 700 "$runtime"

export HOME=/tmp
export XDG_RUNTIME_DIR=$runtime
export XDG_PICTURES_DIR=$shots
export MDRS_LOG=1
export WLR_BACKENDS=headless
export WLR_RENDERER=gles2
export WLR_LIBINPUT_NO_DEVICES=1
export WLR_HEADLESS_OUTPUTS=1
export DBUS_SESSION_BUS_ADDRESS=unix:path="$runtime/bus"

sway --debug -c "$work/scripts/sandbox-sway.conf" >"$work/target/sandbox-sway.log" 2>&1 &
sway_pid=$!
trap 'kill $sway_pid 2>/dev/null || true' EXIT

socket_path=$(find "$runtime" -maxdepth 1 -type s -name 'wayland-*' | head -n 1)
while [ -z "$socket_path" ]; do
    kill -0 "$sway_pid" 2>/dev/null || { cat "$work/target/sandbox-sway.log"; exit 1; }
    sleep 0.1
    socket_path=$(find "$runtime" -maxdepth 1 -type s -name 'wayland-*' | head -n 1)
done
export WAYLAND_DISPLAY=$(basename "$socket_path")

while [ -z "$(ls /tmp/.X11-unix/X* 2>/dev/null)" ]; do
    kill -0 "$sway_pid" 2>/dev/null || { cat "$work/target/sandbox-sway.log"; exit 1; }
    sleep 0.1
done
x_socket=$(ls /tmp/.X11-unix/X* | head -n 1)
x_display=:$(basename "$x_socket" | tr -d X)
export DISPLAY=$x_display

env -u WAYLAND_DISPLAY "$work/target/release/mdrs" "$doc" \
    >"$work/target/sandbox-app.log" 2>&1 &
app_pid=$!
trap 'kill $app_pid 2>/dev/null || true; kill $sway_pid 2>/dev/null || true' EXIT

while ! grep -q 'selected wgpu adapter' "$work/target/sandbox-app.log"; do
    kill -0 "$app_pid" 2>/dev/null || { cat "$work/target/sandbox-app.log"; exit 1; }
    sleep 0.1
done

if grep -qiE 'llvmpipe|lavapipe|swiftshader|softpipe' \
    "$work/target/sandbox-app.log" "$work/target/sandbox-sway.log"; then
    cat "$work/target/sandbox-app.log"
    exit 1
fi

sleep 1
grim -o "$output" "$work/target/screenshot.png"

ticks() {
    awk '{print $14 + $15}' "/proc/$app_pid/stat"
}

now_ns() {
    date +%s%N
}

percent() {
    awk -v ticks="$1" -v ns="$2" -v hz="$ticks_per_second" \
        'BEGIN { printf "%.1f", ticks / hz / (ns / 1000000000) * 100 }'
}

seconds() {
    awk -v ns="$1" 'BEGIN { printf "%.2f", ns / 1000000000 }'
}

changed() {
    if cmp -s "$1" "$2"; then printf '0'; else printf '1'; fi
}

record() {
    label=$1
    action=$2
    start_ticks=$(ticks)
    start_ns=$(now_ns)
    "$action"
    end_ticks=$(ticks)
    end_ns=$(now_ns)
    printf '%s_cpu_percent=%s\n' "$label" "$(percent $((end_ticks - start_ticks)) $((end_ns - start_ns)))"
    printf '%s_seconds=%s\n' "$label" "$(seconds $((end_ns - start_ns)))"
}

wheel_scroll() {
    xdotool mousemove --sync 640 400
    xdotool click --repeat 200 --delay 20 5
}

drag_selection() {
    pass=0
    while [ "$pass" -lt 4 ]; do
        xdotool mousemove --sync 10 200
        xdotool mousedown 1
        y=200
        while [ "$y" -lt 760 ]; do
            y=$((y + 20))
            xdotool mousemove --sync 10 "$y"
            sleep 0.02
        done
        xdotool mouseup 1
        pass=$((pass + 1))
    done
}

result=$work/target/perf.txt
adapter=$(sed -n 's/^\[INFO\] selected wgpu adapter: //p' "$work/target/sandbox-app.log")
printf 'adapter=%s\n' "$adapter" > "$result"

grim -o "$output" "$shots/wheel-before.png"
record wheel wheel_scroll >> "$result"
grim -o "$output" "$shots/wheel-after.png"

grim -o "$output" "$shots/drag-before.png"
record drag drag_selection >> "$result"
grim -o "$output" "$shots/drag-after.png"

printf 'wheel_frame_changed=%s\n' "$(changed "$shots/wheel-before.png" "$shots/wheel-after.png")" >> "$result"
printf 'drag_frame_changed=%s\n' "$(changed "$shots/drag-before.png" "$shots/drag-after.png")" >> "$result"

cp "$shots/wheel-after.png" "$work/target/screenshot-scroll.png"
cp "$shots/drag-after.png" "$work/target/screenshot-selection.png"
cat "$result"
