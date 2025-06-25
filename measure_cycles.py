import argparse
import subprocess
import json
import os
from statistics import mean


def parse_dfx_output(output: str) -> int:
    """Extract the cycle cost from dfx output.

    This function expects the install_code output to contain a JSON line
    with a field `cycles`. If not found, it raises a ValueError.
    """
    try:
        data = json.loads(output)
        return int(data.get("cycles"))
    except Exception as e:
        raise ValueError("Could not parse cycles from dfx output") from e


def measure_install_code_cycles(wasm_path: str, canister_name: str) -> int:
    """Run `dfx canister install` and return the cycles used."""
    cmd = [
        "dfx",
        "canister",
        "install",
        canister_name,
        "--wasm",
        wasm_path,
        "--mode",
        "reinstall",
        "--with-cycles",
    ]
    try:
        result = subprocess.run(
            cmd, check=True, capture_output=True, text=True
        )
    except subprocess.CalledProcessError as e:
        raise RuntimeError(
            f"dfx failed with code {e.returncode}: {e.stderr.strip()}"
        )

    return parse_dfx_output(result.stdout)


def linear_regression(xs, ys):
    """Return slope and intercept for ys = a*xs + b."""
    if len(xs) != len(ys):
        raise ValueError("Mismatched data")
    n = len(xs)
    if n == 0:
        raise ValueError("No data")

    avg_x = mean(xs)
    avg_y = mean(ys)
    cov = sum((x - avg_x) * (y - avg_y) for x, y in zip(xs, ys))
    var = sum((x - avg_x) ** 2 for x in xs)
    if var == 0:
        raise ValueError("Variance is zero")
    slope = cov / var
    intercept = avg_y - slope * avg_x
    return slope, intercept


def main():
    parser = argparse.ArgumentParser(
        description="Measure cycles spent by install_code for given Wasm files"
    )
    parser.add_argument("canister", help="Target canister name")
    parser.add_argument(
        "wasm",
        nargs="+",
        help="Paths to Wasm files to install",
    )

    args = parser.parse_args()

    sizes = []
    cycles = []

    for wasm_path in args.wasm:
        size = os.path.getsize(wasm_path)
        cost = measure_install_code_cycles(wasm_path, args.canister)
        sizes.append(size)
        cycles.append(cost)
        print(f"{wasm_path}: size={size} bytes cycles={cost}")

    slope, intercept = linear_regression(sizes, cycles)
    print("Linear approximation: cycles ~= {:.2f} * bytes + {:.2f}".format(slope, intercept))


if __name__ == "__main__":
    main()
