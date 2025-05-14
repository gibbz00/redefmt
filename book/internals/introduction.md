# Internals

## Terminology

There are to kind of *register* functions, write statemtens and print
statements. The former is invoked when implementing the `Format` trait,
whilst the latter is used for `println!()` and `log!()` like statement. Both
register format statements, but only print statements register a log level
and the call site location. Write statements may alternatively register type
structures (field names etc.), letting a printer itself choose how types should
be displayed.
