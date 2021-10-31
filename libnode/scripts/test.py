import os
import subprocess

from . import config

assert __name__ == "__main__"

workspace_path = os.path.realpath(os.path.join(
    os.path.dirname(__file__),
    "..", ".."
))

libnode_path = os.path.realpath(os.path.join(
    os.path.dirname(__file__),
    "..", "libnode"
))

os.environ["LIBNODE_PATH"] = libnode_path

subprocess.check_call(
    ["cargo", "test", "--release", "--workspace"],
    cwd = workspace_path,
)
