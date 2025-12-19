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
load("@rules_rust//rust:defs.bzl", "rust_binary")

rust_binary(
    name = "test_scenarios",
    srcs = glob(["src/**/*.rs"]),
    tags = [
        "manual",
    ],
    visibility = ["//tests/test_cases:__pkg__"],
    deps = [
        "//src/orchestration:liborchestration",
        "@score_crates//:futures",
        "@score_crates//:libc",
        "@score_crates//:serde",
        "@score_crates//:serde_json",
        "@score_crates//:tracing",
        "@score_crates//:tracing_subscriber",
        "@score_kyron//src/kyron:libkyron",
        "@score_kyron//src/kyron-foundation:libkyron_foundation",
        "@score_test_scenarios//test_scenarios_rust",
    ],
)
