#! /usr/bin/env nix-shell
#! nix-shell --pure -i bash -p man utillinux git cargo pandoc ronn

set -eux

target="${1:-default}"
dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

cd "$dir"

format_doc () {
  pandoc -f gfm -t gfm $1 > $1.tmp
  mv $1.tmp $1
}

case "$target" in
  default)
    echo "Read this script to get a list of valid commands."
  ;;
  doc)
    # format docs
    format_doc README.md
    format_doc ./example/README.md
    format_doc ./dist/README.md
    
    # generate man pages.
    rm -rf ./man/generated
    mkdir -p ./man/generated
    cp man/orderly.1.md ./man/generated/
    cd ./man/generated
    ronn orderly.1.md
    rm orderly.1.md
    MANWIDTH=100 man -l ./orderly.1 | col -bx > ./orderly.1.txt
  ;;
  test)
    cargo build
    export PATH="$PATH:$(pwd)/target/debug/"
    ./test/run_tests
  ;;
  git-push)
    git push origin
    git push origin --tags
    git push github
    git push github --tags
  ;;
  *)
    echo "Don't know how to do '$target'"
    exit 1
  ;;
esac