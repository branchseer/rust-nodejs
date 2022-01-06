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

test_command = ["cargo", "test", "--target", config.target_triple, "-vv", "--release"]

if sys.platform == 'darwin' and config.arch == 'arm64':
    test_command += [ "--no-run" ]
else:
    test_command += [ "--", "--nocapture" ]

subprocess.check_call(test_command, cwd=crate_path)
