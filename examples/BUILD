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

_BASE_DEPS = [
    "@score_kyron//src/kyron-foundation:libkyron_foundation",
    "@score_crates//:iceoryx2_qnx8",
    "@score_crates//:iceoryx2_bb_container_qnx8",
    "@score_crates//:libc",
]

_COMMON_DEPS = _BASE_DEPS + [
    "@score_kyron//src/kyron:libkyron",
]

_EXAMPLE_DEPS = _COMMON_DEPS + [
    "//src/orchestration:liborchestration",
    "@score_kyron//src/logging_tracing:liblogging_tracing",
]

_COMMON_SRCS = glob([
    "common/**/*.rs",
])


rust_binary(
    name = "basic",
    srcs = [
        "basic.rs",
    ] + _COMMON_SRCS,
    visibility = ["//visibility:public"],
    deps = _EXAMPLE_DEPS,
)

rust_binary(
    name = "main_macro_basic",
    srcs = [
        "main_macro_basic.rs",
    ] + _COMMON_SRCS,
    proc_macro_deps = [
        "@score_kyron//src/kyron-macros:runtime_macros",
    ],
    visibility = ["//visibility:public"],
    deps = _EXAMPLE_DEPS,
)

rust_binary(
    name = "catch_error",
    srcs = [
        "catch_error.rs",
    ] + _COMMON_SRCS,
    visibility = ["//visibility:public"],
    deps = _EXAMPLE_DEPS,
)

rust_binary(
    name = "events_across_local_programs",
    srcs = [
        "events_across_local_programs.rs",
    ] + _COMMON_SRCS,
    visibility = ["//visibility:public"],
    deps = _EXAMPLE_DEPS,
)

rust_binary(
    name = "inter_process_event_sender",
    srcs = [
        "inter_process_event/sender.rs",
    ] + _COMMON_SRCS,
    crate_root = "inter_process_event/sender.rs",
    visibility = ["//visibility:public"],
    deps = _EXAMPLE_DEPS,
)

rust_binary(
    name = "inter_process_event_receiver",
    srcs = [
        "inter_process_event/receiver.rs",
    ] + _COMMON_SRCS,
    crate_root = "inter_process_event/receiver.rs",
    visibility = ["//visibility:public"],
    deps = _EXAMPLE_DEPS,
)
