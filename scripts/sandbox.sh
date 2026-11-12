#!/bin/sh
set -eu

usage() {
    echo "usage: scripts/sandbox.sh <file.md>" >&2
    exit 2
}

[ $# -eq 1 ] || usage
doc=$1
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

if [ -z "${MDRS_SANDBOX_ENV:-}" ]; then
    exec direnv exec "$root" env MDRS_SANDBOX_ENV=1 "$0" "$doc"
fi

case $doc in
    /*) ;;
    *) doc=$root/$doc ;;
esac
[ -f "$doc" ] || { echo "no such document: $doc" >&2; exit 1; }

case $doc in
    "$root"/*) ;;
    *) echo "document must be inside $root" >&2; exit 1 ;;
esac

ls /dev/dri/renderD* >/dev/null 2>&1 || {
    echo "no hardware GPU: /dev/dri/renderD* is missing" >&2
    exit 1
}

chrt -i 0 nice -n 19 ionice -c 3 cargo build --release

icd=$(ls /run/opengl-driver/share/vulkan/icd.d/*.json |
    grep -Ev 'lvp|dzn|gfxstream' |
    paste -sd:)

podman run --rm \
    --userns=keep-id \
    --device /dev/dri \
    --group-add keep-groups \
    --network=none \
    --cpus=10 \
    --memory=8g \
    --pids-limit=2048 \
    --blkio-weight=10 \
    --cap-drop=all \
    --security-opt=no-new-privileges \
    -v "$root":/work \
    -v /nix/store:/nix/store:ro \
    -v /run/opengl-driver:/run/opengl-driver:ro \
    -e "PATH=$PATH" \
    -e "LD_LIBRARY_PATH=${LD_LIBRARY_PATH:-}" \
    -e "VK_ICD_FILENAMES=$icd" \
    -e HOME=/tmp \
    docker.io/library/ubuntu:latest \
    /bin/sh /work/scripts/sandbox-session.sh "/work/${doc#"$root"/}"

echo "screenshot: $root/target/screenshot.png"
