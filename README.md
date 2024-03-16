# Yet Another Simple template engine

Inspired by *nix shell and Java version from some years ago and some code influenced by other Rust templating engines.

Features:
1. Default delimiters "${" and "}". Can be overridden to anything.
2. Default values with :- syntax.
3. Nesting of variables allowed (to 8 levels).
4. Can use environment variables.
5. Value lookup order: supplied HashMap, Environment, Default (if supplied).
6. Plays nicely with serde HashMaps.
7. Can include files (which can nest). With .inc extension only.
8. Zero dependencies.
9. Fast.
10. Can use multi-valued variables for lists etc. (only through HashMap)

Please read the `API documentation` https://docs.rs/stemplate/

