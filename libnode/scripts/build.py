import sys
import os
import subprocess
from . import config

assert __name__ == "__main__"


os.chdir('node-{}'.format(config.nodeVersion))

configureArgvs = ['--enable-static'] + config.configFlags

if sys.platform == 'win32':
    os.environ["config_flags"] = ' '.join(configureArgvs)
    subprocess.check_call(
        ['cmd', '/c', 'vcbuild.bat', config.arch],
    )
else:
    if sys.platform == 'darwin' and config.arch != 'x64':
        os.environ.update({
            "GYP_CROSSCOMPILE": '1',
            "CC": "cc -arch " + config.arch,
            "CXX": "c++ -arch " + config.arch,
            "CC_target": "cc -arch " + config.arch,
            "CXX_target": "c++ -arch " + config.arch,
            "CC_host": "cc -arch x86_64",
            "CXX_host": "c++ -arch x86_64"
        })
    configureArgvs += ["--dest-cpu=" + config.arch]
    subprocess.check_call([sys.executable, 'configure.py'] + configureArgvs)
    subprocess.check_call(['make', '-j4'])
