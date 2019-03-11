#! /usr/bin/env nix-shell
#! nix-shell -i bash -p ronn cargo 

set -eux

target="${1:-all}"
dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

cd "$dir"

case "$target" in
  all)
    "$dir/do" doc
    # does not work currently, we are using rustup not nix.
    # "$dir/do" release
  ;;
  clean)
  	cargo clean
  	rm -f ./man/overseer.1
  	rm -f ./man/overseer.1.html
  ;;
  doc)
    ronn ./man/overseer.1.ronn
  ;;
  release)
    cargo build --release
  ;;
  *)
    echo "Don't know how to do '$target'"
    exit 1
  ;;
esac