import sys
import os
import shutil
import subprocess
import glob

from . import config

assert __name__ == "__main__"

nodeSrcFolder = 'node-{}'.format(config.nodeVersion)
resultFolder = 'libnode'

libFolder = os.path.join(resultFolder, 'lib')

shutil.rmtree(resultFolder, ignore_errors=True)

os.mkdir(resultFolder)

header_path = os.path.realpath(
    os.path.join(
        os.path.dirname(__file__),
        "..", "patch", "node", "src", "node_embedding_api.h"
    )
)
subprocess.check_call([
    "bindgen", "--allowlist-function", "^node_.*", header_path,
    "--output", os.path.join(resultFolder, "sys.rs"),
    "--", "-target", config.target_triple
])

os.mkdir(libFolder)

def filterLibFile(filename):
    return 'gtest' not in filename and 'v8_nosnapshot' not in filename and 'v8_init' not in filename and 'icutools' not in filename

if sys.platform == 'win32':
    for libFile in os.scandir(nodeSrcFolder + '\\out\\Release\\lib'):
        if libFile.is_file() and libFile.name.endswith('.lib') and filterLibFile(libFile.name):
            print('Copying', libFile.name)
            shutil.copy(libFile.path, libFolder)
elif sys.platform == 'darwin':
    for libFile in os.scandir(nodeSrcFolder + '/out/Release'):
        if libFile.is_file() and libFile.name.endswith('.a') and filterLibFile(libFile.name):
            print('Copying', libFile.name)
            shutil.copy(libFile.path, libFolder)
            print('Striping', libFile.name)
            subprocess.check_call(['strip', '-x', os.path.join(libFolder, libFile.name)])
elif sys.platform == 'linux':
    for dirname, _, basenames in os.walk(nodeSrcFolder + '/out/Release/obj.target'):
        for basename in basenames:
            if basename.endswith('.a') and filterLibFile(basename):
                subprocess.run(
                    'ar -t {} | xargs ar rs {}'.format(
                        os.path.join(dirname, basename),
                        os.path.join(libFolder, basename)
                    ),
                    check=True, shell=True
                )

additional_obj_glob = nodeSrcFolder + '/out/Release/obj.target/node/gen/*.o'
if sys.platform == 'win32':
    additional_obj_glob = nodeSrcFolder + '/out/Release/obj/node_mksnapshot/src/*.obj'

if sys.platform == 'win32':
    subprocess.check_call([
            'lib', '/OUT:' + os.path.join(libFolder, "libnode_snapshot.lib")
        ] + 
        glob.glob(additional_obj_glob) + 
        glob.glob(nodeSrcFolder + '/out/Release/obj/node_mksnapshot/tools/msvs/pch/*.obj')
    )
else:
    subprocess.check_call([
        'ar', 'cr', 
        os.path.join(libFolder, "libnode_snapshot.a")
    ] + glob.glob(additional_obj_glob))
