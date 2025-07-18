//! Test utilities for repositories
//!
//! This module provides utilities for testing repositories, including
//! test data generation and verification helpers.

use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::error::Result;
use crate::storage::db::DatabaseManager;

/// Initialize an in-memory database for testing
pub async fn setup_test_db() -> Pool<Sqlite> {
    let db = DatabaseManager::setup_test_db().await;
    db.pool.clone()
}

/// Test data generator for repositories
pub mod generators {
    use chrono::{DateTime, Duration, Utc};
    use serde_json::json;
    use sqlx::types::Json;
    use uuid::Uuid;

    use crate::entities::tasks::{Task, TaskImportance, TaskPriority, TaskStatus};

    /// Generate a test task with the given ID
    pub fn task(id: Uuid) -> Task {
        let now = Utc::now();
        Task {
            id,
            title: format!("Test Task {}", id),
            description: Some(format!("Description for test task {}", id)),
            status: TaskStatus::Pending,
            start_time: now,
            end_time: None,
            due_date: Some(now + Duration::days(7)),
            priority: TaskPriority::Medium,
            importance: TaskImportance::Medium,
            tags: Json(json!(["test", "sample"])),
            url: None,
            metadata: None,
            created_at: now,
            updated_at: now,
            created_by_id: None,
            assignee_participant_id: None,
            workspace_id: None,
            conversation_id: None,
            memory_id: None,
            plan_id: None,
            document_id: None,
            file_id: None,
        }
    }

    /// Generate a batch of test tasks
    pub fn tasks(count: usize) -> Vec<Task> {
        (0..count).map(|_| task(Uuid::new_v4())).collect()
    }

    /// Generate a completed test task
    pub fn completed_task(id: Uuid) -> Task {
        let now = Utc::now();
        let mut task = task(id);
        task.status = TaskStatus::Completed;
        task.end_time = Some(now);
        task
    }

    /// Generate a high priority test task
    pub fn high_priority_task(id: Uuid) -> Task {
        let mut task = task(id);
        task.priority = TaskPriority::High;
        task.importance = TaskImportance::High;
        task
    }

    /// Generate an overdue test task
    pub fn overdue_task(id: Uuid) -> Task {
        let now = Utc::now();
        let mut task = task(id);
        task.due_date = Some(now - Duration::days(1));
        task
    }
}

/// Test assertions for repositories
pub mod assertions {
    use std::collections::HashSet;
    use uuid::Uuid;

    use crate::entities::tasks::Task;
    use crate::error::Result;
    use crate::repositories::task_repository::{TaskRepository, TaskStats};

    /// Assert that a task exists in the repository
    pub async fn assert_task_exists(repo: &TaskRepository, id: &Uuid) -> Result<()> {
        assert!(repo.exists(id).await?, "Task with ID {} should exist", id);
        Ok(())
    }

    /// Assert that a task does not exist in the repository
    pub async fn assert_task_not_exists(repo: &TaskRepository, id: &Uuid) -> Result<()> {
        assert!(!repo.exists(id).await?, "Task with ID {} should not exist", id);
        Ok(())
    }

    /// Assert that the task count matches the expected count
    pub async fn assert_task_count(repo: &TaskRepository, expected: i64) -> Result<()> {
        let count = repo.count(&Default::default()).await?;
        assert_eq!(count, expected, "Task count should be {}", expected);
        Ok(())
    }

    /// Assert that the task stats match the expected stats
    pub async fn assert_task_stats(repo: &TaskRepository, expected: TaskStats) -> Result<()> {
        let stats = repo.get_task_stats().await?;
        assert_eq!(stats.total_tasks, expected.total_tasks, "Total tasks should match");
        assert_eq!(stats.pending_tasks, expected.pending_tasks, "Pending tasks should match");
        assert_eq!(stats.in_progress_tasks, expected.in_progress_tasks, "In-progress tasks should match");
        assert_eq!(stats.completed_tasks, expected.completed_tasks, "Completed tasks should match");
        assert_eq!(stats.failed_tasks, expected.failed_tasks, "Failed tasks should match");
        assert_eq!(stats.high_priority_tasks, expected.high_priority_tasks, "High priority tasks should match");
        assert_eq!(stats.important_tasks, expected.important_tasks, "Important tasks should match");
        assert_eq!(stats.overdue_tasks, expected.overdue_tasks, "Overdue tasks should match");
        Ok(())
    }

    /// Assert that the tasks match the expected tasks
    pub async fn assert_tasks_match(actual: &[Task], expected: &[Task]) -> Result<()> {
        assert_eq!(actual.len(), expected.len(), "Task count should match");
        
        let actual_ids: HashSet<Uuid> = actual.iter().map(|t| t.id).collect();
        let expected_ids: HashSet<Uuid> = expected.iter().map(|t| t.id).collect();
        
        assert_eq!(actual_ids, expected_ids, "Task IDs should match");
        
        for expected_task in expected {
            let actual_task = actual.iter().find(|t| t.id == expected_task.id)
                .expect(&format!("Task with ID {} should exist", expected_task.id));
            
            assert_eq!(actual_task.title, expected_task.title, "Task title should match");
            assert_eq!(actual_task.status, expected_task.status, "Task status should match");
            assert_eq!(actual_task.priority, expected_task.priority, "Task priority should match");
            assert_eq!(actual_task.importance, expected_task.importance, "Task importance should match");
        }
        
        Ok(())
    }
}