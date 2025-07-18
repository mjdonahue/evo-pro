use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use kameo::prelude::*;
use tokio::time::sleep;

use crate::actors::supervision::{SupervisionStrategy, SupervisorActor, SupervisionExt, create_fault_tolerant_supervisor};
use crate::actors::supervision_tree::{SupervisionTreeNode, SupervisionTreeNodeType, create_supervision_tree_root, SupervisionTreeExt};
use crate::actors::fault_detection::{create_heartbeat_monitor, create_circuit_breaker_actor};
use crate::actors::lifecycle::{create_lifecycle_manager};
use crate::error::Result;

// Test actor that can be configured to fail in different ways
#[derive(Actor, Clone)]
struct TestActor {
    name: String,
    fail_on_message: bool,
    restart_count: Arc<AtomicUsize>,
}

impl TestActor {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fail_on_message: false,
            restart_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn with_failure(mut self) -> Self {
        self.fail_on_message = true;
        self
    }
}

// Message that can trigger a failure
#[derive(Debug, Clone)]
struct TriggerFailure;

impl Message<TriggerFailure> for TestActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        _msg: TriggerFailure,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if self.fail_on_message {
            panic!("Actor {} failed as configured", self.name);
        }

        Ok(())
    }
}

// Message to get the restart count
#[derive(Debug, Clone)]
struct GetRestartCount;

impl Message<GetRestartCount> for TestActor {
    type Reply = usize;

    async fn handle(
        &mut self,
        _msg: GetRestartCount,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.restart_count.load(Ordering::SeqCst)
    }
}

impl Actor for TestActor {
    fn on_start(&mut self, _ctx: &mut Context<Self, ()>) {
        self.restart_count.fetch_add(1, Ordering::SeqCst);
    }
}

#[tokio::test]
async fn test_basic_supervision() -> Result<()> {
    // Create a supervisor with restart strategy
    let supervisor = SupervisorActor::spawn(
        SupervisorActor::new("test-supervisor", SupervisionStrategy::Restart)
    );

    // Create a test actor that will fail
    let test_actor = TestActor::new("test-actor").with_failure();
    let restart_count = test_actor.restart_count.clone();

    // Supervise the actor
    let actor_ref = supervisor
        .ask(&crate::actors::supervision::SuperviseActor {
            actor: test_actor,
            strategy: None,
        })
        .await?;

    // Trigger a failure
    let _ = actor_ref.ask(&TriggerFailure).await;

    // Wait for restart
    sleep(Duration::from_millis(100)).await;

    // Check that the actor was restarted
    let restart_count = restart_count.load(Ordering::SeqCst);
    assert!(restart_count > 1, "Actor should have been restarted at least once");

    Ok(())
}

#[tokio::test]
async fn test_failure_specific_strategy() -> Result<()> {
    // Create a supervisor with failure-specific strategy
    let strategy = SupervisionStrategy::FailureSpecific {
        panic: Box::new(SupervisionStrategy::Restart),
        peer_disconnected: Box::new(SupervisionStrategy::Stop),
        killed: Box::new(SupervisionStrategy::RestartWithDelay(Duration::from_millis(50))),
    };

    let supervisor = SupervisorActor::spawn(
        SupervisorActor::new("test-supervisor", strategy)
    );

    // Create a test actor that will fail
    let test_actor = TestActor::new("test-actor").with_failure();
    let restart_count = test_actor.restart_count.clone();

    // Supervise the actor
    let actor_ref = supervisor
        .ask(&crate::actors::supervision::SuperviseActor {
            actor: test_actor,
            strategy: None,
        })
        .await?;

    // Trigger a failure (panic)
    let _ = actor_ref.ask(&TriggerFailure).await;

    // Wait for restart
    sleep(Duration::from_millis(100)).await;

    // Check that the actor was restarted
    let restart_count = restart_count.load(Ordering::SeqCst);
    assert!(restart_count > 1, "Actor should have been restarted after panic");

    Ok(())
}

#[tokio::test]
async fn test_supervision_tree() -> Result<()> {
    // Create a supervision tree root with one-for-one strategy
    let root = create_supervision_tree_root::<TestActor>(
        "root",
        SupervisionTreeNodeType::OneForOne,
        SupervisionStrategy::Restart,
    );

    // Create a test actor that will fail
    let test_actor = TestActor::new("test-actor").with_failure();
    let restart_count = test_actor.restart_count.clone();

    // Supervise the actor with the root node
    let actor_ref = root
        .ask(&crate::actors::supervision_tree::SuperviseTreeActor {
            actor: test_actor,
            strategy: None,
        })
        .await?;

    // Trigger a failure
    let _ = actor_ref.ask(&TriggerFailure).await;

    // Wait for restart
    sleep(Duration::from_millis(100)).await;

    // Check that the actor was restarted
    let restart_count = restart_count.load(Ordering::SeqCst);
    assert!(restart_count > 1, "Actor should have been restarted at least once");

    Ok(())
}

#[tokio::test]
async fn test_fault_tolerant_supervisor() -> Result<()> {
    // Create the fault detection components
    let heartbeat_monitor = create_heartbeat_monitor(
        Duration::from_millis(50),
        Duration::from_millis(150),
    );

    let circuit_breaker = create_circuit_breaker_actor();

    let lifecycle_manager = create_lifecycle_manager(Duration::from_secs(1));

    // Create a fault-tolerant supervisor
    let supervisor = create_fault_tolerant_supervisor::<TestActor>(
        "fault-tolerant-supervisor",
        SupervisionStrategy::Restart,
        Some(&heartbeat_monitor),
        Some(&circuit_breaker),
        Some(&lifecycle_manager),
    );

    // Create a test actor that will fail
    let test_actor = TestActor::new("test-actor").with_failure();
    let restart_count = test_actor.restart_count.clone();

    // Supervise the actor
    let actor_ref = supervisor
        .ask(&crate::actors::supervision::SuperviseActor {
            actor: test_actor,
            strategy: None,
        })
        .await?;

    // Start sending heartbeats
    actor_ref.start_heartbeats(&heartbeat_monitor, Duration::from_millis(50)).await?;

    // Trigger a failure
    let _ = actor_ref.ask(&TriggerFailure).await;

    // Wait for restart
    sleep(Duration::from_millis(200)).await;

    // Check that the actor was restarted
    let restart_count = restart_count.load(Ordering::SeqCst);
    assert!(restart_count > 1, "Actor should have been restarted at least once");

    Ok(())
}
