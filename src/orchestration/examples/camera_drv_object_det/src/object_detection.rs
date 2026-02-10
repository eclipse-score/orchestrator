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

use orchestration_macros::import_from_cpp;

#[import_from_cpp("pre_processing", "drive_q1", "drive_q2", "drive_q3", "object_fusion")]
pub struct ObjectDetection;

// For logging from C++ methods
use kyron_foundation::prelude::*;
use std::ffi::CStr;
use std::os::raw::c_char;

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn rust_log_info(msg: *const c_char) {
    if msg.is_null() {
        return;
    }
    let c_str = unsafe { CStr::from_ptr(msg) };
    if let Ok(rust_str) = c_str.to_str() {
        info!("{}", rust_str);
    }
}
