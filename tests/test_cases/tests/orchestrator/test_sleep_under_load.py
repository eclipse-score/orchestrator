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
import math
from typing import Any

import pytest
from cit_scenario import CitScenario
from testing_utils import LogContainer


class TestShortSleepUnderLoad2W256Q(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.sleep.under_load"

    @pytest.fixture(scope="class")
    def sleep_duration_ms(self) -> int:
        return 100

    @pytest.fixture(scope="class")
    def test_config(self, sleep_duration_ms: int) -> dict[str, Any]:
        return {
            "runtime": {"workers": 2, "task_queue_size": 256},
            "test": {
                "sleep_duration_ms": sleep_duration_ms,
                "run_count": 1,
                "cpu_load": "high",
            },
        }

    @pytest.fixture(scope="class")
    def execution_timeout(self) -> float:
        return 10.0

    def test_start_and_finish(self, logs_info_level: LogContainer):
        assert logs_info_level[0].message == "StartAction was executed", (
            "Program not started with first action in sequence"
        )
        assert logs_info_level[-1].message == "FinishAction was executed", (
            "Program not finished with last action in sequence"
        )

    @pytest.mark.parametrize(
        "sleep_name",
        ["Sleep1", "Sleep2", "Sleep3", "Sleep4", "Sleep5"],
    )
    def test_sleep_duration_strict(
        self,
        logs_info_level: LogContainer,
        sleep_duration_ms: int,
        sleep_name: str,
    ):
        (sleep_start, sleep_finish) = logs_info_level.get_logs(
            field="id",
            value=sleep_name,
        )
        measured_sleep_duration = (sleep_finish.timestamp - sleep_start.timestamp).total_seconds() * 1000  # ms
        assert sleep_duration_ms <= measured_sleep_duration, "Sleep finished too early"


class TestMediumSleepUnderLoad2W256Q(TestShortSleepUnderLoad2W256Q):
    @pytest.fixture(scope="class")
    def sleep_duration_ms(self) -> int:
        return 500

    @pytest.fixture(scope="class")
    def execution_timeout(self) -> float:
        return 10.0


class TestLongSleepUnderLoad2W256Q(TestMediumSleepUnderLoad2W256Q):
    @pytest.fixture(scope="class")
    def sleep_duration_ms(self) -> int:
        return 1000

    @pytest.fixture(scope="class")
    def execution_timeout(self) -> float:
        return 10.0


@pytest.mark.do_not_repeat
@pytest.mark.only_nightly
class TestHugeAmountOfShortSleeps(TestShortSleepUnderLoad2W256Q):
    @pytest.fixture(scope="class")
    def sleep_duration_ms(self) -> int:
        return 5

    @pytest.fixture(scope="class")
    def test_config(self, sleep_duration_ms: int) -> dict[str, Any]:
        return {
            "runtime": {"workers": 2, "task_queue_size": 256},
            "test": {
                "sleep_duration_ms": sleep_duration_ms,
                "run_count": 2_000,
                "cpu_load": "low",
            },
        }

    @pytest.fixture(scope="class")
    def execution_timeout(self) -> float:
        return 120.0

    @pytest.mark.parametrize(
        "sleep_name",
        ["Sleep1", "Sleep2", "Sleep3", "Sleep4", "Sleep5"],
    )
    def test_sleep_duration_strict(
        self,
        logs_info_level: LogContainer,
        sleep_duration_ms: int,
        sleep_name: str,
    ):
        # Collect all start and finish logs for the given sleep_name
        location_group = logs_info_level.get_logs(field="id", value=sleep_name).group_by("location")
        sleep_starts = location_group["begin"]
        sleep_finishes = location_group["end"]

        # Calculate duration of each sleep
        for sleep_start, sleep_finish in zip(sleep_starts, sleep_finishes):
            measured_sleep_duration = (sleep_finish.timestamp - sleep_start.timestamp).total_seconds() * 1000
            assert measured_sleep_duration >= sleep_duration_ms, (
                f"Expected sleep duration at least {sleep_duration_ms} ms, but got {measured_sleep_duration} ms."
            )
