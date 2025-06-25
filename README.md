# measure-install_code

This repository provides a small CLI tool to measure the cycles consumed by
`install_code` for Wasm modules of different sizes. It then fits a linear
approximation of the form:

```
cycles ~= slope * bytes + intercept
```

The tool assumes a local `dfx` environment is available. Each invocation of the
CLI will reinstall code on a given canister and read the cycle cost from the
JSON output produced by `dfx`.

## Usage

```
python measure_cycles.py <canister-name> path/to/first.wasm path/to/second.wasm ...
```

Example:

```
python measure_cycles.py my_canister build/a.wasm build/b.wasm
```

The script prints the size and cycles for each file and finally the linear
approximation parameters.

## Notes on ICP semantics

When analysing code running on the Internet Computer keep in mind that any code
after an `await` is not guaranteed to start. Once started, however, it will run
to completion. This tool performs synchronous installs and does not rely on any
post-`await` code execution semantics.
