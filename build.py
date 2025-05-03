#!/usr/bin/env python
from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name="cgn_ec_rust_parsers",
    version="0.1.1",
    rust_extensions=[RustExtension("cgn_ec_rust_parsers", binding=Binding.PyO3)],
    zip_safe=False,
)
