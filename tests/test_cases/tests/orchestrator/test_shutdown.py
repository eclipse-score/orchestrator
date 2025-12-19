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
import pytest
from cit_scenario import CitScenario
from result_code import ResultCode
from testing_utils.log_container import LogContainer
from testing_utils.scenario import ScenarioResult


class TestSingleProgramSingleShutdown(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self):
        return "orchestration.shutdown.single_program_single_shutdown"

    @pytest.fixture(scope="class")
    def execution_timeout(self, request, *args, **kwargs):
        return 1.0

    @pytest.fixture(scope="class", params=[1, 2, 42])
    def workers(self, request: pytest.FixtureRequest) -> int:
        return request.param

    @pytest.fixture(scope="class")
    def test_config(self, workers: int):
        return {"runtime": {"task_queue_size": 256, "workers": workers}}

    def test_program_executed(self, logs_info_level: LogContainer):
        assert logs_info_level.contains_log(field="message", pattern="Action1"), "Action1 was not executed as expected"

    def test_shutdown_executed(self, logs_info_level: LogContainer):
        assert logs_info_level.contains_log(field="message", pattern="StopAction"), (
            "Shutdown was not executed as expected"
        )


class TestTwoProgramsSingleShutdown(TestSingleProgramSingleShutdown):
    @pytest.fixture(scope="class")
    def scenario_name(self):
        return "orchestration.shutdown.two_programs_single_shutdown"

    def test_shutdown_executed(self, logs_info_level: LogContainer):
        assert logs_info_level.contains_log(field="message", pattern="ShutdownDesign1::StopAction was executed"), (
            "Program1 Shutdown was not executed as expected"
        )
        assert logs_info_level.contains_log(field="message", pattern="ShutdownDesign2::StopAction was executed"), (
            "Program2 Shutdown was not executed as expected"
        )


class TestTwoProgramsTwoShutdowns(TestTwoProgramsSingleShutdown):
    @pytest.fixture(scope="class")
    def scenario_name(self):
        return "orchestration.shutdown.two_programs_two_shutdowns"

    def test_shutdown_order(self, logs_info_level: LogContainer):
        shutdown1 = logs_info_level.find_log(field="message", pattern="ShutdownDesign1::StopAction was executed")
        shutdown2 = logs_info_level.find_log(field="message", pattern="ShutdownDesign2::StopAction was executed")

        assert shutdown2.timestamp < shutdown1.timestamp, "Program2 Shutdown did not happen before Program1 Shutdown"


class TestTwoProgramsAllShutdowns(TestTwoProgramsSingleShutdown):
    @pytest.fixture(scope="class")
    def scenario_name(self):
        return "orchestration.shutdown.two_programs_all_shutdowns"


class TestOneProgramNotShut(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self):
        return "orchestration.shutdown.one_program_not_shut"

    @pytest.fixture(scope="class", params=[1, 2, 42])
    def workers(self, request: pytest.FixtureRequest) -> int:
        return request.param

    @pytest.fixture(scope="class")
    def test_config(self, workers: int):
        return {"runtime": {"task_queue_size": 256, "workers": workers}}

    @pytest.fixture(scope="class")
    def execution_timeout(self, request, *args, **kwargs):
        return 1.0

    def expect_command_failure(self) -> bool:
        return True

    def test_infinite_design_was_executed(self, logs_info_level: LogContainer):
        assert logs_info_level.contains_log(field="message", pattern="InfiniteDesign::Action1 was executed"), (
            "InfiniteDesign::Action1 was not executed as expected"
        )

    def test_program_execution_is_running_infinitely(self, results: ScenarioResult):
        assert results.hang, "Program execution was not running infinitely as expected"
        # The program should run infinitely, so we kill it after a execution_timeout
        assert results.return_code == ResultCode.SIGKILL, "Program execution was not killed as expected"

    def test_shutdown_designs_were_executed_and_shut(self, logs_info_level: LogContainer):
        # Design1
        assert logs_info_level.contains_log(field="message", pattern="ShutdownDesign1::Action1 was executed"), (
            "ShutdownDesign1 was not executed as expected"
        )
        assert logs_info_level.contains_log(field="message", pattern="ShutdownDesign1::StopAction was executed"), (
            "ShutdownDesign1 was not shut down as expected"
        )

        # Design2
        assert logs_info_level.contains_log(field="message", pattern="ShutdownDesign2::Action1 was executed"), (
            "ShutdownDesign2 was not executed as expected"
        )
        assert logs_info_level.contains_log(field="message", pattern="ShutdownDesign2::StopAction was executed"), (
            "ShutdownDesign2 was not shut down as expected"
        )


class TestShutdownBeforeStart(TestSingleProgramSingleShutdown):
    @pytest.fixture(scope="class")
    def scenario_name(self):
        return "orchestration.shutdown.before_start"

    @pytest.mark.skip("Behavior to be clarified - https://github.com/qorix-group/inc_orchestrator_internal/issues/148")
    def test_execution_order(self, logs_info_level: LogContainer):
        actions = logs_info_level.get_logs(field="message", pattern="Action1 was executed")
        shutdown = logs_info_level.find_log(field="message", pattern="StopAction was executed")
        # Current implementation - first action executes before shutdown
        # To be changed when shutdown behavior is modified
        assert actions[0].timestamp < shutdown.timestamp, (
            "Shutdown was executed after the program started, which is not expected"
        )
