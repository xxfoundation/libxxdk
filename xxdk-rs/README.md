# Rust bindings for XXDK

## Organization

This directory is a Cargo workspace organized into the following crates:

- xxdk-sys: auto-generated unsafe bindings to the library emitted by CGo.
- xxdk (planned): safe wrappers and useful abstractions built on top of xxdk-sys.

## Prerequisites

In addition to Go, the build script for the xxdk-sys crate will need libclang in order to
generate the raw bindings.

## Building

The Rust bindings currently use Cargo as a build system instead of Make. From this
directory,

```
$ cargo build
```

should do the trick. Maybe do a quick

```
$ cargo test
```

to run the test suite if you like.

In the near future there will be an executable demo using these bindings.
