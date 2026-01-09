import os
import shutil
import sys


def _find_repo_root(start: str) -> str | None:
    current = os.path.abspath(start)
    while True:
        cargo_toml = os.path.join(current, "Cargo.toml")
        cli_crate = os.path.join(current, "crates", "cli", "Cargo.toml")
        if os.path.isfile(cargo_toml) and os.path.isfile(cli_crate):
            return current
        parent = os.path.dirname(current)
        if parent == current:
            return None
        current = parent


def main() -> None:
    args = sys.argv[1:]
    binary = shutil.which("arqonhpo-cli")
    if binary:
        os.execv(binary, [binary, *args])

    cargo = shutil.which("cargo")
    repo_root = _find_repo_root(os.getcwd())
    if cargo and repo_root:
        os.chdir(repo_root)
        os.execv(cargo, [cargo, "run", "-p", "arqonhpo-cli", "--", *args])

    sys.stderr.write(
        "arqonhpo CLI not found. Install with 'cargo install --path crates/cli --bin arqonhpo-cli'\n"
    )
    sys.exit(1)


if __name__ == "__main__":
    main()
