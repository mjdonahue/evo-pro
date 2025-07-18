//! Tests for the TaskRepository
//!
//! This module contains tests for the TaskRepository implementation.

use uuid::Uuid;

use crate::entities::tasks::{Task, TaskFilter, TaskStatus};
use crate::repositories::task_repository::TaskRepository;
use crate::repositories::tests::{assertions, generators, setup_test_db};

#[tokio::test]
async fn test_create_task() -> crate::error::Result<()> {
    // Setup
    let pool = setup_test_db().await;
    let repo = TaskRepository::new(pool);
    let task_id = Uuid::new_v4();
    let task = generators::task(task_id);
    
    // Execute
    let created_task = repo.create(&task).await?;
    
    // Verify
    assert_eq!(created_task.id, task_id);
    assert_eq!(created_task.title, task.title);
    assert_eq!(created_task.status, TaskStatus::Pending);
    
    // Verify using assertions
    assertions::assert_task_exists(&repo, &task_id).await?;
    assertions::assert_task_count(&repo, 1).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_get_task_by_id() -> crate::error::Result<()> {
    // Setup
    let pool = setup_test_db().await;
    let repo = TaskRepository::new(pool);
    let task_id = Uuid::new_v4();
    let task = generators::task(task_id);
    repo.create(&task).await?;
    
    // Execute
    let retrieved_task = repo.get_by_id(&task_id).await?;
    
    // Verify
    assert!(retrieved_task.is_some());
    let retrieved_task = retrieved_task.unwrap();
    assert_eq!(retrieved_task.id, task_id);
    assert_eq!(retrieved_task.title, task.title);
    
    Ok(())
}

#[tokio::test]
async fn test_update_task() -> crate::error::Result<()> {
    // Setup
    let pool = setup_test_db().await;
    let repo = TaskRepository::new(pool);
    let task_id = Uuid::new_v4();
    let mut task = generators::task(task_id);
    repo.create(&task).await?;
    
    // Modify the task
    task.title = "Updated Task Title".to_string();
    task.status = TaskStatus::InProgress;
    
    // Execute
    repo.update(&task).await?;
    
    // Verify
    let updated_task = repo.get_by_id(&task_id).await?.unwrap();
    assert_eq!(updated_task.title, "Updated Task Title");
    assert_eq!(updated_task.status, TaskStatus::InProgress);
    
    Ok(())
}

#[tokio::test]
async fn test_delete_task() -> crate::error::Result<()> {
    // Setup
    let pool = setup_test_db().await;
    let repo = TaskRepository::new(pool);
    let task_id = Uuid::new_v4();
    let task = generators::task(task_id);
    repo.create(&task).await?;
    
    // Verify task exists
    assertions::assert_task_exists(&repo, &task_id).await?;
    
    // Execute
    repo.delete(&task_id).await?;
    
    // Verify task no longer exists
    assertions::assert_task_not_exists(&repo, &task_id).await?;
    assertions::assert_task_count(&repo, 0).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_list_tasks() -> crate::error::Result<()> {
    // Setup
    let pool = setup_test_db().await;
    let repo = TaskRepository::new(pool);
    
    // Create multiple tasks
    let task1 = generators::task(Uuid::new_v4());
    let task2 = generators::high_priority_task(Uuid::new_v4());
    let task3 = generators::completed_task(Uuid::new_v4());
    
    repo.create(&task1).await?;
    repo.create(&task2).await?;
    repo.create(&task3).await?;
    
    // Execute - list all tasks
    let all_tasks = repo.list(&TaskFilter::default()).await?;
    
    // Verify
    assert_eq!(all_tasks.len(), 3);
    
    // Execute - list only high priority tasks
    let high_priority_filter = TaskFilter {
        priority: Some(crate::entities::tasks::TaskPriority::High),
        ..Default::default()
    };
    let high_priority_tasks = repo.list(&high_priority_filter).await?;
    
    // Verify
    assert_eq!(high_priority_tasks.len(), 1);
    assert_eq!(high_priority_tasks[0].id, task2.id);
    
    // Execute - list only completed tasks
    let completed_filter = TaskFilter {
        status: Some(TaskStatus::Completed),
        ..Default::default()
    };
    let completed_tasks = repo.list(&completed_filter).await?;
    
    // Verify
    assert_eq!(completed_tasks.len(), 1);
    assert_eq!(completed_tasks[0].id, task3.id);
    
    Ok(())
}

#[tokio::test]
async fn test_batch_operations() -> crate::error::Result<()> {
    // Setup
    let pool = setup_test_db().await;
    let repo = TaskRepository::new(pool);
    
    // Generate batch of tasks
    let tasks = generators::tasks(5);
    let task_ids: Vec<Uuid> = tasks.iter().map(|t| t.id).collect();
    
    // Execute - batch create
    let created_tasks = repo.batch_create(&tasks).await?;
    
    // Verify
    assertions::assert_task_count(&repo, 5).await?;
    assertions::assert_tasks_match(&created_tasks, &tasks).await?;
    
    // Execute - batch get by ids
    let retrieved_tasks = repo.get_by_ids(&task_ids).await?;
    
    // Verify
    assertions::assert_tasks_match(&retrieved_tasks, &tasks).await?;
    
    // Execute - batch update
    let mut updated_tasks = created_tasks.clone();
    for task in &mut updated_tasks {
        task.title = format!("Updated {}", task.title);
        task.status = TaskStatus::InProgress;
    }
    
    repo.batch_update(&updated_tasks).await?;
    
    // Verify
    let retrieved_tasks = repo.get_by_ids(&task_ids).await?;
    assertions::assert_tasks_match(&retrieved_tasks, &updated_tasks).await?;
    
    // Execute - batch delete
    repo.batch_delete(&task_ids).await?;
    
    // Verify
    assertions::assert_task_count(&repo, 0).await?;
    for id in &task_ids {
        assertions::assert_task_not_exists(&repo, id).await?;
    }
    
    Ok(())
}

#[tokio::test]
async fn test_task_stats() -> crate::error::Result<()> {
    // Setup
    let pool = setup_test_db().await;
    let repo = TaskRepository::new(pool);
    
    // Create tasks with different statuses
    let pending_task = generators::task(Uuid::new_v4());
    let in_progress_task = generators::task(Uuid::new_v4());
    let completed_task = generators::completed_task(Uuid::new_v4());
    let high_priority_task = generators::high_priority_task(Uuid::new_v4());
    let overdue_task = generators::overdue_task(Uuid::new_v4());
    
    // Update in_progress_task status
    let mut in_progress = in_progress_task.clone();
    in_progress.status = TaskStatus::InProgress;
    
    // Create all tasks
    repo.create(&pending_task).await?;
    repo.create(&in_progress).await?;
    repo.create(&completed_task).await?;
    repo.create(&high_priority_task).await?;
    repo.create(&overdue_task).await?;
    
    // Execute
    let stats = repo.get_task_stats().await?;
    
    // Verify
    assert_eq!(stats.total_tasks, 5);
    assert_eq!(stats.pending_tasks, 2); // pending_task and overdue_task
    assert_eq!(stats.in_progress_tasks, 1);
    assert_eq!(stats.completed_tasks, 1);
    assert_eq!(stats.failed_tasks, 0);
    assert_eq!(stats.high_priority_tasks, 1);
    assert_eq!(stats.overdue_tasks, 1);
    
    Ok(())
}