#!/bin/bash
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

# For the documentation, please refer to tests/test_cases/run_tests.py
#
# Example usage:
#     Run all tests:
#     ./run_component_tests.sh
#
#     Run all tests in a specific file (file path must be relative to test_cases folder):
#     ./run_component_tests.sh tests/runtime/worker/test_worker_basic.py
#
#     Run a specific scenario:
#     ./run_component_tests.sh tests/runtime/worker/test_worker_basic.py::TestRuntimeOneWorkerOneTask

FILE_PATH=$(realpath "$0")
FILE_DIR=$(dirname "$FILE_PATH")

cd "$FILE_DIR"/../tests/test_cases || exit 1
./run_tests.py "$@"
