# Yet Another template engine

Inspired by %nix shell and Java version from some years ago and some code influenced by other Rust templating engines.

Features:
1. Default delimiters "${" and "}". Can be overridden to anything.
2. Default values with :- syntax.
3. Nesting of variables allowed (to 16 levels).
4. Can use environment variables.
5. Value lookup order: supplied HashMap, Environment, Default (if supplied).
6. Plays nicely with serde HashMaps.
7. Can include files (which can nest). With .inc extension only.
8. No dependencies.
9. Fast.
10. Can use multi-valued variables for lists etc. (only through HashMap).
11. Can check existence of a value and if true give it a default. Useful for 
    HTML forms when variable has a particular value, and this value should
    be the default selected value.
12. Provide a list of values, '#' separated, and instantiate multiple instances of variable (at same recursive depth), with successive values.
13. Literal expansion, do not recursively resolve contents. Useful for embedded
    code or example.


Normal variables with default delimiters would be: ${variable_name} and 
will be looked up in supplied HashMap. If not found in the HashMap then the 
environment will be queried. If the value is not found in either place, then 
a default value can be used, if supplied. A default value would
be indicated by ":-" or ":=". So ${variable_name:=default value} would result
in "default value" being added to the output.

The contents of the variable will normally be recursive checked for further
embedded variables, to a depth of 16 levels. Values will be trimed of leading and following whitespace by default.

```
The variable name may be preceded by a modifier. Modifiers are :
'=' - Do not recursively check the content of the variables for further
      expansion or trim the spaces.
'*' - Multiple values, separated by '|' delimiters are supplied in referenced
      embedded variables. See test case.
'!' - An external file (which must end with .inc) is supplied and will be
      included. Further recursive expansion is done as usual.
'?' - Condition, if variable has value then use default (usefule for drop
      down lists in HTML for example, to indicate selected item)
'#' - Simple Multiple values are supplied, again separated by '|' see test case.
```

Please see the `API documentation` https://docs.rs/stemplate/ and test cases.
