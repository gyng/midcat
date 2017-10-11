# midcat

Example application for using [synthrs](https://github.com/gyng/synthrs) with [cpal](https://github.com/tomaka/cpal).

Plays a MIDI file (ignoring instruments) from the command line.

## Usage

```
$ cargo +nightly run -- foobar.mid
$ cargo +nightly run -- --help

Play a MIDI file, ignoring instruments

Usage:
  midcat <file> [--volume=<frac>] [--speed=<times>]
  midcat (-h | --help)
  midcat --version

Options:
  -h --help                   Show this screen
  -v=<frac> --volume=<frac>   Play volume as a fraction (linear scale) [default: 1.0]
  -s=<times> --speed=<times>  Play speed as a fraction [default: 1.0]
```
