#!/usr/bin/env python3

# *******************************************************************************
# Copyright (c) 2025 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache License Version 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0
#
# SPDX-License-Identifier: Apache-2.0
# *******************************************************************************
"""
This script is designed to manage and execute Python-based integration tests
within a virtual environment. It ensures that the virtual environment is set up
correctly, dependencies are installed, and the tests are executed in a controlled
environment.


Usage:
Run this script from the test_cases directory.
Script adds implicitly below pytest arguments:
* -vv - to increase verbosity of pytest output
* --build-scenarios - to build tests scenarios if not built yet
* --traces=all - to collect test and components under test traces

Example:
     ./run_tests.py
     ./run_tests.py tests/runtime/worker/test_worker_basic.py
     ./run_tests.py [additional pytest arguments]
"""

import hashlib
import subprocess
import sys
from pathlib import Path


class VEnv:
    def __init__(self, venv_path: Path):
        self._venv_path = venv_path
        self._python = venv_path / "bin" / "python3"
        self._pip = venv_path / "bin" / "pip3"

        if not self._python.is_file() or not self._pip.is_file():
            print(
                "Virtual environment does not exist, creating...",
                file=sys.stderr,
            )
            self._create()

    @property
    def python_path(self) -> Path:
        return self._python

    def _create(self):
        try:
            subprocess.run(
                [sys.executable, "-m", "venv", self._venv_path.resolve()],
                check=True,
            )
        except subprocess.CalledProcessError as e:
            print(f"Failed to create virtual environment: {e}", file=sys.stderr)
            sys.exit(1)

    def install_dependencies(self, requirements_file: Path):
        try:
            subprocess.run(
                [
                    str(self._pip),
                    "install",
                    "--force-reinstall",
                    "-r",
                    str(requirements_file),
                ],
                check=True,
            )
        except subprocess.CalledProcessError as e:
            print(f"Failed to install dependencies: {e}", file=sys.stderr)
            sys.exit(1)


class FileHash:
    def __init__(self, file_path: Path):
        self._hash = self._compute_hash(file_path)

    def _compute_hash(self, file_path: Path) -> str:
        hash = hashlib.sha256()
        hash.update(file_path.read_bytes())
        return hash.hexdigest()

    def save_to_file(self, hash_file: Path):
        hash_file.write_text(f"{self._hash}\n")

    def is_the_same_as(self, hash_file: Path) -> bool:
        if not hash_file.exists():
            return False
        existing_hash = hash_file.read_text().strip()
        return self._hash == existing_hash


def check_cwd():
    cwd = Path.cwd()
    expected_cwd = Path(__file__).parent.resolve()

    if cwd != expected_cwd:
        print(
            f"Error: Script must be run from {expected_cwd}, but current directory is {cwd}",
            file=sys.stderr,
        )
        sys.exit(1)


def main():
    check_cwd()

    venv_path = Path(".venv")
    venv = VEnv(venv_path)

    req_file = Path("requirements.txt")
    req_hash = FileHash(req_file.absolute())

    old_req_hash_file = venv_path / "requirements_sha256.txt"
    if req_hash.is_the_same_as(old_req_hash_file.absolute()):
        print(
            "Requirements file up to date, no need to reinstall dependencies",
            file=sys.stderr,
        )
    else:
        print(
            "Requirements file changed or not installed, installing dependencies...",
            file=sys.stderr,
        )
        venv.install_dependencies(req_file.absolute())
        req_hash.save_to_file(old_req_hash_file.absolute())

    args_no_script_name = sys.argv[1:]
    subprocess.run(
        [
            venv.python_path,
            "-m",
            "pytest",
            "-vv",
            "--build-scenarios",
            "--traces=all",
        ]
        + args_no_script_name
    )


if __name__ == "__main__":
    main()
