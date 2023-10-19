# Implementation details

Token parsing and generation doesn't have to exactly follow the rules provided by the spec. because in some cases there's a cheaper way to achieve the same results.

For instance, writing a style attribute declaration list inserts a mandatory
semicolon for first n-1 declarations which avoids a branch that would be caused
by stricly adhering to `: S* declaration? [ ';' S* declaration? ]*`.

TODO: this is just an inverse of that if, so... adhere to the standard instead

# Attribute bundles

While the specification groups some attributes together, there's no concept of "attribute bundle" in the spec.
In the implementation, we're using a concept of "attribute bundle" to group attributes together for convenience.
As such, some attributes are bundled together even if they aren't in the specification or only in SVG 2. This is intended to
make the library more convenient and structures more normalized (decrease in redundancy).

That isn't a purely stylistic choice as it causes pointer dereferencing and higher likelihood of cache misses which negatively impacts performance.

# Data storage

Storing attributes and children directly in memory would make this library
extremely memory inefficient. Initial testing did so and `<path>` ended up
consuming around 0.5KB of memory without pointers, and ~64B when bundles were
boxed.

Instead, the library stores the attributes in `HashMap`s and children in `Vec`,
while providing a type-safe API for storing and reading those entries. That
provides additional type safety while still enforcing the specification.