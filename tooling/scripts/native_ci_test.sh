#!/usr/bin/env bash
# Run native tests without unrelated protected host processes in the Linux scan.
set -euo pipefail

if [[ ${1:-} == --test-binary ]]; then
  shift
  # Only the application library shares a process table across its test cases.
  # Keep integration tests and independent crates at their normal parallelism.
  case "$1" in
    */qiongli-[0-9a-f]*) export RUST_TEST_THREADS=1 ;;
  esac
  exec "$@"
fi

cargo_bin=$(command -v cargo)
if [[ $(uname -s) == Linux ]]; then
  host_triple=$(rustc -vV | awk '/^host:/ {print $2}')
  runner_array=$(python3 -c 'import json,sys; print(json.dumps(["bash", sys.argv[1], "--test-binary"]))' \
    "$(realpath "${BASH_SOURCE[0]}")")
  runner_config="target.$host_triple.runner=$runner_array"
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
        env "PATH=$4" "$3" --config "$5" test --no-fail-fast "${@:6}"
    ' native-ci "$(id -u)" "$(id -g)" "$cargo_bin" "$PATH" \
      "$runner_config" "$@"
fi
exec "$cargo_bin" test --no-fail-fast "$@"
