# measure-install_code

This repository provides a Rust CLI tool to measure the cycles consumed by
`install_code` for Wasm modules of different sizes. It then fits a linear
approximation of the form:

```
cycles ~= slope * bytes + intercept
```

The CLI uses the management canister directly and requires access to an
Internet Computer replica (for example a local `dfx` instance).

## Usage

```
cargo run -- --canister-id <CANISTER_ID> path/to/a.wasm path/to/b.wasm
```

Example:

```
cargo run -- --canister-id rrkah-fqaaa-aaaaa-aaaaq-cai build/a.wasm build/b.wasm
```

The program prints the size of each file and the cycles required. Finally it
outputs the parameters of the linear approximation.

## Notes on ICP semantics

When analysing code running on the Internet Computer, keep in mind that any
code after an `await` is not guaranteed to start. Once started, however, it will
run to completion. This tool performs synchronous installs and does not rely on
any post-`await` code execution semantics.
