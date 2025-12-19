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


class TestSingleConcurrency1W256Q(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.concurrency.single"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": 1}}

    def test_each_function_standalone_order(self, logs_info_level: LogContainer):
        for function in ["Function1", "Function2", "Function3"]:
            expected_order = [
                f"Start of '{function}' function",
                f"End of '{function}' function",
            ]
            execution_order = [log.message for log in logs_info_level.get_logs(field="message", pattern=function)]

            assert execution_order == expected_order, f"Execution order for {function} is incorrect"

    def test_concurrency_finished_before_final_task(self, logs_info_level: LogContainer):
        # Check if all branches in Concurrency were executed before next Sequence step
        finish_action = logs_info_level.find_log(field="message", value="FinishAction was executed")

        end_fun_1 = logs_info_level.find_log(field="message", value="End of 'Function1' function")
        assert end_fun_1.timestamp < finish_action.timestamp, (
            "Function1 execution should be finished before FinishAction was executed"
        )
        end_fun_2 = logs_info_level.find_log(field="message", value="End of 'Function2' function")
        assert end_fun_2.timestamp < finish_action.timestamp, (
            "Function2 execution should be finished before FinishAction was executed"
        )
        end_fun_3 = logs_info_level.find_log(field="message", value="End of 'Function3' function")

        assert end_fun_3.timestamp < finish_action.timestamp, (
            "Function3 execution should be finished before FinishAction was executed"
        )


class TestSingleConcurrency2W256Q(TestSingleConcurrency1W256Q):
    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": 2}}


class TestMultipleConcurrency1W256Q(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.concurrency.multiple"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": 1}}

    def test_concurrency1_finished_before_final_task(self, logs_info_level: LogContainer):
        # Check if all branches in Concurrency1 were executed before FinishAction
        finish_action = logs_info_level.find_log(field="message", value="FinishAction was executed")

        end_fun_1 = logs_info_level.find_log(field="message", value="End of 'Function1' function")
        assert end_fun_1.timestamp < finish_action.timestamp, (
            "Function1 execution should be finished before FinishAction was executed"
        )
        end_fun_2 = logs_info_level.find_log(field="message", value="End of 'Function2' function")
        assert end_fun_2.timestamp < finish_action.timestamp, (
            "Function2 execution should be finished before FinishAction was executed"
        )
        end_fun_3 = logs_info_level.find_log(field="message", value="End of 'Function3' function")

        assert end_fun_3.timestamp < finish_action.timestamp, (
            "Function3 execution should be finished before FinishAction was executed"
        )

    def test_concurrency1_finished_before_concurrency2_start(self, logs_info_level: LogContainer):
        # Check if all branches in Concurrency1 were executed before starting Concurrency2
        # Find all functions starts from Concurrency2 and pick the first one by timestamp
        functions_starts_from_concurrency2 = logs_info_level.get_logs(
            field="message", pattern="Start of 'Function(4|5|6)' function"
        )
        concurrency2_start = functions_starts_from_concurrency2[0]

        end_fun_1 = logs_info_level.find_log(field="message", value="End of 'Function1' function")
        assert end_fun_1.timestamp < concurrency2_start.timestamp, (
            "Function1 execution from Concurrency1 block should be finished before Concurrency2 was started"
        )
        end_fun_2 = logs_info_level.find_log(field="message", value="End of 'Function2' function")
        assert end_fun_2.timestamp < concurrency2_start.timestamp, (
            "Function2 execution from Concurrency1 block should be finished before Concurrency2 was started"
        )
        end_fun_3 = logs_info_level.find_log(field="message", value="End of 'Function3' function")

        assert end_fun_3.timestamp < concurrency2_start.timestamp, (
            "Function3 execution from Concurrency1 block should be finished before Concurrency2 was started"
        )

    def test_concurrency2_finished_before_final_task(self, logs_info_level: LogContainer):
        # Check if all branches in Concurrency2 were executed before FinishAction
        finish_action = logs_info_level.find_log(field="message", value="FinishAction was executed")

        end_fun_4 = logs_info_level.find_log(field="message", value="End of 'Function4' function")
        assert end_fun_4.timestamp < finish_action.timestamp, (
            "Function4 execution should be finished before FinishAction was executed"
        )
        end_fun_5 = logs_info_level.find_log(field="message", value="End of 'Function5' function")
        assert end_fun_5.timestamp < finish_action.timestamp, (
            "Function5 execution should be finished before FinishAction was executed"
        )
        end_fun_6 = logs_info_level.find_log(field="message", value="End of 'Function6' function")

        assert end_fun_6.timestamp < finish_action.timestamp, (
            "Function6 execution should be finished before FinishAction was executed"
        )

    def test_concurrency1_execution_order(self, logs_info_level: LogContainer):
        # Find all functions starts from Concurrency1 and pick the first one by timestamp
        functions_starts_from_concurrency1 = logs_info_level.get_logs(
            field="message", pattern="Start of 'Function(1|2|3)' function"
        )
        concurrency1_start = functions_starts_from_concurrency1[0]

        # Find all functions ends from Concurrency1 and pick the last one by timestamp
        functions_ends_from_concurrency1 = logs_info_level.get_logs(
            field="message", pattern="End of 'Function(1|2|3)' function"
        )
        concurrency1_end = functions_ends_from_concurrency1[-1]

        # IntermediateAction should be executed after Concurrency1
        intermediate_action = logs_info_level.find_log(field="message", value="IntermediateAction was executed")

        assert concurrency1_start.timestamp < concurrency1_end.timestamp < intermediate_action.timestamp, (
            "Incorrect execution order. Expected Concurrency1 to finish before IntermediateAction"
        )

    def test_concurrency2_execution_order(self, logs_info_level: LogContainer):
        # IntermediateAction should be executed after Concurrency1
        intermediate_action = logs_info_level.find_log(field="message", value="IntermediateAction was executed")
        # Find all functions starts from Concurrency2 and pick the first one by timestamp
        functions_starts_from_concurrency2 = logs_info_level.get_logs(
            field="message", pattern="Start of 'Function(4|5|6)' function"
        )
        concurrency2_start = functions_starts_from_concurrency2[0]
        # Find all functions ends from Concurrency2 and pick the last one by timestamp
        functions_ends_from_concurrency2 = logs_info_level.get_logs(
            field="message", pattern="End of 'Function(4|5|6)' function"
        )
        concurrency2_end = functions_ends_from_concurrency2[-1]

        # FinishActrion should be executed after Concurrency2
        finish_action = logs_info_level.find_log(field="message", value="FinishAction was executed")

        assert (
            intermediate_action.timestamp
            < concurrency2_start.timestamp
            < concurrency2_end.timestamp
            < finish_action.timestamp
        ), (
            "Incorrect execution order. Expected Concurrency2 to start after IntermediateAction "
            "and finish before FinishAction"
        )


class TestMultipleConcurrency2W256Q(TestMultipleConcurrency1W256Q):
    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": 2}}


class TestMultipleConcurrency5W256Q(TestMultipleConcurrency1W256Q):
    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": 5}}


class TestNestedConcurrency1W256Q(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.concurrency.nested"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": 1}}

    def test_inner_concurrency_finished_before_final_task(self, logs_info_level: LogContainer):
        # Check if all branches in InnerConcurrency were executed before FinishAction
        finish_action = logs_info_level.find_log(field="message", value="FinishAction was executed")

        functions_end_from_inner_concurrency = logs_info_level.get_logs(
            field="message", pattern="End of 'InnerFunction.*' function"
        )
        inner_concurrency_end = functions_end_from_inner_concurrency[-1]
        assert inner_concurrency_end.timestamp < finish_action.timestamp, (
            "Inner concurrency execution should be finished before FinishAction was executed"
        )

    def test_outer_concurrency_finished_before_final_task(self, logs_info_level: LogContainer):
        # Check if all branches in OuterConcurrency were executed before FinishAction
        finish_action = logs_info_level.find_log(field="message", value="FinishAction was executed")

        functions_end_from_outer_concurrency = logs_info_level.get_logs(
            field="message", pattern="End of 'OuterFunction.*' function"
        )
        outer_concurrency_end = functions_end_from_outer_concurrency[-1]
        assert outer_concurrency_end.timestamp < finish_action.timestamp, (
            "Outer concurrency execution should be finished before FinishAction was executed"
        )


class TestNestedConcurrency2W256Q(TestNestedConcurrency1W256Q):
    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": 2}}


class TestNestedConcurrency5W256Q(TestNestedConcurrency1W256Q):
    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {"runtime": {"task_queue_size": 256, "workers": 5}}
