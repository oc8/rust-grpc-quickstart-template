from setuptools import setup, find_packages

setup(
    name='rust-server',
    version='0.0.0',
    package_dir={'': 'libs/gen/src/python'},
    install_requires=[
        'betterproto',
    ],
)