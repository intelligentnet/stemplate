# Yet Another Simple template engine

Inspired by *nix shell and Java version from some years ago and some code influenced by other Rust templating engines.

Features:
1. Default delimiters "${" and "}". Can be overridden to anything.
2. Default values with :- syntax.
3. Nesting of variables allowed (to 8 levels).
4. Can use environment variables.
5. Value lookup order: supplied HashMap, Environment, Default (if supplied).
6. Plays nicely with serde HashMaps.
7. Zero dependencies.
8. Fast.

Please read the `API documentation` https://docs.rs/stemplate/

