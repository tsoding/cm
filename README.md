[![Build Status](https://github.com/tsoding/cm/workflows/CI/badge.svg)](https://github.com/tsoding/cm/actions)

# cm

**WARNING! The application is in an active development state and is not even alpha yet. Use it at your own risk. Nothing is documented, anything can be changed at any moment or stop working at all.**

The goal of this application is to recreate the [compilation-mode] of Emacs as a standalone TUI application.

## Build

### Dependencies

- [rust](https://www.rust-lang.org/)
- [ncurses](https://invisible-island.net/ncurses/)
- [pcre2](https://www.pcre.org/)

### Workflow

```console
$ cargo build
$ ./target/debug/cm 'grep -rn String src/'
```

## Usage

### Default Shortcuts

|Shortcut|Description|
|---|---|
|<kbd>j</kbd> or <kbd>Down</kbd>|Move cursor down|
|<kbd>k</kbd> or <kbd>up</kbd>|Move cursor up|
|<kbd>Alt</kbd>+<kbd>j</kbd> or <kbd>Alt</kbd>+<kbd>Down</kbd>|Move cursor to the next match|
|<kbd>Alt</kbd>+<kbd>k</kbd> or <kbd>Alt</kbd>+<kbd>Up</kbd>|Move cursor to the prev match|

<!-- TODO(#87): Document shortcuts -->
Rest default shortcuts are [TBD](https://github.com/tsoding/cm/issues/87)

### Application layout

<!-- TODO(#86): Document application layout -->
[TBD](https://github.com/tsoding/cm/issues/86)

### Configuration file cm.conf

<!-- TODO(#45): Document config format -->
[TBD](https://github.com/tsoding/cm/issues/45)

### Self-recurring

<!-- TODO(#125): Document self-recurring -->

## Examples

## Searching with grep

[![asciicast](https://asciinema.org/a/327091.svg)](https://asciinema.org/a/327091)

## Fixing compilation errors

[![asciicast](https://asciinema.org/a/337846.svg)](https://asciinema.org/a/337846)

[compilation-mode]: https://www.gnu.org/software/emacs/manual/html_node/emacs/Compilation-Mode.html
