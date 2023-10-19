# StructuredVG

A work in progress library/experiment that aims to provide structured, type-safe
access for SVG file format.

- Tags are represented as Rust structs.
- Attributes are represented as fields on tag structs.
  - Attributes can be grouped into structs and applied to several tags uniformly.
- Children values are represented as enums.

To achieve this, it attacks the problem of covering such a large specification
from several angles:

- Uses traits to **generalize attribute handling**, grouping, reading and writing.
- Uses a [specification scraper](./spec-scraper) to simplify code generation.
  - Has to be ran manually and it allows incremental source modification (NYI).
- Uses proc-macros to generate repetitive implementations such as serializing
  each attribute and concatenating them.
  - Macro could also download and scrape the spec. but that could cause
    caching issues (see [RFC: proc macro include!](https://github.com/rust-lang/rfcs/pull/3200)).

## Goals & Scope

This library is meant to be a strongly typed, structured representation of SVG
format that supports deserialization and serialization while

That means that all elements should define all attributes and children as
denoted in the specification, and all attributes should be typed as strictly as
sensible (e.g. not checking IRI validity by default).

Non standard attributes and tags are still supported, but separated from the
recognized ones.

### Non-goals

This library will not include features that aren't fully described within the
SVG specification such as CSS style resolution, rendering, bounding box
computation, conversion of relative paths to absolute by applying transforms...

For simplification of parsed SVGs see [`usvg`](https://github.com/RazrFalcon/resvg/tree/master/crates/usvg).

## Remaining work

This is a high level overview of the work that still has to be done for this
library to be usable:

- [x] Leverage traits to make attributes easily composable through struct
      members.
- [x] Create a proc-macro that abstracts away composition details.
- [ ] Write a parser that generates most of the structs, almost correctly with
      minimal required intervention.
  - [x] Scrape SVG 1.1 spec
  - [x] Cache downloaded spec and intermediate results
  - [ ] Generate structs from intermediate results
    - [ ] Do so additively
    - [ ] Try using language transformers to provide a sparse but sufficient
          element and attribute documentation (with links to the docs).
  - [ ] Load intermediate results to "upgrade them"
    - [ ] Allow manual intervention for elements/attributes with information that's too complex to deduce from the specification.
- [ ] Implement any attribute values that have a better representation than `Cow<'_, str>`
  - [ ] Generate value enums from specification where value can be one of
        fixed number of strings.
  - [ ] Implement CSS units
- [ ] Add support for children
  - [ ] Add `Tag` trait that provides high level information about SVG elements.
    - [ ] Add `Tag::children` iterator.
- [ ] Handle [`<use>`](https://www.w3.org/TR/SVG11/single-page.html#struct-UseElement) resolving.
- [ ] Add `AttributeBundle::get` and `AttributeBundle::set` that play nicely with both standard and non-standard attributes.
- [ ] Improve and cleanup documentation as well as public API.
- [ ] Feature gate [interactivity](https://www.w3.org/TR/SVG11/single-page.html#chapter-interact),
      [linking](https://www.w3.org/TR/SVG11/single-page.html#chapter-linking),
      [scripting](https://www.w3.org/TR/SVG11/single-page.html#chapter-script) and
      [animation](https://www.w3.org/TR/SVG11/single-page.html#chapter-animate)
      (chapters 16-19 of the specification), because in a lot of cases SVG is used
      for static, non-interactive graphics and not integrated into websites.
- [ ] Consider some form of dynamic property inheritance and propagation? (e.g. 'computed value')
- [ ] Support rest of the ecosystem
  - [ ] Pass node tree to `usvg`
  - [ ] `web-sys` integration and DOM?
  - [ ] `serde` support?
- [ ] Support for SVG 2 specification
  - [ ] Provide a feature flag to control provided versions?

Most important building blocks are in place, but there's still a huge amount
that needs to be done for this library to be a viable alternative to partial
implementations.

## Alternatives

- [`usvg`](https://github.com/RazrFalcon/resvg/tree/master/crates/usvg)
  - Only overlap with this library is parsing and strong typing.
  - `usvg` is a better choice if you intend to render an SVG file as it does
    most of the heavy lifting for you.
  - Because it simplifies parsed files it's not ideal for applying minimal
  - Doesn't support generation of SVG files.

## License

This project is licensed under [Zlib](./LICENSE_ZLIB), [MIT](./LICENSE_MIT), or
[Apache-2.0](./LICENSE_APACHE) license, choose whichever suits you most.
