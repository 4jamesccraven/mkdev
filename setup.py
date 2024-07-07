'''
This is included specifically to support packaging in Nix with
pkgs.python3Packages.buildPythonPackage.
'''

from setuptools import setup, find_packages

setup(
    name='mkdev',
    version='2.0',
    packages=find_packages(where='src'),
    package_dir={'': 'src'},
)
