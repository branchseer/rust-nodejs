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
    configureArgvs += ["--dest-cpu=" + config.arch]
    subprocess.check_call([sys.executable, 'configure.py'] + configureArgvs)
    subprocess.check_call(['make', '-j4'])
