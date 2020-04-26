from setuptools import setup

from zshhistorycleaner import __version__, __prog__

with open('Readme.md') as f:
    long_description = f.read()

setup(
    name='zsh-history-cleaner',
    version=__version__,
    long_description=long_description,
    long_description_content_type='text/markdown',
    packages=['zshhistorycleaner'],
    url='https://github.com/haidaraM/zsh-history-cleaner',
    license='MIT',
    author='Mohamed El Mouctar HAIDARA',
    author_email='elmhaidara@gmail.com',
    description='Clean your zsh history',
    download_url=f'https://github.com/haidaraM/zsh-history-cleaner/archive/v{__version__}.tar.gz',
    classifiers=[
        'Development Status :: 5 - Production/Stable',
        'Intended Audience :: Developers',
        'Intended Audience :: System Administrators',
        'License :: OSI Approved :: MIT License',
        'Environment :: Console',
        'Topic :: Utilities',
        'Programming Language :: Python :: 3.6',
        'Programming Language :: Python :: 3.7',
    ],
    entry_points={
        'console_scripts': [
            f'{__prog__} = zshhistorycleaner.cli:main'
        ]
    }
)
