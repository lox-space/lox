import os
from pathlib import Path
from subprocess import run

import PyInstaller.__main__


fspath = getattr(os, 'fspath', str)

test_file = Path(__file__).parent.joinpath('tai-utc-example.py')


def test_start_and_stop(tmp_path):
    name = 'orekit_jpype_test_app'
    dist = tmp_path.joinpath('dist')
    work = tmp_path.joinpath('build')

    PyInstaller.__main__.run([
        '--name', name,
        '--distpath', fspath(dist),
        '--workpath', fspath(work),
        fspath(test_file)
    ])

    run([str(dist / name / name)], check=True)
