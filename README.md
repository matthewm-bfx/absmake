# Introduction

This is a `make` wrapper which aims to convert compiler diagnostics from relative to absolute
paths. This may be useful when using a compiler like GCC/G++, which lacks the
`-fdiagnostics-absolute-paths` option exposed by Clang, with an IDE like VSCode which
interprets all diagnostic paths relative to a fixed base path.

# Building

This is a Rust project which is built using the normal `cargo` command, e.g.

`$ cargo build --release`

This will produce an executable binary in the `target/release` subdirectory, which can then be
moved to a directory of your choice.

# Usage

`$ absmake <make options>`

`absmake` works as a wrapper for `make` and does not expose any options of its own.

# Limitations

`absmake` keeps track of the current working directory by interpreting messages from `make`
like `Entering directory '/dir'`, which are emitted in response to the `-w` option (and
possibly by default). Since there is only a single working directory stored, this may not give
correct results in a multi-job `make` run where different directories are compiled
simultaneously.
