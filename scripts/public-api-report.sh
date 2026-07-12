#!/usr/bin/env bash

set -euo pipefail

ROOT="$(CDPATH="" cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$ROOT/target/doc/public-api"

if ! cargo public-api --help >/dev/null 2>&1; then
    echo "Installing cargo-public-api..." >&2
    cargo install --locked cargo-public-api
fi

mkdir -p "$OUT_DIR"

metadata_file="$(mktemp)"
trap 'rm -f "$metadata_file"' EXIT

cargo metadata --no-deps --format-version 1 --manifest-path "$ROOT/Cargo.toml" > "$metadata_file"

python3 - "$ROOT" "$metadata_file" "$OUT_DIR" <<'PY'
import json
import pathlib
import subprocess
import sys

root = pathlib.Path(sys.argv[1])
metadata_path = pathlib.Path(sys.argv[2])
out_dir = pathlib.Path(sys.argv[3])

with metadata_path.open() as fh:
    metadata = json.load(fh)

workspace_members = set(metadata["workspace_members"])

packages = []
for pkg in metadata["packages"]:
    if pkg["id"] not in workspace_members:
        continue
    if any("lib" in target["kind"] or "proc-macro" in target["kind"] for target in pkg["targets"]):
        packages.append((pkg["name"], pkg["manifest_path"]))

for package, manifest_path in sorted(packages):
    out_file = out_dir / f"{package}.txt"
    try:
        result = subprocess.run(
            ["cargo", "public-api", "--manifest-path", manifest_path, "--all-features"],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
        )
    except subprocess.CalledProcessError as exc:
        warning = out_dir / f"{package}.error.txt"
        warning.write_text(exc.stderr or exc.stdout or str(exc))
        print(f"Skipped {package} ({warning.relative_to(root)})")
        continue
    out_file.write_text(result.stdout)
    print(f"Wrote {out_file.relative_to(root)}")
PY
