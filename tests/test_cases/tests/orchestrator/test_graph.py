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
from abc import abstractmethod
from typing import Any

import pytest
from cit_scenario import CitScenario
from result_code import ResultCode
from testing_utils import ScenarioResult
from testing_utils.log_container import LogContainer


# region Positive Scenarios
class CommonGraphProgramConfig(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.graphs.graph_program"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {
                "task_queue_size": 256,
                "workers": 4,
            },
            "test": {"graph_name": self.graph_name()},
        }

    @abstractmethod
    def graph_name(self) -> str: ...

    @pytest.fixture(scope="class")
    def logs_nodes(self, logs_info_level: LogContainer) -> LogContainer:
        return logs_info_level.get_logs(field="message", pattern=r"node\d+")


class TestGraphTwoNodes(CommonGraphProgramConfig):
    def graph_name(self) -> str:
        return "two_nodes"

    def test_valid(self, logs_nodes: LogContainer):
        assert len(logs_nodes) == 2
        # Edges guarantee order
        assert logs_nodes[0].message == "node0 was executed"
        assert logs_nodes[1].message == "node1 was executed"


class TestGraphNoEdges(CommonGraphProgramConfig):
    def graph_name(self) -> str:
        return "no_edges"

    def test_valid(self, logs_nodes: LogContainer):
        assert len(logs_nodes) == 2

        # No edges, so order is not guaranteed
        messages = [log.message for log in logs_nodes]
        assert "node0 was executed" in messages
        assert "node1 was executed" in messages


class TestGraphOneNode(CommonGraphProgramConfig):
    def graph_name(self) -> str:
        return "one_node"

    def test_valid(self, logs_nodes: LogContainer):
        assert len(logs_nodes) == 1
        assert "node0 was executed" in logs_nodes[0].message


class TestGraphEmptyEdges(CommonGraphProgramConfig):
    def graph_name(self) -> str:
        return "empty_edges"

    def test_valid(self, logs_nodes: LogContainer):
        assert len(logs_nodes) == 3

        # Empty edges, so order is not guaranteed
        messages = [log.message for log in logs_nodes]
        assert "node0 was executed" in messages
        assert "node1 was executed" in messages
        assert "node2 was executed" in messages


class TestGraphMultipleEdges(CommonGraphProgramConfig):
    _visualization = r"""
             [    n0   ]
            /   /  |   |
           v   /   v   |
        [n1]  /  [n2]  |
          |  |   / |   |
          v  v  v  |   |
          [ n3 ]   |   |
               \   |   /
                v  v  v
                [ n4 ]

    After topological sorting:
    |node 0 { indegree: 0, edges: [1, 2, 3, 4] }
    |node 1 { indegree: 1, edges: [3] }
    |node 2 { indegree: 1, edges: [3, 4] }
    |node 3 { indegree: 3, edges: [4] }
    |node 4 { indegree: 3, edges: [] }
    """

    def graph_name(self) -> str:
        return "multiple_edges"

    def test_valid(self, logs_nodes: LogContainer):
        assert len(logs_nodes) == 5
        n0 = logs_nodes.find_log(field="message", pattern="node0 was executed")
        n1 = logs_nodes.find_log(field="message", pattern="node1 was executed")
        n2 = logs_nodes.find_log(field="message", pattern="node2 was executed")
        n3 = logs_nodes.find_log(field="message", pattern="node3 was executed")
        n4 = logs_nodes.find_log(field="message", pattern="node4 was executed")

        # Node0 must be first
        assert n0.timestamp == min(log.timestamp for log in logs_nodes), "Node0 is not the first executed node"

        # Node0, edges: 0->1, 0->2, 0->3, 0->4
        assert n0.timestamp < n1.timestamp, self._visualization
        assert n0.timestamp < n2.timestamp, self._visualization
        assert n0.timestamp < n3.timestamp, self._visualization
        assert n0.timestamp < n4.timestamp, self._visualization

        # Node1, edges: 1->3
        assert n1.timestamp < n3.timestamp, self._visualization

        # Node2, edges: 2->3, 2->4
        assert n2.timestamp < n3.timestamp, self._visualization
        assert n2.timestamp < n4.timestamp, self._visualization

        # Node3, edges: 3->4
        assert n3.timestamp < n4.timestamp, self._visualization

        assert n4.timestamp == max(log.timestamp for log in logs_nodes), "Node4 is not the last executed node"


class TestGraphCube(CommonGraphProgramConfig):
    _visualization = r"""
          [n5]------->[n7]
         ^^           ^  ^
        / |          /   |
       [n4]------->[n6]  |
       ^  |         |    |
       |  [n1]------|>[n3]
       | ^          |   ^
       |/           |  /
       [n0]------->[n2]

    After topological sorting:
    |node 0 { indegree: 0, edges: [1, 2, 3] }
    |node 1 { indegree: 1, edges: [4, 5] }
    |node 2 { indegree: 1, edges: [4, 6] }
    |node 3 { indegree: 1, edges: [5, 6] }
    |node 4 { indegree: 2, edges: [7] }
    |node 5 { indegree: 2, edges: [7] }
    |node 6 { indegree: 2, edges: [7] }
    |node 7 { indegree: 3, edges: [] }
    """

    def graph_name(self) -> str:
        return "cube"

    def test_valid(self, logs_nodes: LogContainer):
        assert len(logs_nodes) == 8
        n0 = logs_nodes.find_log(field="message", pattern="node0 was executed")
        n1 = logs_nodes.find_log(field="message", pattern="node1 was executed")
        n2 = logs_nodes.find_log(field="message", pattern="node2 was executed")
        n3 = logs_nodes.find_log(field="message", pattern="node3 was executed")
        n4 = logs_nodes.find_log(field="message", pattern="node4 was executed")
        n5 = logs_nodes.find_log(field="message", pattern="node5 was executed")
        n6 = logs_nodes.find_log(field="message", pattern="node6 was executed")
        n7 = logs_nodes.find_log(field="message", pattern="node7 was executed")

        # Node0 must be first
        assert n0.timestamp == min(log.timestamp for log in logs_nodes), "Node0 is not the first executed node"

        # Node0, edges: 0->1, 0->2, 0->4
        assert n0.timestamp < n1.timestamp, self._visualization
        assert n0.timestamp < n2.timestamp, self._visualization
        assert n0.timestamp < n4.timestamp, self._visualization

        # Node1, edges: 1->3, 1->5
        assert n1.timestamp < n3.timestamp, self._visualization
        assert n1.timestamp < n5.timestamp, self._visualization

        # Node2, edges: 2->3, 2->6
        assert n2.timestamp < n3.timestamp, self._visualization
        assert n2.timestamp < n6.timestamp, self._visualization

        # Node3, edges: 3->7
        assert n3.timestamp < n7.timestamp, self._visualization

        # Node4, edges: 4->5, 4->6
        assert n4.timestamp < n5.timestamp, self._visualization
        assert n4.timestamp < n6.timestamp, self._visualization

        # Node5, edges: 5->7
        assert n5.timestamp < n7.timestamp, self._visualization

        # Node6, edges: 6->7
        assert n6.timestamp < n7.timestamp, self._visualization

        # Node7 must be last
        assert n7.timestamp == max(log.timestamp for log in logs_nodes), "Node7 is not the last executed node"


class TestGraphParallelFlows(CommonGraphProgramConfig):
    _visualization = r"""
          [n0]------->[n1]------->[n2]

          [n3]------->[n4]------->[n5]


    After topological sorting:
    |node 0 { indegree: 0, edges: [2] }
    |node 1 { indegree: 0, edges: [3] }
    |node 2 { indegree: 1, edges: [4] }
    |node 3 { indegree: 1, edges: [5] }
    |node 4 { indegree: 1, edges: [] }
    |node 5 { indegree: 1, edges: [] }
    """

    def graph_name(self) -> str:
        return "parallel_flows"

    def test_valid(self, logs_nodes: LogContainer):
        assert len(logs_nodes) == 6
        n0 = logs_nodes.find_log(field="message", pattern="node0 was executed")
        n1 = logs_nodes.find_log(field="message", pattern="node1 was executed")
        n2 = logs_nodes.find_log(field="message", pattern="node2 was executed")
        n3 = logs_nodes.find_log(field="message", pattern="node3 was executed")
        n4 = logs_nodes.find_log(field="message", pattern="node4 was executed")
        n5 = logs_nodes.find_log(field="message", pattern="node5 was executed")

        # Nodes: 0,1,2 are independent of Nodes: 3,4,5
        assert n0.timestamp < n1.timestamp < n2.timestamp, self._visualization
        assert n3.timestamp < n4.timestamp < n5.timestamp, self._visualization


# region Negative Scenarios


class CommonInvalidGraphConfig(CitScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.graphs.graph_program"

    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {
                "task_queue_size": 256,
                "workers": 4,
            },
            "test": {"graph_name": self.graph_name()},
        }

    @abstractmethod
    def graph_name(self) -> str: ...

    def capture_stderr(self) -> bool:
        return True

    def expect_command_failure(self) -> bool:
        return True


class TestGraphInvalidLoop(CommonInvalidGraphConfig):
    def graph_name(self) -> str:
        return "loop"

    def test_invalid(self, results: ScenarioResult) -> None:
        assert results.return_code == ResultCode.PANIC
        assert results.stderr is not None
        assert "Graph contains a cycle, which is not allowed" in results.stderr


class TestGraphInvalidSelfLoop(CommonInvalidGraphConfig):
    def graph_name(self) -> str:
        return "self_loop"

    def test_invalid(self, results: ScenarioResult) -> None:
        assert results.return_code == ResultCode.PANIC
        assert results.stderr is not None
        assert "Self-loop edges are not allowed" in results.stderr


class TestGraphInvalidNotEnoughNodes(CommonInvalidGraphConfig):
    def graph_name(self) -> str:
        return "not_enough_nodes"

    def test_invalid(self, results: ScenarioResult) -> None:
        assert results.return_code == ResultCode.PANIC
        assert results.stderr is not None
        assert "Graph requires at least two nodes to add edges" in results.stderr


class TestGraphInvalidNode(CommonInvalidGraphConfig):
    def graph_name(self) -> str:
        return "invalid_node"

    def test_invalid(self, results: ScenarioResult) -> None:
        assert results.return_code == ResultCode.PANIC
        assert results.stderr is not None
        assert "Invalid node ID" in results.stderr


class TestGraphInvalidEdge(CommonInvalidGraphConfig):
    def graph_name(self) -> str:
        return "invalid_edge"

    def test_invalid(self, results: ScenarioResult) -> None:
        assert results.return_code == ResultCode.PANIC
        assert results.stderr is not None
        assert "Invalid edge ID" in results.stderr


class TestGraphInvalidDuplicatedEdge(CommonInvalidGraphConfig):
    def graph_name(self) -> str:
        return "duplicated_edge"

    def test_invalid(self, results: ScenarioResult) -> None:
        assert results.return_code == ResultCode.PANIC
        assert results.stderr is not None
        assert "Duplicate edges are not allowed" in results.stderr


# endregion


# region Integration Scenarios
class TestGraphInSequence(CommonGraphProgramConfig):
    def graph_name(self) -> str:
        return "two_steps"

    def test_valid(self, logs_nodes: LogContainer):
        n0 = logs_nodes.find_log(field="message", pattern="node0 was executed")
        n1 = logs_nodes.find_log(field="message", pattern="node1 was executed")
        n2 = logs_nodes.find_log(field="message", pattern="node2 was executed")
        n3 = logs_nodes.find_log(field="message", pattern="node3 was executed")

        assert n0.timestamp < n1.timestamp < n2.timestamp < n3.timestamp, (
            "Nodes were not executed in the expected order"
        )


class TestGraphInConcurrency(CommonGraphProgramConfig):
    def graph_name(self) -> str:
        return "concurrency"

    def test_valid(self, logs_nodes: LogContainer):
        n0 = logs_nodes.find_log(field="message", pattern="node0 was executed")
        n1 = logs_nodes.find_log(field="message", pattern="node1 was executed")
        n2 = logs_nodes.find_log(field="message", pattern="node2 was executed")
        n3 = logs_nodes.find_log(field="message", pattern="node3 was executed")
        n4 = logs_nodes.find_log(field="message", pattern="node4 was executed")

        assert n0.timestamp < n1.timestamp, "First graph has incorrect execution order"
        assert n2.timestamp < n3.timestamp < n4.timestamp, "Second graph has incorrect execution order"


class TestGraphInSeparatePrograms(CommonGraphProgramConfig):
    def graph_name(self) -> str:
        return "two_programs"

    @pytest.mark.xfail(reason="https://github.com/qorix-group/inc_orchestrator_internal/issues/382")
    def test_valid(self, logs_nodes: LogContainer):
        n0 = logs_nodes.find_log(field="message", pattern="node0 was executed")
        n1 = logs_nodes.find_log(field="message", pattern="node1 was executed")
        n2 = logs_nodes.find_log(field="message", pattern="node2 was executed")
        n3 = logs_nodes.find_log(field="message", pattern="node3 was executed")
        n4 = logs_nodes.find_log(field="message", pattern="node4 was executed")

        assert n0.timestamp < n1.timestamp, "First graph has incorrect execution order"
        assert n2.timestamp < n3.timestamp < n4.timestamp, "Second graph has incorrect execution order"
        # Programs are executed one after another
        assert n1.timestamp < n2.timestamp, "Programs were not executed sequentially"


class TestGraphDedicatedWorker(CitScenario):
    @pytest.fixture(scope="class")
    def test_config(self) -> dict[str, Any]:
        return {
            "runtime": {
                "task_queue_size": 256,
                "workers": 4,
                "dedicated_workers": [{"id": "dedicated_worker_0"}],
            }
        }

    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "orchestration.graphs.dedicated_graph"

    def test_dedicated_execution(self, logs_info_level: LogContainer):
        dedicated_tasks = logs_info_level.get_logs(field="message", pattern=r"sync\d*")
        dedicated_threads = {log.thread_id for log in dedicated_tasks}
        assert len(dedicated_threads) == 1, "Expected execution only on 1 dedicated worker"
