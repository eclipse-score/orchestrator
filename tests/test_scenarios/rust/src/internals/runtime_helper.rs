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
use kyron::core::types::UniqueWorkerId;
use kyron::prelude::ThreadParameters as AsyncRtThreadParameters;
use kyron::runtime::*;
use kyron::scheduler::SchedulerType;
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;
use tracing::debug;

fn deserialize_scheduler_type<'de, D>(deserializer: D) -> Result<Option<SchedulerType>, D::Error>
where
    D: Deserializer<'de>,
{
    let value_opt: Option<String> = Option::deserialize(deserializer)?;
    if let Some(value_str) = value_opt {
        let value = match value_str.as_str() {
            "fifo" => SchedulerType::Fifo,
            "round_robin" => SchedulerType::RoundRobin,
            "other" => SchedulerType::Other,
            _ => return Err(de::Error::custom("Unknown scheduler type")),
        };
        return Ok(Some(value));
    }

    Ok(None)
}

/// Thread parameters.
#[derive(Deserialize, Debug, Clone)]
pub struct ThreadParameters {
    pub thread_priority: Option<u8>,
    pub thread_affinity: Option<Vec<usize>>,
    pub thread_stack_size: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_scheduler_type")]
    pub thread_scheduler: Option<SchedulerType>,
}

/// Dedicated worker configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct DedicatedWorkerConfig {
    pub id: String,
    #[serde(flatten)]
    pub thread_parameters: ThreadParameters,
}

/// Execution engine configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct ExecEngineConfig {
    pub task_queue_size: u32,
    pub workers: usize,
    #[serde(flatten)]
    pub thread_parameters: ThreadParameters,
    pub dedicated_workers: Option<Vec<DedicatedWorkerConfig>>,
    pub safety_worker: Option<ThreadParameters>,
}

fn deserialize_exec_engines<'de, D>(deserializer: D) -> Result<Vec<ExecEngineConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RuntimeConfig {
        Object(ExecEngineConfig),
        Array(Vec<ExecEngineConfig>),
    }

    let exec_engines = match RuntimeConfig::deserialize(deserializer)? {
        RuntimeConfig::Object(exec_engine_config) => vec![exec_engine_config],
        RuntimeConfig::Array(exec_engine_configs) => exec_engine_configs,
    };
    Ok(exec_engines)
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct Runtime {
    #[serde(deserialize_with = "deserialize_exec_engines")]
    exec_engines: Vec<ExecEngineConfig>,
}

impl Runtime {
    /// Parse `Runtime` from JSON string.
    /// JSON is expected to contain `runtime` field.
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let v: Value = serde_json::from_str(json_str).map_err(|e| e.to_string())?;
        serde_json::from_value(v["runtime"].clone()).map_err(|e| e.to_string())
    }

    pub fn build(&self) -> kyron::runtime::Runtime {
        debug!(
            "Creating kyron::Runtime with {} execution engines",
            self.exec_engines.len()
        );

        let mut async_rt_builder = kyron::runtime::RuntimeBuilder::new();
        for exec_engine in self.exec_engines.as_slice() {
            debug!("Creating ExecutionEngine with: {:?}", exec_engine);

            let mut exec_engine_builder = ExecutionEngineBuilder::new()
                .task_queue_size(exec_engine.task_queue_size)
                .workers(exec_engine.workers);

            // Set thread parameters.
            {
                let thread_params = &exec_engine.thread_parameters;
                if let Some(thread_priority) = thread_params.thread_priority {
                    exec_engine_builder = exec_engine_builder.thread_priority(thread_priority);
                }
                if let Some(thread_affinity) = &thread_params.thread_affinity {
                    exec_engine_builder = exec_engine_builder.thread_affinity(thread_affinity);
                }
                if let Some(thread_stack_size) = thread_params.thread_stack_size {
                    exec_engine_builder = exec_engine_builder.thread_stack_size(thread_stack_size);
                }
                if let Some(thread_scheduler) = thread_params.thread_scheduler {
                    exec_engine_builder = exec_engine_builder.thread_scheduler(thread_scheduler);
                }
            }

            // Set dedicated workers.
            if let Some(dedicated_workers) = &exec_engine.dedicated_workers {
                for dedicated_worker in dedicated_workers {
                    // Create thread parameters object.
                    let async_rt_thread_params = self.set_thread_params(&dedicated_worker.thread_parameters);

                    // Create `UniqueWorkerId`.
                    let unique_worker_id = UniqueWorkerId::from(&dedicated_worker.id);

                    exec_engine_builder =
                        exec_engine_builder.with_dedicated_worker(unique_worker_id, async_rt_thread_params);
                }
            }

            // Set safety worker.
            if let Some(safety_worker) = &exec_engine.safety_worker {
                let safety_worker_thread_params = self.set_thread_params(safety_worker);
                exec_engine_builder = exec_engine_builder.enable_safety_worker(safety_worker_thread_params);
            }

            let (builder, _) = async_rt_builder.with_engine(exec_engine_builder);
            async_rt_builder = builder;
        }

        async_rt_builder.build().expect("Failed to build async runtime")
    }

    fn set_thread_params(&self, src_thread_params: &ThreadParameters) -> AsyncRtThreadParameters {
        let mut dst_thread_params = AsyncRtThreadParameters::default();
        if let Some(thread_priority) = src_thread_params.thread_priority {
            dst_thread_params = dst_thread_params.priority(thread_priority);
        }
        if let Some(thread_affinity) = &src_thread_params.thread_affinity {
            dst_thread_params = dst_thread_params.affinity(thread_affinity);
        }
        if let Some(thread_stack_size) = src_thread_params.thread_stack_size {
            dst_thread_params = dst_thread_params.stack_size(thread_stack_size);
        }
        if let Some(thread_scheduler) = src_thread_params.thread_scheduler {
            dst_thread_params = dst_thread_params.scheduler_type(thread_scheduler);
        }

        dst_thread_params
    }
}
