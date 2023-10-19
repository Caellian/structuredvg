A scraper that reads the spec and generates json summaries and code based the
sources.

Intended to be used by this crate only, but feel free to copy parts you find
useful.

Code in this is very ugly and untested as it only intended to be ran once.
Maybe it will get cleaned up if I end up adding support for SVG 2 as it's a WIP
spec and would require multiple runs.

But uhm some sensible things to do would be:

- [ ] Add support for SVG 2 spec
  - HTML is very similarly structured to 1.1, only a bit better so it should be less work in theory
- [ ] Use `clap`
- [ ] Add incremental generation
  - use `syn` to figure out what exists already and what's changed