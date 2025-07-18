//! Example tests for the actor testing framework
//!
//! This file contains example tests that demonstrate how to use the actor testing framework.
//! These tests serve as practical examples for developers to follow when writing their own tests.

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use kameo::prelude::*;
    use tokio::time::sleep;
    use tracing::{debug, error, info, warn};

    use crate::error::{AppError, Result};
    use crate::tests::framework::{
        create_test_env,
        harness::{ActorTestHarness, create_actor_test_harness},
        mock::{MockActor, create_mock_actor_with_env, MockExt},
        scenario::{TestScenario, ActorScenarioBuilder, create_actor_scenario},
        verification::{Assert, EventVerifier},
        TestEventType,
    };

    // Example actor for testing
    #[derive(Actor, Clone)]
    struct ExampleActor {
        counter: usize,
        dependency: Option<ActorRef<MockActor>>,
    }

    impl ExampleActor {
        fn new() -> Self {
            Self {
                counter: 0,
                dependency: None,
            }
        }

        fn with_dependency(mut self, dependency: ActorRef<MockActor>) -> Self {
            self.dependency = Some(dependency);
            self
        }
    }

    // Example messages
    #[derive(Debug, Clone)]
    struct Increment;

    impl Message<Increment> for ExampleActor {
        type Reply = usize;

        async fn handle(
            &mut self,
            _msg: Increment,
            _ctx: &mut Context<Self, Self::Reply>,
        ) -> Self::Reply {
            self.counter += 1;
            self.counter
        }
    }

    #[derive(Debug, Clone)]
    struct GetCounter;

    impl Message<GetCounter> for ExampleActor {
        type Reply = usize;

        async fn handle(
            &mut self,
            _msg: GetCounter,
            _ctx: &mut Context<Self, Self::Reply>,
        ) -> Self::Reply {
            self.counter
        }
    }

    #[derive(Debug, Clone)]
    struct ProcessWithDependency {
        data: String,
    }

    impl ProcessWithDependency {
        fn new(data: impl Into<String>) -> Self {
            Self {
                data: data.into(),
            }
        }
    }

    #[derive(Debug, Clone)]
    struct DependencyRequest {
        data: String,
    }

    impl DependencyRequest {
        fn new(data: impl Into<String>) -> Self {
            Self {
                data: data.into(),
            }
        }
    }

    #[derive(Debug, Clone)]
    struct DependencyResponse {
        result: String,
    }

    impl DependencyResponse {
        fn new(result: impl Into<String>) -> Self {
            Self {
                result: result.into(),
            }
        }
    }

    impl Message<ProcessWithDependency> for ExampleActor {
        type Reply = Result<String>;

        async fn handle(
            &mut self,
            msg: ProcessWithDependency,
            _ctx: &mut Context<Self, Self::Reply>,
        ) -> Self::Reply {
            if let Some(ref dependency) = self.dependency {
                let response = dependency
                    .ask(&DependencyRequest::new(msg.data))
                    .await
                    .map_err(|e| AppError::ActorError(format!("Dependency error: {}", e)))?;
                
                Ok(response.result)
            } else {
                Err(AppError::InvalidStateError("Dependency not set".to_string()))
            }
        }
    }

    // Basic test example
    #[tokio::test]
    async fn test_example_actor_basic() -> Result<()> {
        // Create a test harness
        let harness = create_actor_test_harness::<ExampleActor>()
            .with_actor(ExampleActor::spawn(ExampleActor::new()));
        
        // Get the actor
        let actor = harness.actor.as_ref().unwrap();
        
        // Send a message to the actor
        let result = actor.ask(&Increment).await?;
        
        // Verify the result
        Assert::new(result).equals(1)?;
        
        // Send another message
        let result = actor.ask(&Increment).await?;
        
        // Verify the result
        Assert::new(result).equals(2)?;
        
        // Get the counter
        let counter = actor.ask(&GetCounter).await?;
        
        // Verify the counter
        Assert::new(counter).equals(2)?;
        
        // Clean up
        harness.cleanup().await?;
        
        Ok(())
    }

    // Test with mock dependency
    #[tokio::test]
    async fn test_example_actor_with_mock() -> Result<()> {
        // Create a test environment
        let test_env = create_test_env();
        
        // Create a mock actor
        let mock = create_mock_actor_with_env("dependency", test_env.clone());
        
        // Configure the mock to respond to DependencyRequest
        mock.mock_response::<DependencyRequest, DependencyResponse>(
            DependencyResponse::new("processed data")
        ).await?;
        
        // Create the actor with the mock dependency
        let actor = ExampleActor::spawn(ExampleActor::new().with_dependency(mock.clone()));
        
        // Create a test harness
        let harness = ActorTestHarness::new(test_env)
            .with_actor(actor);
        
        // Process data with the dependency
        let result = harness.actor.as_ref().unwrap()
            .ask(&ProcessWithDependency::new("test data"))
            .await?;
        
        // Verify the result
        Assert::new(result).equals("processed data".to_string())?;
        
        // Verify that the mock received the expected message
        mock.verify().await?;
        
        // Clean up
        harness.cleanup().await?;
        
        Ok(())
    }

    // Test with scenario
    #[tokio::test]
    async fn test_example_actor_scenario() -> Result<()> {
        // Create a test environment
        let test_env = create_test_env();
        
        // Create a mock actor
        let mock = create_mock_actor_with_env("dependency", test_env.clone());
        
        // Configure the mock to respond to DependencyRequest
        mock.mock_response::<DependencyRequest, DependencyResponse>(
            DependencyResponse::new("processed data")
        ).await?;
        
        // Create the actor with the mock dependency
        let actor = ExampleActor::spawn(ExampleActor::new().with_dependency(mock.clone()));
        
        // Create a scenario
        let scenario = create_actor_scenario("Example scenario", actor.clone())
            .add_step("Increment counter", |harness| {
                Box::pin(async move {
                    // Increment the counter
                    let result = harness.actor.as_ref().unwrap()
                        .ask(&Increment)
                        .await?;
                    
                    // Verify the result
                    Assert::new(result).equals(1)?;
                    Ok(())
                })
            })
            .add_step("Process data", |harness| {
                Box::pin(async move {
                    // Process data with the dependency
                    let result = harness.actor.as_ref().unwrap()
                        .ask(&ProcessWithDependency::new("test data"))
                        .await?;
                    
                    // Verify the result
                    Assert::new(result).equals("processed data".to_string())?;
                    Ok(())
                })
            })
            .add_step("Verify counter", |harness| {
                Box::pin(async move {
                    // Get the counter
                    let counter = harness.actor.as_ref().unwrap()
                        .ask(&GetCounter)
                        .await?;
                    
                    // Verify the counter
                    Assert::new(counter).equals(1)?;
                    Ok(())
                })
            });
        
        // Run the scenario
        scenario.run().await?;
        
        // Verify that the mock received the expected message
        mock.verify().await?;
        
        Ok(())
    }

    // Test with event verification
    #[tokio::test]
    async fn test_example_actor_events() -> Result<()> {
        // Create a test environment
        let test_env = create_test_env();
        
        // Create a test harness with the environment
        let harness = ActorTestHarness::new(test_env.clone())
            .with_actor(ExampleActor::spawn(ExampleActor::new()));
        
        // Send a message to the actor
        harness.actor.as_ref().unwrap()
            .ask(&Increment)
            .await?;
        
        // Verify that the expected events were captured
        let verifier = EventVerifier::new(test_env.clone());
        verifier.verify_event_count(TestEventType::MessageSent, 1)?;
        verifier.verify_event_count(TestEventType::MessageReceived, 1)?;
        
        // Clean up
        harness.cleanup().await?;
        
        Ok(())
    }
}