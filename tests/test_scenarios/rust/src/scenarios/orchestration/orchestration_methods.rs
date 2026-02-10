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
use super::*;
use crate::internals::runtime_helper::Runtime;
use kyron_foundation::prelude::*;
use orchestration::{
    api::{design::Design, Orchestration},
    common::{
        tag::{AsTagTrait, Tag},
        DesignConfig, ProgramDatabaseParams,
    },
};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use test_scenarios_rust::scenario::Scenario;

#[derive(Clone, Copy)]
struct CustomTag(Tag);

impl AsTagTrait for CustomTag {
    fn as_tag(&self) -> &Tag {
        &self.0
    }
}

#[derive(Clone)]
pub struct SampleStruct {}

impl SampleStruct {
    pub fn new() -> Self {
        Self {}
    }

    pub fn sample_method(&mut self) -> InvokeResult {
        info!(id = "sample_method");
        Ok(())
    }

    pub async fn sample_async_method(&mut self) -> InvokeResult {
        info!(id = "sample_async_method", location = "begin");
        sleep::sleep(::core::time::Duration::from_millis(100)).await;
        info!(id = "sample_async_method", location = "end");
        Ok(())
    }
}

#[derive(Deserialize)]
struct Program {
    program_name: String,
}
#[derive(Deserialize)]
struct Capacity {
    registration_capacity: usize,
}

pub struct TagMethods;

fn test_design() -> Result<Design, CommonErrors> {
    // Prepare design
    let mut design = Design::new("test_design".into(), DesignConfig::default());
    info!(message = "Design created", id = design.id().tracing_str());

    // Create tags
    let method_tag = Tag::from_str_static("sample_method");
    info!(
        message = "Tag created",
        id = method_tag.id(),
        tracing_str = method_tag.tracing_str()
    );
    let method_tag_async = Tag::from_str_static("sample_async_method");
    let extra_tag = Tag::from_str_static("extra_tag");
    let tags_collection = [CustomTag(method_tag), CustomTag(method_tag_async)];

    // Check if tags are in tags_collection
    let result = method_tag.is_in_collection(tags_collection.iter().cloned());
    info!(tag = method_tag.tracing_str(), is_in_collection = result);
    let result = extra_tag.is_in_collection(tags_collection.iter().cloned());
    info!(tag = extra_tag.tracing_str(), is_in_collection = result);

    // Check if tags are found in tags_collection
    let result = method_tag.find_in_collection(tags_collection.iter().cloned());
    info!(tag = method_tag.tracing_str(), find_in_collection = result.is_some());

    let result = extra_tag.find_in_collection(tags_collection.iter().cloned());
    info!(tag = extra_tag.tracing_str(), find_in_collection = result.is_some());

    let sample = Arc::new(Mutex::new(SampleStruct::new()));
    let _ = design.register_invoke_method(method_tag, sample.clone(), SampleStruct::sample_method)?;
    let _ = design.register_invoke_method_async(method_tag_async, sample.clone(), |arc_mutex_sample| {
        // Lock the mutex and extract the data before entering the async block
        let guard = arc_mutex_sample.lock().expect("Failed to lock mutex");
        // If SampleStruct implements Clone, you can clone it here. Otherwise, extract the needed data.
        let mut sample_struct = (*guard).clone();
        Box::pin(async move { sample_struct.sample_async_method().await })
    })?;

    let orch_tag = design.get_orchestration_tag(method_tag)?;
    let orch_tag_async = design.get_orchestration_tag(method_tag_async)?;

    design.add_program("test_program", move |design, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(Invoke::from_tag(&orch_tag_async, design.config()))
                .with_step(Invoke::from_tag(&orch_tag, design.config()))
                .build(),
        );
        Ok(())
    });
    Ok(design)
}

impl Scenario for TagMethods {
    fn name(&self) -> &str {
        "tag_methods"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();

        let orch = Orchestration::new()
            .add_design(test_design().expect("Failed to create design"))
            .design_done();

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut program = program_manager
            .get_program("test_program")
            .expect("Failed to get program");

        rt.block_on(async move {
            let h1 = kyron::spawn(async move {
                let _ = program.run_n(1).await;
            });
            let _ = h1.await;
        });

        Ok(())
    }
}

pub struct InvalidInvokes;

fn register_same_method_twice() -> Result<Design, CommonErrors> {
    // Prepare design
    let design = Design::new("register_same_method_twice".into(), DesignConfig::default());
    info!(message = "Design created", id = design.id().tracing_str());

    // Create tags
    let method_tag1 = Tag::from_str_static("sample_async_method");
    let method_tag2 = Tag::from_str_static("sample_async_method");

    let sample = Arc::new(Mutex::new(SampleStruct::new()));
    let _ = design.register_invoke_method(method_tag1, sample.clone(), SampleStruct::sample_method)?;
    let _ = design.register_invoke_method(method_tag2, sample.clone(), SampleStruct::sample_method)?;

    Ok(design)
}

fn register_same_async_method_twice() -> Result<Design, CommonErrors> {
    // Prepare design
    let design = Design::new("register_same_async_method_twice".into(), DesignConfig::default());
    info!(message = "Design created", id = design.id().tracing_str());

    // Create tags
    let method_tag1 = Tag::from_str_static("sample_method");
    let method_tag2 = Tag::from_str_static("sample_method");

    let sample = Arc::new(Mutex::new(SampleStruct::new()));

    let _ = design.register_invoke_method_async(method_tag1, sample.clone(), |arc_mutex_sample| {
        // Lock the mutex and extract the data before entering the async block
        let guard = arc_mutex_sample.lock().expect("Failed to lock mutex");
        // If SampleStruct implements Clone, you can clone it here. Otherwise, extract the needed data.
        let mut sample_struct = (*guard).clone();
        Box::pin(async move { sample_struct.sample_async_method().await })
    })?;
    let _ = design.register_invoke_method_async(method_tag2, sample.clone(), |arc_mutex_sample| {
        let guard = arc_mutex_sample.lock().expect("Failed to lock mutex");
        let mut sample_struct = (*guard).clone();
        Box::pin(async move { sample_struct.sample_async_method().await })
    })?;

    Ok(design)
}

fn get_invalid_tag() -> Result<Design, CommonErrors> {
    // Prepare design
    let design = Design::new("get_invalid_tag".into(), DesignConfig::default());
    info!(message = "Design created", id = design.id().tracing_str());

    // Create tag and register
    let method_tag = Tag::from_str_static("sample_async_method");

    let sample = Arc::new(Mutex::new(SampleStruct::new()));
    let _ = design.register_invoke_method(method_tag, sample.clone(), SampleStruct::sample_method)?;

    let _non_existing_tag = design.get_orchestration_tag("NonExistingTag".into())?;

    Ok(design)
}

impl Scenario for InvalidInvokes {
    fn name(&self) -> &str {
        "error_scenarios"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let logic: Program = serde_json::from_str(input).expect("Failed to parse input");

        // Add design based on logic.program_name
        let selected_design = match logic.program_name.as_str() {
            "register_same_method_twice" => register_same_method_twice,
            "register_same_async_method_twice" => register_same_async_method_twice,
            "get_invalid_tag" => get_invalid_tag,
            _ => panic!("Unknown program name"),
        };

        let _ = Orchestration::new()
            .add_design(selected_design().expect("Failed to create design"))
            .design_done();

        Ok(())
    }
}

fn too_many_tags(capacity: usize) -> Result<Design, CommonErrors> {
    // Prepare design
    let design = Design::new(
        "too_many_tags".into(),
        DesignConfig {
            db_params: ProgramDatabaseParams {
                registration_capacity: capacity,
            },
            ..Default::default()
        },
    );
    info!(message = "Design created", id = design.id().tracing_str());

    // Create more tags than registration capacity
    let sample = Arc::new(Mutex::new(SampleStruct::new()));
    for i in 0..=design.config().db_params.registration_capacity {
        let tag_str = format!("tag_{}", i);
        info!(message = "Registering method", tag = tag_str.as_str());
        let tag = Tag::from_str_static(Box::leak(tag_str.into_boxed_str()));
        let _ = design.register_invoke_method(tag, sample.clone(), SampleStruct::sample_method)?;
    }

    Ok(design)
}

pub struct TooManyTags;

impl Scenario for TooManyTags {
    fn name(&self) -> &str {
        "too_many_tags"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let logic: Capacity = serde_json::from_str(input).expect("Failed to parse input");

        let _ = Orchestration::new()
            .add_design(too_many_tags(logic.registration_capacity).expect("Failed to create design"))
            .design_done();

        Ok(())
    }
}
