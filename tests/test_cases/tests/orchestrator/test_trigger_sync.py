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
from datetime import timedelta
from typing import Any

import pytest
from cit_scenario import CitScenario
from testing_utils import LogContainer

BLOCKING_TASK_ID = "blocking_sleep_task"
BLOCKING_TASK_DELAY_MS = 1000
BASIC_TASK_A_ID = "basic_task_A"
BASIC_TASK_B_ID = "basic_task_B"


class TestOneTriggerOneSyncTwoPrograms(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.trigger_sync.1_trigger_1_sync_2_programs"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": 4}}

    def test_execution_order(self, logs_info_level: LogContainer):
        results = list(logs_info_level)
        assert len(results) == 3, "Expected 3 messages in total"

        sleep_begin_msg = results.pop(0)
        assert sleep_begin_msg.id == BLOCKING_TASK_ID, "Expected the first message to have id equal to BLOCKING_TASK_ID"
        assert sleep_begin_msg.location == "begin", (
            "Expected the first message to be the start of the blocking sleep task"
        )

        sleep_end_msg = results.pop(0)
        assert sleep_end_msg.id == BLOCKING_TASK_ID, "Expected the second message to have id equal to BLOCKING_TASK_ID"
        assert sleep_end_msg.location == "end", "Expected the second message to be the end of the blocking sleep task"

        basic_task_msg = results.pop(0)
        assert basic_task_msg.id == BASIC_TASK_A_ID, f"Expected the third message to be {BASIC_TASK_A_ID}"

    def test_execution_delay(self, logs_info_level: LogContainer):
        sleep_begin_msg = logs_info_level.get_logs("id", value=BLOCKING_TASK_ID).find_log("location", value="begin")
        sleep_end_msg = logs_info_level.get_logs("id", value=BLOCKING_TASK_ID).find_log("location", value="end")

        assert sleep_end_msg.timestamp - sleep_begin_msg.timestamp >= timedelta(milliseconds=BLOCKING_TASK_DELAY_MS), (
            "Expected the blocking sleep task to take at least "
            f"{BLOCKING_TASK_DELAY_MS} ms, but it took "
            f"{sleep_end_msg.timestamp - sleep_begin_msg.timestamp}"
        )


class TestOneTriggerTwoSyncsThreePrograms(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.trigger_sync.1_trigger_2_syncs_3_programs"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": 4}}

    def test_execution_order(self, scenario_name, test_config, logs_info_level: LogContainer):
        results = list(logs_info_level)
        assert len(results) == 4, "Expected 4 messages in total"

        sleep_begin_msg = results.pop(0)
        assert sleep_begin_msg.id == BLOCKING_TASK_ID, "Expected the first message to have id equal to BLOCKING_TASK_ID"
        assert sleep_begin_msg.location == "begin", (
            "Expected the first message to be the start of the blocking sleep task"
        )

        sleep_end_msg = results.pop(0)
        assert sleep_end_msg.id == BLOCKING_TASK_ID, "Expected the second message to have id equal to BLOCKING_TASK_ID"
        assert sleep_end_msg.location == "end", "Expected the second message to be the end of the blocking sleep task"

        expected_basic_task_ids = {BASIC_TASK_A_ID, BASIC_TASK_B_ID}
        basic_task_msg = results.pop(0)
        assert basic_task_msg.id in expected_basic_task_ids, (
            f"Expected the third message to be one of {expected_basic_task_ids}, "
        )
        expected_basic_task_ids.remove(basic_task_msg.id)

        basic_task_msg = results.pop(0)
        assert basic_task_msg.id in expected_basic_task_ids, (
            "Expected the fourth message to be {expected_basic_task_ids}"
        )

    def test_execution_delay(self, logs_info_level: LogContainer):
        sleep_begin_msg = logs_info_level.get_logs("id", value=BLOCKING_TASK_ID).find_log("location", value="begin")
        sleep_end_msg = logs_info_level.get_logs("id", value=BLOCKING_TASK_ID).find_log("location", value="end")

        assert sleep_end_msg.timestamp - sleep_begin_msg.timestamp >= timedelta(milliseconds=BLOCKING_TASK_DELAY_MS), (
            "Expected the blocking sleep task to take at least "
            f"{BLOCKING_TASK_DELAY_MS} ms, but it took "
            f"{sleep_end_msg.timestamp - sleep_begin_msg.timestamp}"
        )


class TestTriggerAndSyncInNestedBranches(TestOneTriggerOneSyncTwoPrograms):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.trigger_sync.nested_branches"


class TestTriggerSyncOneAfterAnother(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.trigger_sync.one_after_another"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": 1}}

    def test_execution_order(self, logs_info_level: LogContainer):
        results = list(logs_info_level)
        assert len(results) == 2, "Expected 2 messages in total"

        basic_task_a_msg = results.pop(0)
        assert basic_task_a_msg.id == BASIC_TASK_A_ID, f"Expected the first message to be {BASIC_TASK_A_ID}"

        basic_task_b_msg = results.pop(0)
        assert basic_task_b_msg.id == BASIC_TASK_B_ID, f"Expected the first message to be {BASIC_TASK_B_ID}"
