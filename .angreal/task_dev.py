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


