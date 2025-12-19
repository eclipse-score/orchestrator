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
from testing_utils.log_container import LogContainer
from testing_utils.scenario import ScenarioResult


class TestTagMethods(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self):
        return "orchestration.tag_methods.tag_methods"

    @pytest.fixture(scope="class")
    def test_config(self):
        return {"runtime": {"task_queue_size": 256, "workers": 4}}

    @staticmethod
    def compute_djb2_tag_hash(tag: str) -> int:
        """
        Compute DJB2 hash for a UTF-8 string, returning a 64-bit unsigned value.
        Equivalent to the Rust implementation in Tag::compute_djb2_hash.
        """
        tag_bytes = tag.encode("utf-8")
        hash_val = 5381
        for byte in tag_bytes:
            hash_val = ((hash_val << 5) + hash_val + byte) & 0xFFFFFFFFFFFFFFFF
        return hash_val

    def test_design_id(self, logs_info_level: LogContainer):
        log = logs_info_level.find_log(field="message", pattern="Design created")
        assert log.id == "test_design", "Design ID is not as expected"

    def test_tag_creation(self, logs_info_level: LogContainer):
        log = logs_info_level.find_log(field="message", pattern="Tag created")

        assert log.tracing_str == "sample_method", "Tag ID is not as expected"
        assert log.id == self.compute_djb2_tag_hash("sample_method"), "Tag ID is not as expected"

    def test_valid_tag_is_in_collection(self, logs_info_level: LogContainer):
        log = logs_info_level.get_logs(field="is_in_collection").find_log(field="tag", value="sample_method")
        assert log.is_in_collection, "Tag was not found in collection as expected"

    def test_invalid_tag_is_in_collection(self, logs_info_level: LogContainer):
        log = logs_info_level.get_logs(field="is_in_collection").find_log(field="tag", value="extra_tag")
        assert not log.is_in_collection, "Tag was found in collection, but it should not be"

    def test_valid_tag_find_in_collection(self, logs_info_level: LogContainer):
        log = logs_info_level.get_logs(field="find_in_collection").find_log(field="tag", value="sample_method")
        assert log.find_in_collection, "Tag was not found in collection as expected"

    def test_invalid_tag_find_in_collection(self, logs_info_level: LogContainer):
        log = logs_info_level.get_logs(field="find_in_collection").find_log(field="tag", value="extra_tag")
        assert not log.find_in_collection, "Tag was found in collection, but it should not be"


class TestRegisterMethod(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self):
        return "orchestration.tag_methods.tag_methods"

    @pytest.fixture(scope="class")
    def test_config(self):
        return {"runtime": {"task_queue_size": 256, "workers": 4}}

    def test_method_executed(self, logs_info_level: LogContainer):
        assert logs_info_level.contains_log(field="id", pattern="sample_method"), (
            "SampleMethod was not executed as expected"
        )

    def test_async_method_executed(self, logs_info_level: LogContainer):
        logs = logs_info_level.get_logs(field="id", pattern="sample_async_method")
        assert len(logs) == 2, "Async SampleMethod should have 2 logs"
        assert logs[0].location == "begin", "Async SampleMethod did not start as expected"
        assert logs[-1].location == "end", "Async SampleMethod did not end as expected"


class CitScenarioWithCorruptedPrograms(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self):
        return "orchestration.tag_methods.error_scenarios"

    def expect_command_failure(self) -> bool:
        return True

    def capture_stderr(self) -> bool:
        return True

    @pytest.fixture(scope="class")
    def test_config(self, program_name):
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "program_name": program_name,
        }


class TestRegisterSameMethodTwice(CitScenarioWithCorruptedPrograms):
    @pytest.fixture(scope="class")
    def program_name(self):
        return "register_same_method_twice"

    def test_invalid(self, results: ScenarioResult):
        assert results.return_code == ResultCode.PANIC, "Test scenario was expected to fail with panic"
        assert "Failed to create design: AlreadyDone" in results.stderr, (
            "Test scenario did not with expected error message"
        )


class TestRegisterSameAsyncMethodTwice(CitScenarioWithCorruptedPrograms):
    @pytest.fixture(scope="class")
    def program_name(self):
        return "register_same_async_method_twice"

    def test_invalid(self, results: ScenarioResult):
        assert results.return_code == ResultCode.PANIC, "Test scenario was expected to fail with panic"
        assert "Failed to create design: AlreadyDone" in results.stderr, (
            "Test scenario did not with expected error message"
        )


class TestGetNonExistingTag(CitScenarioWithCorruptedPrograms):
    @pytest.fixture(scope="class")
    def program_name(self):
        return "get_invalid_tag"

    def test_invalid(self, results: ScenarioResult):
        assert results.return_code == ResultCode.PANIC, "Test scenario was expected to fail with panic"
        assert "Failed to create design: NotFound" in results.stderr, (
            "Test scenario did not with expected error message"
        )


class TestRegisterTooManyTags(CitScenarioWithCorruptedPrograms):
    @pytest.fixture(scope="class")
    def scenario_name(self):
        return "orchestration.tag_methods.too_many_tags"

    @pytest.fixture(scope="class", params=[1, 5, 256])
    def capacity(self, request) -> int:
        return request.param

    @pytest.fixture(scope="class")
    def test_config(self, capacity: int) -> dict[str, Any]:
        return {
            "runtime": {"task_queue_size": 256, "workers": 4},
            "registration_capacity": capacity,
        }

    def test_invalid(self, results: ScenarioResult):
        assert results.return_code == ResultCode.PANIC, "Test scenario was expected to fail with panic"
        assert "Failed to create design: NoSpaceLeft" in results.stderr, (
            "Test scenario did not with expected error message"
        )
