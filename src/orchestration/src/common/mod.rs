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

pub mod orch_tag;
pub mod tag;

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct ProgramDatabaseParams {
    pub registration_capacity: usize,
}

impl Default for ProgramDatabaseParams {
    fn default() -> Self {
        Self {
            registration_capacity: 256,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct DesignConfig {
    pub db_params: ProgramDatabaseParams,
    pub max_concurrent_action_executions: usize,
}

impl Default for DesignConfig {
    fn default() -> Self {
        DesignConfig {
            db_params: ProgramDatabaseParams::default(),
            max_concurrent_action_executions: 2,
        }
    }
}
