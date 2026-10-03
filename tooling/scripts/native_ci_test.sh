#!/usr/bin/env bash
# Run native tests without unrelated protected host processes in the Linux scan.
set -euo pipefail

cargo_bin=$(command -v cargo)
if [[ $(uname -s) == Linux ]]; then
  # Mount proc inside a new PID/mount namespace, then drop back to the runner's
  # uid/gid before Cargo or any test executes. Host processes remain untouched.
  exec sudo --preserve-env unshare --mount --pid --fork --kill-child \
    --propagation private bash -c '
      set -euo pipefail
      # Avoid stacked /proc mounts: the production guard rejects ambiguity.
      umount -l /proc
      mount -t proc -o nosuid,nodev,noexec proc /proc
      # sudo replaces PATH even with --preserve-env. Restore the toolchain path
      # only after dropping privileges, so Cargo can resolve rustc and rustdoc.
      exec setpriv --reuid "$1" --regid "$2" --clear-groups \
        env "PATH=$4" "$3" test "${@:5}"
    ' native-ci "$(id -u)" "$(id -g)" "$cargo_bin" "$PATH" "$@"
fi
exec "$cargo_bin" test "$@"
