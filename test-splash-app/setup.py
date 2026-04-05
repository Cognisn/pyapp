from setuptools import setup

setup(
    name="test-splash",
    version="1.0.0",
    py_modules=["test_splash"],
    entry_points={
        "console_scripts": [
            "test-splash=test_splash:main",
        ],
    },
)
