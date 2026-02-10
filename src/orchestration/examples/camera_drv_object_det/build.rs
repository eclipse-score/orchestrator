// *******************************************************************************
// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0
// *******************************************************************************

fn main() {
    let orchestration_macro_include = "../../../orchestration_macros/cpp/include";
    println!("cargo::rerun-if-changed=cpp/include/object_detection.h");
    println!("cargo::rerun-if-changed=cpp/object_detection.cpp");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", orchestration_macro_include);

    cc::Build::new()
        .cpp(true)
        .file("cpp/object_detection.cpp")
        .include(orchestration_macro_include)
        .compile("libobject_detection");
}
