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


class TestUnrecoverableCatchSequence(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.catch.sequence_user_error"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "test": {
                "design_type": "unrecoverable",
                "error_code": 42,
                "run_count": 1,
            },
        }

    def test_execution_order(self, test_config, logs_info_level: LogContainer):
        results = list(logs_info_level)
        assert len(results) == 2, "Expected 2 messages in total"

        error_code = test_config["test"]["error_code"]

        error_msg = results.pop(0)
        assert error_msg.id == "user_error_task", "Expected error triggering task: wrong id"
        assert error_msg.error_code == error_code, "Expected error triggering task: wrong error_code"

        catch_msg = results.pop(0)
        assert catch_msg.id == "catch", "Expected catch block to handle the user error: wrong id"
        assert catch_msg.error_code == error_code, "Expected catch block to handle the user error: wrong error_code"


class TestRecoverableFailedCatchSequence(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.catch.sequence_user_error"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "test": {
                "design_type": "recoverable_false",
                "error_code": 42,
                "run_count": 1,
            },
        }

    def test_execution_order(self, test_config, logs_info_level: LogContainer):
        results = list(logs_info_level)
        assert len(results) == 2, "Expected 2 messages in total"

        error_code = test_config["test"]["error_code"]

        error_msg = results.pop(0)
        assert error_msg.id == "user_error_task", "Expected error triggering task: wrong id"
        assert error_msg.error_code == error_code, "Expected error triggering task: wrong error_code"

        catch_msg = results.pop(0)
        assert catch_msg.id == "catch_recoverable", "Expected catch block to handle the user error: wrong id"
        assert catch_msg.error_code == error_code, "Expected catch block to handle the user error: wrong error_code"
        assert catch_msg.is_recoverable is False, (
            "Expected catch block to handle the user error: is_recoverable should be False"
        )


class TestRecoverableCatchSequence(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.catch.sequence_user_error"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "test": {
                "design_type": "recoverable_true",
                "error_code": 42,
                "run_count": 1,
            },
        }

    def test_execution_order(self, test_config, logs_info_level: LogContainer):
        results = list(logs_info_level)
        assert len(results) == 3, "Expected 3 messages in total"

        error_code = test_config["test"]["error_code"]

        error_msg = results.pop(0)
        assert error_msg.id == "user_error_task", "Expected error triggering task: wrong id"
        assert error_msg.error_code == error_code, "Expected error triggering task: wrong error_code"

        catch_msg = results.pop(0)
        assert catch_msg.id == "catch_recoverable", "Expected catch block to handle the user error: wrong id"
        assert catch_msg.error_code == error_code, "Expected catch block to handle the user error: wrong error_code"
        assert catch_msg.is_recoverable is True, (
            "Expected catch block to handle the user error: is_recoverable should be True"
        )

        after_catch_msg = results.pop(0)
        assert after_catch_msg.id == "log_after_catch_task", "Expected task after catch block to be executed"


class TestUnrecoverableCatchInMultipleRuns(TestUnrecoverableCatchSequence):
    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "test": {
                "design_type": "unrecoverable",
                "error_code": 42,
                "run_count": 3,
            },
        }


class TestRecoverableFailedCatchInMultipleRuns(TestRecoverableFailedCatchSequence):
    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "test": {
                "design_type": "recoverable_false",
                "error_code": 42,
                "run_count": 3,
            },
        }


class TestRecoverableCatchInMultipleRuns(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.catch.sequence_user_error"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "test": {
                "design_type": "recoverable_true",
                "error_code": 42,
                "run_count": 3,
            },
        }

    def test_execution_count(self, test_config, logs_info_level: LogContainer):
        expected_iter_count = test_config["test"]["run_count"]

        logs = list(logs_info_level)
        for _ in range(expected_iter_count):
            log = logs.pop(0)
            assert log.id == "user_error_task", "Expected user error task to be executed"

            log = logs.pop(0)
            assert log.id == "catch_recoverable", "Expected catch block to handle the user error"

            log = logs.pop(0)
            assert log.id == "log_after_catch_task", "Expected task after catch block to be executed"

        assert len(logs) == 0, "Expected no additional logs after the expected iterations"


class TestNestedSequenceCatch(TestUnrecoverableCatchSequence):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.catch.nested_sequence_user_error"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "test": {"error_code": 42},
        }


class TestConcurrencyCatch(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.catch.concurrency_user_error"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "test": {
                "concurrent_valid_tasks": ["task_A", "task_B", "task_C"],
                "error_code": 42,
            },
        }

    def test_execution_completeness(self, test_config, logs_info_level: LogContainer):
        valid_tasks = test_config["test"]["concurrent_valid_tasks"]

        for task in valid_tasks:
            assert logs_info_level.contains_log(field="id", value=task), f"Expected task {task} to be executed"

        assert logs_info_level.contains_log(field="id", value="user_error_task"), (
            "Expected user error task to be executed"
        )

        last_message = list(logs_info_level).pop(-1)
        catch_message = logs_info_level.find_log(field="id", value="catch")
        assert last_message == catch_message, "Expected last message to be the catch block"


class TestNestedConcurrencyCatch(TestConcurrencyCatch):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.catch.nested_concurrency_user_error"


class TestDoubleMixedErrorCatch(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.catch.double_mixed_user_error"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "test": {
                "error_codes": [
                    18446744073709551615,
                    43,
                ]  # The latter is unrecoverable
            },
        }

    def test_concurrent_branches_execution(self, test_config, logs_info_level: LogContainer):
        error_codes = test_config["test"]["error_codes"]

        concurrent_branches = 3
        concurrency_tasks = LogContainer(logs_info_level[:concurrent_branches])

        assert concurrency_tasks.contains_log(field="error_code", value=error_codes[0]), (
            f"Expected user error task for code {error_codes[0]} to be executed"
        )

        assert concurrency_tasks.contains_log(field="error_code", value=error_codes[1]), (
            f"Expected user error task for code {error_codes[1]} to be executed"
        )

        assert concurrency_tasks.contains_log(field="id", value="just_log_task"), (
            "Expected 'just_log_task' to be executed in concurrency branches"
        )

    def test_error_code_arbitration(self, test_config, logs_info_level: LogContainer):
        error_codes = test_config["test"]["error_codes"]

        catch_message = logs_info_level.find_log(field="id", value="catch_recoverable")
        assert catch_message.error_code == error_codes[1], (
            f"Expected catch block to get an error log from the last executed branch {error_codes[1]}"
        )


class TestDoubleRecoverableErrorCatch(TestDoubleMixedErrorCatch):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.catch.double_recoverable_user_error"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "test": {"error_codes": [0, 43]},
        }

    def test_execution_continuation(self, logs_info_level: LogContainer):
        last_message = logs_info_level[-1]
        assert last_message.id == "log_after_catch_task", (
            "Expected 'log_after_catch_task' to be executed after catch block"
        )


class TestDoubleCatchSequence(TestUnrecoverableCatchSequence):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.catch.per_nested_sequence"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "test": {"error_code": 42},
        }
