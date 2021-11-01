import os
import subprocess

from . import config

assert __name__ == "__main__"

crate_path = os.path.realpath(os.path.join(
    os.path.dirname(__file__),
    "..", ".."
))

libnode_path = os.path.realpath(os.path.join(
    os.path.dirname(__file__),
    "..", "libnode"
))

os.environ["LIBNODE_PATH"] = libnode_path

subprocess.check_call(
    ["cargo", "test", "--target", config.target_triple, "-vvvv", "--release"],
    cwd=crate_path,
)
