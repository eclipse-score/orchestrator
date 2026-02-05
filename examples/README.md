# Orchestrator Examples

This directory contains example programs demonstrating various features of the Orchestration framework.

## Running Examples

All examples can be run using Bazel:

```bash
bazel run //examples:basic
bazel run //examples:main_macro_basic
bazel run //examples:catch_error
bazel run //examples:dag
bazel run //examples:events_across_local_programs
bazel run //examples:shutdown
bazel run //examples/camera_drv_object_det:camera_drv_object_det
```

## Available Examples

### basic.rs

A foundational example demonstrating the core concepts of the orchestration framework.

This example shows:

- Creating an orchestration design with events and actions
- Registering common functions and events into a design
- Building a program with sequential and concurrent action execution
- Using `SequenceBuilder` to chain actions in order
- Using `ConcurrencyBuilder` to execute actions in parallel
- Synchronizing on timer events
- Binding actions to dedicated workers
- Running a program with cyclic timer events

**Key concepts:** Design creation, program registration, sequential vs concurrent execution, worker binding, timer events

### main_macro_basic.rs

Demonstrates using the `#[kyron::main]` macro for simplified Kyron runtime initialization when working with orchestration.

This example illustrates:

- Using the main macro to reduce boilerplate setup code
- Quick runtime initialization with default parameters
- Integration between Kyron runtime and orchestration framework
- Simplified program execution flow

**Key concepts:** `#[kyron::main]` macro, simplified setup, runtime integration

### catch_error.rs

Shows comprehensive error handling capabilities in orchestration programs.

This example demonstrates:

- Using `CatchBuilder` to handle errors in action sequences
- Filtering errors by type with `ErrorFilter::UserErrors`
- Implementing non-recoverable error handlers that stop execution
- Implementing recoverable error handlers that allow continuation
- Error handling in concurrent action branches
- Graceful error propagation and logging

**Key concepts:** Error handling, `CatchBuilder`, recoverable vs non-recoverable errors, error filtering

### dag.rs

Demonstrates Directed Acyclic Graph (DAG) execution patterns for complex task dependencies.

This example shows:

- Creating a DAG with 10 nodes representing a complex dependency graph
- Using `LocalGraphActionBuilder` to define nodes and edges
- Adding action nodes in any order (automatic topological sorting)
- Defining dependencies between nodes with edges
- Parallel execution of independent branches in the graph
- Automatic scheduling based on node dependencies

The example graph structure:

```
N1 ------> N3 --------> N6
 \                       \
  \------> N4 ---> N7 --> N9 ----> N10
  /                      /
 /                      /
N2 ------> N5 --------> N8
```

**Key concepts:** DAG execution, task dependencies, topological sorting, parallel graph traversal

### select_action.rs

Demonstrates conditional action selection based on runtime conditions.

This example shows:

- Implementing conditional branching in orchestration programs
- Selecting different actions based on program state
- Dynamic program flow control
- Decision-making patterns in action sequences

**Key concepts:** Conditional execution, action selection, dynamic flow control

### branching.rs

Illustrates various branching patterns in orchestration programs.

This example demonstrates:

- Different branching strategies for program execution
- Parallel branch execution with synchronization points
- Managing multiple execution paths
- Coordinating concurrent program flows

**Key concepts:** Branching patterns, parallel execution, execution flow management

### events_across_local_programs.rs

Shows event handling and communication between multiple programs within the same process.

This example demonstrates:

- Creating multiple orchestration designs in a single application
- Using `TriggerBuilder` to send events from one program
- Using `SyncBuilder` to wait for events in another program
- Inter-program communication through local events
- Coordinating multiple programs in a single runtime

**Key concepts:** Inter-program events, local event communication, program coordination, `TriggerBuilder`, `SyncBuilder`

### send_ipc_event.rs

Demonstrates sending inter-process communication (IPC) events using iceoryx2.

This example shows:

- Publishing events to external processes
- Integration with iceoryx2 for IPC
- Event-based inter-process communication
- Cross-process orchestration patterns

**Key concepts:** IPC events, iceoryx2 integration, cross-process communication

### inter_process_event_sender.rs & inter_process_event_receiver.rs

A complete IPC example with separate sender and receiver programs.

These examples demonstrate:

- Setting up an event sender program that publishes events
- Setting up an event receiver program that listens for events
- Coordinating two separate processes through IPC events
- Real-world inter-process orchestration patterns

Run both programs simultaneously:

```bash
# Terminal 1 - Start the receiver
bazel run //examples:inter_process_event_receiver

# Terminal 2 - Start the sender
bazel run //examples:inter_process_event_sender
```

**Key concepts:** IPC sender/receiver pattern, process coordination, event-driven IPC

### shutdown.rs

Demonstrates proper program lifecycle management with graceful shutdown.

This example shows:

- Defining start and stop actions for a program
- Using `with_start_action()` for initialization logic
- Using `with_stop_action()` with timeout for cleanup
- Implementing shutdown events for controlled termination
- Handling programs that run indefinitely until shutdown
- Graceful resource cleanup on exit

**Key concepts:** Program lifecycle, start/stop actions, shutdown events, graceful termination

### shutdown_using_signals.rs

Shows how to handle system signals for graceful shutdown.

This example demonstrates:

- Integrating system signal handlers (SIGINT, SIGTERM)
- Triggering orchestration shutdown from signal handlers
- Clean program termination on Ctrl+C or kill signals
- Signal-driven lifecycle management

**Key concepts:** Signal handling, system integration, signal-driven shutdown

### camera_drv_object_det (Multi-Design Example)

A comprehensive real-world example demonstrating a camera driver with object detection pipeline using multiple coordinated orchestration designs and C++/Rust interoperability.

This example shows:

- **Multi-design orchestration** - Three separate designs working together (timer, camera driver, object detection)
- **Cross-design event coordination** - Designs triggering events consumed by other designs
- **C++/Rust interoperability** - Using the `#[import_from_cpp]` macro to call C++ methods from orchestration
- **Method-based actions** - Registering struct methods as invoke actions with `register_invoke_method`
- **Complex execution pipeline** - Sequential camera processing followed by parallel object detection
- **Shared state management** - Using `Arc<Mutex<T>>` to share stateful components across actions

The execution flow:

1. **Timer Design** - Periodically triggers the timer event
2. **Camera Driver Design** - Waits for timer event, then:
   - Reads input from camera
   - Processes the image
   - Writes output
   - Triggers object detection event
3. **Object Detection Design** - Waits for trigger event, then:
   - Pre-processes the data
   - Runs three parallel detection queues (Q1, Q2, Q3) implemented in C++
   - Fuses the results from all queues

The C++ integration uses the `EXPOSE_OBJECT_TO_ORCHESTRATION` macro to export C++ class methods to the orchestration framework, demonstrating seamless interoperability between languages.

Run the complete pipeline:

```bash
bazel run //examples/camera_drv_object_det:camera_drv_object_det
```

**Key concepts:** Multi-design orchestration, cross-design events, C++/Rust FFI, `#[import_from_cpp]` macro, method-based actions, stateful components, parallel pipelines

## Orchestration Design Pattern

Most examples follow this common pattern:

1. **Design Creation** - Create a `Design` with configuration and register events/actions

   ```rust
   let mut design = Design::new("ExampleDesign".into(), DesignConfig::default());
   design.register_event("my_event".into())?;
   ```

2. **Program Registration** - Add programs with their execution logic using builders

   ```rust
   design.add_program("MyProgram", |design_instance, builder| {
       builder.with_run_action(/* action sequence */);
       Ok(())
   });
   ```

3. **Orchestration Setup** - Create orchestration and configure deployment

   ```rust
   let mut orch = Orchestration::new()
       .add_design(design)
       .design_done();
   let mut deployment = orch.get_deployment_mut();
   deployment.bind_events_as_timer(&["my_event".into()], Duration::from_secs(1))?;
   ```

4. **Runtime Integration** - Create Kyron runtime and execute programs

   ```rust
   let (builder, _) = RuntimeBuilder::new().with_engine(/* config */);
   let mut runtime = builder.build()?;
   runtime.block_on(async move { program.run().await });
   ```

Common utilities are provided in the `common/` directory, including reusable event registrations, shared function implementations, and helper utilities.

## Logging

All examples use the `logging_tracing` crate for structured logging. The log output shows:

- Execution flow through action sequences
- Worker assignment and task scheduling
- Event triggering and synchronization
- Error handling and recovery

Adjust the log level by modifying the logger configuration:

```rust
let _logger = LogAndTraceBuilder::new()
    .global_log_level(Level::INFO)  // Change to DEBUG or TRACE for more detail
    .enable_logging(true)
    .build()
    .expect("Failed to build tracing library");
```
