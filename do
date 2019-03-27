#! /usr/bin/env nix-shell
#! nix-shell -i bash -p ronn

set -eux

target="${1:-default}"
dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

cd "$dir"

case "$target" in
  default)
    echo "Try ./do test or ./do doc."
  ;;
  doc)
    mkdir -p ./man/generated/
    cp ./man/orderly.1.ronn ./man/generated/
    ronn ./man/generated/orderly.1.ronn
    rm ./man/generated/orderly.1.ronn
    MANWIDTH=100 man ./man/generated/orderly.1 | col -bx > ./man/generated/orderly.1.txt
  ;;
  test)
    cargo build
    export PATH="$PATH:$(pwd)/target/debug/"
    ./test/run_tests
  ;;
  *)
    echo "Don't know how to do '$target'"
    exit 1
  ;;
esac