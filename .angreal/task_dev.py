"""Clotho development tasks."""
import angreal
import subprocess
import sys


@angreal.command(name="test", about="Run all tests")
def test():
    """Run cargo test for the full workspace."""
    subprocess.run(
        ["cargo", "test", "--workspace"],
        check=True,
    )


@angreal.command(name="build", about="Build all crates")
def build():
    """Build the full workspace."""
    subprocess.run(
        ["cargo", "build", "--workspace"],
        check=True,
    )


@angreal.command(name="build-release", about="Build release binaries")
def build_release():
    """Build release binaries for clotho and clotho-mcp."""
    subprocess.run(
        ["cargo", "build", "--release", "--workspace"],
        check=True,
    )


@angreal.command(name="check", about="Run clippy, fmt check, and cargo check")
def check():
    """Run all lints and checks."""
    print("==> clippy")
    subprocess.run(
        ["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
        check=True,
    )
    print("==> fmt")
    subprocess.run(
        ["cargo", "fmt", "--all", "--", "--check"],
        check=True,
    )
    print("==> check")
    subprocess.run(
        ["cargo", "check", "--workspace"],
        check=True,
    )


@angreal.command(name="clean", about="Clean build artifacts")
def clean():
    """Run cargo clean."""
    subprocess.run(["cargo", "clean"], check=True)


@angreal.command(name="install-local", about="Install clotho and clotho-mcp to ~/.local/bin")
def install_local():
    """Build release and copy binaries to ~/.local/bin."""
    import os
    import shutil

    build_release()

    install_dir = os.path.expanduser("~/.local/bin")
    os.makedirs(install_dir, exist_ok=True)

    for binary in ["clotho", "clotho-mcp"]:
        src = f"target/release/{binary}"
        dst = os.path.join(install_dir, binary)
        shutil.copy2(src, dst)
        os.chmod(dst, 0o755)
        print(f"Installed {dst}")
