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
from testing_utils.log_container import LogContainer


class CommonDedicatedWorkerConfig(CitScenario):
    @pytest.fixture(scope="class")
    def num_dedicated(self) -> int:
        return 3

    @pytest.fixture(scope="class")
    def dedicated_workers(self, num_dedicated: int) -> list[dict[str, Any]]:
        result = []
        for i in range(num_dedicated):
            result.append({"id": f"dedicated_worker_{i}"})
        return result

    @pytest.fixture(scope="class")
    def test_config(self, dedicated_workers: list[dict[str, Any]]) -> dict[str, Any]:
        return {
            "runtime": {
                "task_queue_size": 256,
                "workers": 1,
                "dedicated_workers": dedicated_workers,
            }
        }


class TestDedicatedWorkerBindTags(CommonDedicatedWorkerConfig):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.dedicated_worker.bind_tags"

    def test_dedicated_assignment(self, logs_info_level: LogContainer):
        # All async functions are assigned to the same dedicated worker 0
        logs_dedicated = logs_info_level.get_logs(field="message", pattern=r"\'async\d\'.*")
        dedicated_threads = {log.thread_id for log in logs_dedicated}
        assert len(dedicated_threads) == 1, "Expected execution only on 1 dedicated worker"

        # Sync1 task is assigned to dedicated worker 1
        logs_sync1 = logs_info_level.get_logs(field="message", pattern=r"\'sync1\'.*")
        sync1_threads = {log.thread_id for log in logs_sync1}
        assert len(sync1_threads) == 1, "Expected execution only on 1 dedicated"

        # Sync2 task is assigned to dedicated worker 2
        logs_sync2 = logs_info_level.get_logs(field="message", pattern=r"\'sync2\'.*")
        sync2_threads = {log.thread_id for log in logs_sync2}
        assert len(sync2_threads) == 1, "Expected execution only on 1 dedicated"


class TestDedicatedWorkerReuse(CommonDedicatedWorkerConfig):
    @pytest.fixture(scope="class")
    def num_dedicated(self) -> int:
        return 1

    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.dedicated_worker.dedicated_works_on_regular"

    def test_dedicated_assignment(self, logs_info_level: LogContainer):
        # Find out which thread was assigned to the 'sync1' task - it should be a dedicated worker
        dedicated_thread_id = logs_info_level.get_logs(field="message", pattern=r"\'sync1\'")[0].thread_id

        # Find all other tasks in design and extract their thread ids
        other_logs = logs_info_level.get_logs(field="message", pattern=r"^(?!.*'sync1').*$")
        other_thread_ids = {log.thread_id for log in other_logs}

        # Check that the dedicated worker thread was reused for other tasks
        assert dedicated_thread_id in other_thread_ids, "Dedicated worker should be reused for other tasks"


class TestDedicatedWorkerRepeatedAssignment(CommonDedicatedWorkerConfig):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.dedicated_worker.repeat_tag_assignment"

    def capture_stderr(self):
        return True

    def expect_command_failure(self):
        return True

    def test_expected_error(self, results: ScenarioResult):
        assert results.return_code == ResultCode.PANIC
        assert "Failed to bind invoke action to worker: AlreadyDone" in results.stderr


class TestNonExistentTag(CommonDedicatedWorkerConfig):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.dedicated_worker.assign_non_existent_tag"

    def capture_stderr(self):
        return True

    def expect_command_failure(self):
        return True

    def test_expected_error(self, results: ScenarioResult):
        assert results.return_code == ResultCode.PANIC
        assert "Failed to bind invoke action to worker: NotFound" in results.stderr


class TestNonExistentDedicatedWorker(CommonDedicatedWorkerConfig):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.dedicated_worker.assign_to_non_existent_dedicated_worker"

    def capture_stderr(self):
        return True

    def expect_command_failure(self):
        return True

    @pytest.mark.xfail(reason="Currently fails with SIGABRT instead of PANIC")
    def test_expected_error(self, results: ScenarioResult):
        assert results.return_code == ResultCode.PANIC
        assert "Tried to spawn on not registered dedicated worker" in results.stderr
