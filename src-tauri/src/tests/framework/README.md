# Actor Testing Framework

This directory contains a comprehensive framework for testing actors in the Evo Design application. The framework provides utilities for mocking actors, capturing events, verifying actor behavior, and running test scenarios.

## Overview

The actor testing framework consists of several components:

1. **Test Environment**: A shared environment for capturing events, registering mocks, and managing test state.
2. **Mock Actors**: Utilities for creating mock actors that can be configured to respond in specific ways.
3. **Test Harnesses**: Specialized test harnesses for different types of actors (regular, supervised, lifecycle-aware).
4. **Test Scenarios**: Utilities for defining and running test scenarios with multiple steps.
5. **Verification Utilities**: Assertion helpers and event verification utilities.

## Getting Started

To use the actor testing framework, you'll need to import the relevant modules and create a test harness for your actor.

```rust
use crate::tests::framework::{
    create_test_env,
    harness::{ActorTestHarness, create_actor_test_harness},
    mock::{MockActor, create_mock_actor_with_env, MockExt},
    scenario::{TestScenario, ActorScenarioBuilder, create_actor_scenario},
    verification::{Assert, EventVerifier},
};
```

## Basic Actor Testing

Here's a simple example of testing an actor:

```rust
#[tokio::test]
async fn test_my_actor() -> Result<()> {
    // Create a test harness
    let harness = create_actor_test_harness::<MyActor>()
        .with_actor(MyActor::spawn(MyActor::new()))
        .with_mock("dependency");
    
    // Get the mock actor
    let mock = harness.get_mock("dependency").unwrap();
    
    // Configure the mock to respond to a specific message
    mock.mock_response::<SomeMessage, SomeResponse>(SomeResponse::new()).await?;
    
    // Send a message to the actor under test
    let result = harness.actor.as_ref().unwrap()
        .ask(&SomeMessage::new())
        .await?;
    
    // Verify the result
    Assert::new(result).equals(expected_result)?;
    
    // Verify that the mock received the expected message
    mock.verify().await?;
    
    // Clean up
    harness.cleanup().await?;
    
    Ok(())
}
```

## Testing Supervised Actors

For testing supervised actors, use the `SupervisedActorTestHarness`:

```rust
#[tokio::test]
async fn test_supervised_actor() -> Result<()> {
    // Create a supervised test harness
    let mut harness = create_supervised_test_harness::<MyActor>()
        .with_actor(MyActor::spawn(MyActor::new()))
        .with_strategy(SupervisionStrategy::Restart);
    
    // Setup supervision
    harness.setup_supervision().await?;
    
    // Test the actor
    // ...
    
    // Clean up
    harness.cleanup().await?;
    
    Ok(())
}
```

## Testing Lifecycle-Aware Actors

For testing lifecycle-aware actors, use the `LifecycleTestHarness`:

```rust
#[tokio::test]
async fn test_lifecycle_actor() -> Result<()> {
    // Create a lifecycle test harness
    let mut harness = create_lifecycle_test_harness::<MyActor>()
        .with_actor(MyActor::spawn(MyActor::new()))
        .with_health_check_interval(Duration::from_millis(100));
    
    // Setup lifecycle management
    harness.setup_lifecycle().await?;
    
    // Test the actor
    // ...
    
    // Clean up
    harness.cleanup().await?;
    
    Ok(())
}
```

## Using Test Scenarios

Test scenarios provide a way to define a sequence of steps for testing actors:

```rust
#[tokio::test]
async fn test_actor_scenario() -> Result<()> {
    // Create an actor
    let actor = MyActor::spawn(MyActor::new());
    
    // Create a scenario
    let scenario = create_actor_scenario("My scenario", actor.clone())
        .add_step("Initialize", |harness| {
            Box::pin(async move {
                // Initialize the actor
                harness.actor.as_ref().unwrap()
                    .ask(&Initialize::new())
                    .await?;
                Ok(())
            })
        })
        .add_step("Process data", |harness| {
            Box::pin(async move {
                // Process some data
                let result = harness.actor.as_ref().unwrap()
                    .ask(&ProcessData::new("test data"))
                    .await?;
                
                // Verify the result
                Assert::new(result).equals(expected_result)?;
                Ok(())
            })
        });
    
    // Run the scenario
    scenario.run().await?;
    
    Ok(())
}
```

## Event Verification

The framework provides utilities for verifying events captured during tests:

```rust
#[tokio::test]
async fn test_actor_events() -> Result<()> {
    // Create a test environment
    let test_env = create_test_env();
    
    // Create a test harness with the environment
    let harness = ActorTestHarness::new(test_env.clone())
        .with_actor(MyActor::spawn(MyActor::new()));
    
    // Send a message to the actor
    harness.actor.as_ref().unwrap()
        .ask(&SomeMessage::new())
        .await?;
    
    // Verify that the expected events were captured
    let verifier = EventVerifier::new(test_env);
    verifier.verify_event_count(TestEventType::MessageSent, 1)?;
    verifier.verify_event_count(TestEventType::MessageReceived, 1)?;
    
    // Clean up
    harness.cleanup().await?;
    
    Ok(())
}
```

## Advanced Features

The framework provides many advanced features for testing actors:

- **Mock Expectations**: Configure mocks to expect specific messages and verify that they were received.
- **Event Filtering**: Filter events by type, actor ID, message type, etc.
- **Timeouts**: Configure timeouts for test operations to prevent tests from hanging.
- **Custom Assertions**: Create custom assertions for verifying complex conditions.
- **Scenario Composition**: Compose multiple scenarios to create complex test cases.

For more information, see the documentation for each module in the framework.