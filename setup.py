from setuptools import setup, find_packages

setup(
    name='rust-server',
    version='0.0.0',
    package_dir={'': 'gen/src/python'},
    install_requires=[
        'betterproto',
    ],
)