from setuptools import setup

setup(
    name='zsh-history-cleaner',
    version='0.0.1',
    packages=['zshhistorycleaner'],
    url='https://github.com/haidaraM/zsh-history-cleaner',
    license='MIT',
    author='Mohamed El Mouctar HAIDARA',
    author_email='elmhaidara@gmail.com',
    description='Clean your zsh history',
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
            'zhc = zshhistorycleaner:main'
        ]
    }
)
