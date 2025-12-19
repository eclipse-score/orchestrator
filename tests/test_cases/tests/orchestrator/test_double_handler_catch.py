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
from typing import Any

import pytest
from cit_scenario import CitScenario
from result_code import ResultCode
from testing_utils import ScenarioResult


class TestDoubleSameCatchHandler(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.catch.double_same_handler_error"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": 4}}

    def capture_stderr(self) -> bool:
        return True

    def expect_command_failure(self) -> bool:
        return True

    def test_double_handler_panic(self, results: ScenarioResult):
        assert results.return_code == ResultCode.PANIC
        assert results.stderr
        assert "Catch: Cannot set handler multiple times" in results.stderr


class TestDoubleDiffCatchHandler(TestDoubleSameCatchHandler):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.catch.double_diff_handler_error"
