# Orderly

**orderly** is a tool that provides ordered and controlled start, stop,
restart and cleanup of a group of processes. It aims to be a building
block for reliable servers/services/containers/dev-environments.

orderly draws inspiration from erlang supervisor trees, It provides
mechanisms to build a tree of supervised processes, and failure can
propagrate when process restarts rate exceeds a specified limit.

orderly does actions via external hooks written in any programming
language you prefer. orderly does not make assumptions about your
application or the setup/teardown that must be done to run it correctly
and reliably.

If this interests you, you can read the manual [here](man/orderly.1.md).

## Rationale

**orderly** was originally made to support reliable recovery of
inter-dependant services after failure on a runit based linux system. In
this configuration orderly runs beneath runit, providing grouping, left
to right start and right to left cleanup that runit lacks. This allows
for things like crashed fuse file systems to be cleanly unmounted and
recreated, where the unordered restarts of runit cause problems.

**orderly** also addresses some quality of life problems when developing
a group of servers. Generally when developing 'microservice' style
projects you are stopping and starting many processes that depend on
eachother. **orderly** makes this work easier, as a single terminal
window + ctrl+c is all that is needed to reliably kill/restart all your
services.

More complicated init systems like systemd support some of this
functionality, but not for these use cases, and only for systems that go
'all-in' with systemd. The project author is a fan of the openbsd
operating system for example, which does not even support systemd.

## Example

``` 
$ orderly -max-start-tokens 2 -start-tokens-per-second 0.1 -- \
  -name redis   -run ./run-redis -wait-started ./wait-redis  -- \
  -name website -run ./run-website -check ./health-check-website -cleanup ./website-cleanup 
```

For a full featured example with code, check the example directory.

## Implementation status

The software works, but the interface is not stable and still being
tweaked. Expect breaking changes between releases until version 1.0.0 is
released.

## Building from source

[![builds.sr.ht
status](https://builds.sr.ht/~ach/orderly.svg)](https://builds.sr.ht/~ach/orderly?)

Orderly is a rust project, so running `cargo build` is sufficient for
most people after cloning the git repository.

## Installation

Binary packages are not yet provided.

The program can be installed from cargo with `cargo install orderly`

## Contact

Try the [mailing list](https://lists.sr.ht/~ach/orderly-dev)

Or create a [github
issue](https://github.com/andrewchambers/orderly/issues)

## Sponsors

This project was sponsored by [backupbox.io](https://backupbox.io)

## Authors

[Andrew Chambers](https://acha.ninja)
