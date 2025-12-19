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
import json
from typing import Any

import pytest
from cit_scenario import CitScenario
from testing_utils import LogContainer


class TestBasicIfElseCondition(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.if_else.basic"

    @pytest.fixture(scope="class", params=[True, False])
    def condition(self, request: pytest.FixtureRequest) -> bool:
        return request.param

    @pytest.fixture(scope="class")
    def test_config(self, condition: bool) -> dict[str, Any]:  # noqa: FBT001
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "test": {"condition": condition},
        }

    def test_execution_branch(self, condition, logs_info_level: LogContainer):
        assert len(logs_info_level) == 1, "Expected exactly one log message"

        condition = json.dumps(condition)
        assert logs_info_level.contains_log(field="id", value=condition), (
            f"Expected execution of task with id={condition}"
        )


class TestNestedIfElseCondition(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.if_else.nested"

    @pytest.fixture(scope="class", params=[True, False])
    def outer_condition(self, request: pytest.FixtureRequest) -> bool:
        return request.param

    @pytest.fixture(scope="class", params=[True, False])
    def inner_condition(self, request: pytest.FixtureRequest) -> bool:
        return request.param

    @pytest.fixture(scope="class")
    def test_config(self, outer_condition: bool, inner_condition: bool) -> dict[str, Any]:  # noqa: FBT001
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "test": {
                "outer_condition": outer_condition,
                "inner_condition": inner_condition,
            },
        }

    def test_execution_branch(self, outer_condition, inner_condition, logs_info_level: LogContainer):
        assert len(logs_info_level) == 1, "Expected exactly one log message"

        outer_condition = json.dumps(outer_condition)
        inner_condition = json.dumps(inner_condition)
        expected_id = f"{outer_condition}_{inner_condition}"
        assert logs_info_level.contains_log(field="id", value=expected_id), (
            f"Expected execution of task with id={expected_id}"
        )
