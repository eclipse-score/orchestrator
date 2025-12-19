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
from testing_utils import LogContainer
from testing_utils.scenario import ScenarioResult


class TestDemo(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "basic.demo"

    @pytest.fixture(scope="class", params=[50, 100, 500])
    def cycle_duration_ms(self, request) -> int:
        return request.param

    @pytest.fixture(scope="class")
    def test_config(self, cycle_duration_ms) -> dict[str, Any]:
        return {
            "runtime": {"task_queue_size": 256, "workers": 3},
            "test": {"cycle_duration_ms": cycle_duration_ms},
        }

    @pytest.fixture(
        scope="class",
        params=[
            pytest.param(2.0),
            pytest.param(
                60.0, marks=(pytest.mark.only_nightly, pytest.mark.do_not_repeat)
            ),  # should be stable for at least 60s
        ],
    )
    def execution_timeout(self, request: pytest.FixtureRequest) -> float:
        return request.param

    def expect_command_failure(self) -> bool:
        # Program is executed continuously until timeout and then killed
        return True

    def test_demo_program_stable(self, results: ScenarioResult):
        assert results.hang, "Demo program did not run until timeout"
        assert results.return_code == ResultCode.SIGKILL, "Demo program did not return error code as expected"

    @pytest.mark.parametrize("program", ["ACC", "S2M", "M2S"])
    def test_program(
        self,
        cycle_duration_ms: int,
        execution_timeout: int,
        logs_info_level: LogContainer,
        program: str,
    ):
        start = logs_info_level.find_log(field="message", value=f"Start{program} was executed")
        program_startup_time = start.timestamp.total_seconds()
        assert program_startup_time < 0.5, f"Program {program} start took too long"
        program_run_time = execution_timeout - program_startup_time
        expected_iterations = int(program_run_time * 1000 / cycle_duration_ms)
        allowed_iteration_deviation = 2

        logs = logs_info_level.get_logs(field="message", value=f"Run{program} was executed")
        assert len(logs) == pytest.approx(expected_iterations, abs=allowed_iteration_deviation), (
            f"Program {program} was executed unexpected number of times"
        )

    def test_all_programs_started(self, logs_info_level: LogContainer):
        assert logs_info_level.contains_log(field="message", value="StartACC was executed"), (
            "Program ACC did not start as expected"
        )
        assert logs_info_level.contains_log(field="message", value="StartM2S was executed"), (
            "Program M2S did not start as expected"
        )
        assert logs_info_level.contains_log(field="message", value="StartS2M was executed"), (
            "Program S2M did not start as expected"
        )

    def test_all_programs_not_stopped(self, logs_info_level: LogContainer):
        assert not logs_info_level.contains_log(field="message", value="StopACC was executed"), (
            "Program ACC was stopped unexpectedly"
        )
        assert not logs_info_level.contains_log(field="message", value="StopM2S was executed"), (
            "Program M2S was stopped unexpectedly"
        )
        assert not logs_info_level.contains_log(field="message", value="StopS2M was executed"), (
            "Program S2M was stopped unexpectedly"
        )
