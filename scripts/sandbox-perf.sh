#!/bin/sh
set -eu

doc=${1:-README.md}
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

[ -f "$doc" ] || { echo "no such document: $doc" >&2; exit 1; }

perf=target/perf.md
: > "$perf"
for _ in $(seq 40); do
    cat "$doc" >> "$perf"
    printf '\n\n' >> "$perf"
done

scripts/sandbox.sh "$perf"

result=target/perf.txt

fail() {
    echo "perf acceptance failed: $1" >&2
    exit 1
}

grep -q '^wheel_frame_changed=1$' "$result" || fail 'wheel scroll did not change the frame'
grep -q '^drag_frame_changed=1$' "$result" || fail 'selection drag did not change the frame'

wheel_cpu=$(sed -n 's/^wheel_cpu_percent=//p' "$result")
drag_cpu=$(sed -n 's/^drag_cpu_percent=//p' "$result")

awk -v value="$wheel_cpu" 'BEGIN { exit !(value < 100) }' || fail "wheel scroll used ${wheel_cpu}% CPU"
awk -v value="$drag_cpu" 'BEGIN { exit !(value < 100) }' || fail "selection drag used ${drag_cpu}% CPU"

echo "perf acceptance passed"
