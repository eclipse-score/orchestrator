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
from testing_utils import LogContainer


class TestSingleSequence1W256Q(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.sequence.single"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": 1}}

    def test_execution_order_one_branch(self, logs_info_level: LogContainer):
        action1 = logs_info_level.find_log(field="message", value="Action1 was executed")
        action2 = logs_info_level.find_log(field="message", value="Action2 was executed")
        action3 = logs_info_level.find_log(field="message", value="Action3 was executed")
        # Assert that execution_order is chronological by timestamp
        assert action1.timestamp < action2.timestamp < action3.timestamp, (
            "Actions were not executed in the expected order: Action1, Action2, Action3"
        )


class TestSingleSequence2W256Q(TestSingleSequence1W256Q):
    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": 2}}


class TestNestedSequence1W256Q(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.sequence.nested"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": 1}}

    def test_outer_sequence_executed(self, logs_info_level: LogContainer):
        assert logs_info_level.contains_log(field="message", pattern="OuterAction*"), (
            "OuterAction was not executed as expected"
        )

    def test_inner_sequence_executed(self, logs_info_level: LogContainer):
        assert logs_info_level.contains_log(field="message", pattern="InnerAction*"), (
            "InnerAction was not executed as expected"
        )

    def test_execution_order_sequence_in_sequence(self, logs_info_level: LogContainer):
        expected_order = [
            "OuterAction1 was executed",
            "InnerAction1 was executed",
            "InnerAction2 was executed",
            "OuterAction2 was executed",
        ]
        execution_order = [log.message for log in logs_info_level.get_logs(field="message", pattern="was executed")]
        assert execution_order == expected_order, "Actions were not executed in the expected order"


class TestNestedSequence2W256Q(TestNestedSequence1W256Q):
    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": 2}}
