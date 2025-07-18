#[cfg(test)]
mod integration_tests {
    use super::*;
    use kameo::prelude::*;
    use kameo_actors::{DeliveryStrategy, message_bus::MessageBus};
    use sqlx::sqlite::SqlitePoolOptions;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_agent_lifecycle_integration() -> Result<()> {
        let (mut agent_manager, _bus) = setup_test_environment().await;
        let workspace_id = Uuid::new_v4();

        // Test creating an agent
        let create_request = CreateAgentRequest {
            name: "Integration Test Agent".to_string(),
            description: Some("An agent for integration testing".to_string()),
            avatar_url: None,
            type_: AgentType::Worker,
            capabilities: vec![
                "chat".to_string(),
                "search".to_string(),
                "analysis".to_string(),
            ],
            memory_config: Some(serde_json::json!({
                "max_memories": 1000,
                "retention_days": 30
            })),
            planning_config: Some(serde_json::json!({
                "max_depth": 5,
                "timeout_seconds": 30
            })),
            context_window: Some(8192),
            tool_config: Some(serde_json::json!({
                "allowed_tools": ["search", "calculator"],
                "max_tool_calls": 10
            })),
            personality: Some(serde_json::json!({
                "traits": ["helpful", "analytical", "concise"],
                "communication_style": "professional"
            })),
            security: Some(serde_json::json!({
                "max_session_time": 3600,
                "require_auth": true
            })),
            is_public: Some(false),
            is_user_operator: Some(false),
            operator_level: Some(0),
            delegation_rules: Some(serde_json::json!({
                "auto_delegate": false,
                "allowed_targets": []
            })),
            workspace_id,
            created_by_id: Some(Uuid::new_v4()),
            operator_user_id: None,
            parent_agent_id: None,
            registry_id: None,
        };

        let created_agent = agent_manager.service.create_agent(create_request).await?;
        let agent_id = created_agent.uuid()?;

        // Verify agent was created correctly
        assert_eq!(created_agent.name, "Integration Test Agent");
        assert_eq!(created_agent.agent_type(), AgentType::Worker);
        assert_eq!(created_agent.agent_status(), AgentStatus::Active);
        assert_eq!(created_agent.context_window, 8192);
        assert_eq!(created_agent.is_public, false);

        let capabilities = created_agent.capabilities()?;
        assert_eq!(capabilities.len(), 3);
        assert!(capabilities.contains(&"chat".to_string()));
        assert!(capabilities.contains(&"search".to_string()));
        assert!(capabilities.contains(&"analysis".to_string()));

        // Test retrieving the agent
        let retrieved_agent = agent_manager.service.get_agent(agent_id).await?;
        assert!(retrieved_agent.is_some());
        let retrieved_agent = retrieved_agent.unwrap();
        assert_eq!(retrieved_agent.uuid()?, agent_id);
        assert_eq!(retrieved_agent.name, "Integration Test Agent");

        // Test updating the agent
        let update_request = UpdateAgentRequest {
            id: agent_id,
            name: Some("Updated Integration Test Agent".to_string()),
            description: Some("Updated description".to_string()),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            status: None,
            config: Some(serde_json::json!({
                "model": "gpt-4",
                "temperature": 0.7
            })),
            capabilities: Some(vec![
                "chat".to_string(),
                "search".to_string(),
                "analysis".to_string(),
                "code_generation".to_string(),
            ]),
            memory_config: Some(serde_json::json!({
                "max_memories": 2000,
                "retention_days": 60
            })),
            planning_config: None,
            context_window: Some(16384),
            tool_config: Some(serde_json::json!({
                "allowed_tools": ["search", "calculator", "file_reader"],
                "max_tool_calls": 15
            })),
            personality: Some(serde_json::json!({
                "traits": ["helpful", "analytical", "concise", "creative"],
                "communication_style": "friendly"
            })),
            security: None,
            is_public: Some(true),
            delegation_rules: Some(serde_json::json!({
                "auto_delegate": true,
                "allowed_targets": ["search_agent", "analysis_agent"]
            })),
            metadata: Some(serde_json::json!({
                "version": "2.0",
                "last_training": "2024-01-01"
            })),
        };

        let updated_agent = agent_manager.service.update_agent(update_request).await?;
        assert_eq!(updated_agent.name, "Updated Integration Test Agent");
        assert_eq!(
            updated_agent.description,
            Some("Updated description".to_string())
        );
        assert_eq!(
            updated_agent.avatar_url,
            Some("https://example.com/avatar.png".to_string())
        );
        assert_eq!(updated_agent.context_window, 16384);
        assert_eq!(updated_agent.is_public, true);

        let updated_capabilities = updated_agent.capabilities()?;
        assert_eq!(updated_capabilities.len(), 4);
        assert!(updated_capabilities.contains(&"code_generation".to_string()));

        // Test recording metrics
        agent_manager
            .service
            .record_session_metrics(agent_id, 120.5, true)
            .await?;
        agent_manager
            .service
            .record_session_metrics(agent_id, 95.3, false)
            .await?;
        agent_manager
            .service
            .record_session_metrics(agent_id, 200.0, true)
            .await?;
        agent_manager
            .service
            .record_session_metrics(agent_id, 75.2, true)
            .await?;

        let metrics = agent_manager.service.get_agent_metrics(agent_id).await?;
        assert!(metrics.is_some());
        let metrics = metrics.unwrap();
        assert_eq!(metrics.total_sessions, 4);
        assert_eq!(metrics.successful_sessions, 3);
        assert!((metrics.success_rate - 75.0).abs() < 0.01);

        // Test listing agents
        let filter = AgentListFilter {
            workspace_id: Some(workspace_id),
            status: None,
            type_: None,
            is_public: None,
            is_user_operator: None,
            created_by_id: None,
            operator_user_id: None,
            parent_agent_id: None,
            limit: None,
            offset: None,
        };

        let agents = agent_manager.service.list_agents(filter).await?;
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].uuid()?, agent_id);

        // Test deleting the agent
        agent_manager.service.delete_agent(agent_id).await?;
        let deleted_agent = agent_manager.service.get_agent(agent_id).await?;
        assert!(deleted_agent.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_agent_hierarchy_integration() -> Result<()> {
        let (mut agent_manager, _bus) = setup_test_environment().await;
        let workspace_id = Uuid::new_v4();

        // Create parent operator agent
        let parent_request = CreateAgentRequest {
            name: "Parent Operator".to_string(),
            description: Some("Parent operator agent".to_string()),
            avatar_url: None,
            type_: AgentType::Operator,
            capabilities: vec!["delegation".to_string(), "coordination".to_string()],
            memory_config: None,
            planning_config: None,
            context_window: None,
            tool_config: None,
            personality: None,
            security: None,
            is_public: Some(false),
            is_user_operator: Some(false),
            operator_level: Some(0),
            delegation_rules: Some(serde_json::json!({
                "auto_delegate": true,
                "max_child_agents": 5
            })),
            workspace_id,
            created_by_id: None,
            operator_user_id: None,
            parent_agent_id: None,
            registry_id: None,
        };

        let parent_agent = agent_manager.service.create_agent(parent_request).await?;
        let parent_id = parent_agent.uuid()?;

        // Create child worker agents
        let mut child_ids = Vec::new();
        for i in 1..=3 {
            let child_request = CreateAgentRequest {
                name: format!("Child Worker {}", i),
                description: Some(format!("Child worker agent {}", i)),
                avatar_url: None,
                type_: AgentType::Worker,
                capabilities: vec![format!("task_{}", i), "reporting".to_string()],
                memory_config: None,
                planning_config: None,
                context_window: None,
                tool_config: None,
                personality: None,
                security: None,
                is_public: Some(false),
                is_user_operator: Some(false),
                operator_level: Some(1),
                delegation_rules: None,
                workspace_id,
                created_by_id: None,
                operator_user_id: None,
                parent_agent_id: Some(parent_id),
                registry_id: None,
            };

            let child_agent = agent_manager.service.create_agent(child_request).await?;
            child_ids.push(child_agent.uuid()?);
        }

        // Test retrieving child agents
        let child_filter = AgentListFilter {
            workspace_id: None,
            status: None,
            type_: None,
            is_public: None,
            is_user_operator: None,
            created_by_id: None,
            operator_user_id: None,
            parent_agent_id: Some(parent_id),
            limit: None,
            offset: None,
        };

        let child_agents = agent_manager.service.list_agents(child_filter).await?;
        assert_eq!(child_agents.len(), 3);

        for child_agent in &child_agents {
            assert_eq!(child_agent.agent_type(), AgentType::Worker);
            assert_eq!(child_agent.operator_level, 1);
        }

        // Test operator level filtering
        let operator_filter = AgentListFilter {
            workspace_id: Some(workspace_id),
            status: None,
            type_: Some(AgentType::Operator),
            is_public: None,
            is_user_operator: None,
            created_by_id: None,
            operator_user_id: None,
            parent_agent_id: None,
            limit: None,
            offset: None,
        };

        let operators = agent_manager.service.list_agents(operator_filter).await?;
        assert_eq!(operators.len(), 1);
        assert_eq!(operators[0].uuid()?, parent_id);

        Ok(())
    }

    #[tokio::test]
    async fn test_user_operator_validation_integration() -> Result<()> {
        let (mut agent_manager, _bus) = setup_test_environment().await;
        let workspace_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Test valid user operator creation
        let valid_request = CreateAgentRequest {
            name: "User Operator Agent".to_string(),
            description: Some("A valid user operator".to_string()),
            avatar_url: None,
            type_: AgentType::Operator,
            capabilities: vec!["user_interface".to_string(), "task_management".to_string()],
            memory_config: None,
            planning_config: None,
            context_window: None,
            tool_config: None,
            personality: None,
            security: None,
            is_public: Some(false),
            is_user_operator: Some(true),
            operator_level: Some(0),
            delegation_rules: None,
            workspace_id,
            created_by_id: None,
            operator_user_id: Some(user_id), // Required for user operators
            parent_agent_id: None,           // Should be None for user operators
            registry_id: None,
        };

        let user_operator = agent_manager.service.create_agent(valid_request).await?;
        assert!(user_operator.is_user_operator);
        assert_eq!(
            user_operator.operator_user_id,
            Some(user_id.as_bytes().to_vec())
        );

        // Test invalid user operator creation (missing operator_user_id)
        let invalid_request_1 = CreateAgentRequest {
            name: "Invalid User Operator 1".to_string(),
            description: None,
            avatar_url: None,
            type_: AgentType::Operator,
            capabilities: vec!["user_interface".to_string()],
            memory_config: None,
            planning_config: None,
            context_window: None,
            tool_config: None,
            personality: None,
            security: None,
            is_public: None,
            is_user_operator: Some(true),
            operator_level: None,
            delegation_rules: None,
            workspace_id,
            created_by_id: None,
            operator_user_id: None, // Should cause validation error
            parent_agent_id: None,
            registry_id: None,
        };

        let result_1 = agent_manager.service.create_agent(invalid_request_1).await;
        assert!(result_1.is_err());

        // Test invalid user operator creation (has parent_agent_id)
        let invalid_request_2 = CreateAgentRequest {
            name: "Invalid User Operator 2".to_string(),
            description: None,
            avatar_url: None,
            type_: AgentType::Operator,
            capabilities: vec!["user_interface".to_string()],
            memory_config: None,
            planning_config: None,
            context_window: None,
            tool_config: None,
            personality: None,
            security: None,
            is_public: None,
            is_user_operator: Some(true),
            operator_level: None,
            delegation_rules: None,
            workspace_id,
            created_by_id: None,
            operator_user_id: Some(user_id),
            parent_agent_id: Some(Uuid::new_v4()), // Should cause validation error
            registry_id: None,
        };

        let result_2 = agent_manager.service.create_agent(invalid_request_2).await;
        assert!(result_2.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_agent_manager_actor_messages() -> Result<()> {
        let (agent_manager, _bus) = setup_test_environment().await;
        let agent_manager_ref = spawn(agent_manager);
        let workspace_id = Uuid::new_v4();

        // Test CreateAgent message
        let create_msg = CreateAgent {
            request: CreateAgentRequest {
                name: "Actor Test Agent".to_string(),
                description: Some("Testing actor messages".to_string()),
                avatar_url: None,
                type_: AgentType::Worker,
                capabilities: vec!["testing".to_string()],
                memory_config: None,
                planning_config: None,
                context_window: None,
                tool_config: None,
                personality: None,
                security: None,
                is_public: None,
                is_user_operator: None,
                operator_level: None,
                delegation_rules: None,
                workspace_id,
                created_by_id: None,
                operator_user_id: None,
                parent_agent_id: None,
                registry_id: None,
            },
        };

        let created_agent = timeout(Duration::from_secs(5), agent_manager_ref.ask(create_msg))
            .await
            .expect("Timeout waiting for response")
            .expect("Actor message failed")?;

        let agent_id = created_agent.uuid()?;

        // Test GetAgent message
        let get_msg = GetAgent { id: agent_id };
        let retrieved_agent = timeout(Duration::from_secs(5), agent_manager_ref.ask(get_msg))
            .await
            .expect("Timeout waiting for response")
            .expect("Actor message failed")?;

        assert!(retrieved_agent.is_some());
        assert_eq!(retrieved_agent.unwrap().uuid()?, agent_id);

        // Test ListAgents message
        let list_msg = ListAgents {
            filter: AgentListFilter {
                workspace_id: Some(workspace_id),
                status: None,
                type_: None,
                is_public: None,
                is_user_operator: None,
                created_by_id: None,
                operator_user_id: None,
                parent_agent_id: None,
                limit: None,
                offset: None,
            },
        };

        let agents = timeout(Duration::from_secs(5), agent_manager_ref.ask(list_msg))
            .await
            .expect("Timeout waiting for response")
            .expect("Actor message failed")?;

        assert_eq!(agents.len(), 1);

        // Test RecordSessionMetrics message
        let metrics_msg = RecordSessionMetrics {
            agent_id,
            session_duration: 150.0,
            success: true,
        };

        timeout(Duration::from_secs(5), agent_manager_ref.ask(metrics_msg))
            .await
            .expect("Timeout waiting for response")
            .expect("Actor message failed")?;

        // Test GetAgentMetrics message
        let get_metrics_msg = GetAgentMetrics { id: agent_id };
        let metrics = timeout(
            Duration::from_secs(5),
            agent_manager_ref.ask(get_metrics_msg),
        )
        .await
        .expect("Timeout waiting for response")
        .expect("Actor message failed")?;

        assert!(metrics.is_some());
        let metrics = metrics.unwrap();
        assert_eq!(metrics.total_sessions, 1);
        assert_eq!(metrics.successful_sessions, 1);
        assert_eq!(metrics.success_rate, 100.0);

        // Test DeleteAgent message
        let delete_msg = DeleteAgent { id: agent_id };
        timeout(Duration::from_secs(5), agent_manager_ref.ask(delete_msg))
            .await
            .expect("Timeout waiting for response")
            .expect("Actor message failed")?;

        // Verify agent was deleted
        let get_deleted_msg = GetAgent { id: agent_id };
        let deleted_agent = timeout(
            Duration::from_secs(5),
            agent_manager_ref.ask(get_deleted_msg),
        )
        .await
        .expect("Timeout waiting for response")
        .expect("Actor message failed")?;

        assert!(deleted_agent.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_agent_operations() -> Result<()> {
        let (agent_manager, _bus) = setup_test_environment().await;
        let agent_manager_ref = spawn(agent_manager);
        let workspace_id = Uuid::new_v4();

        // Create multiple agents concurrently
        let create_tasks: Vec<_> = (0..10)
            .map(|i| {
                let agent_manager_ref = agent_manager_ref.clone();
                let workspace_id = workspace_id;
                tokio::spawn(async move {
                    let create_msg = CreateAgent {
                        request: CreateAgentRequest {
                            name: format!("Concurrent Agent {}", i),
                            description: Some(format!("Concurrent test agent {}", i)),
                            avatar_url: None,
                            type_: AgentType::Worker,
                            capabilities: vec![format!("task_{}", i)],
                            memory_config: None,
                            planning_config: None,
                            context_window: None,
                            tool_config: None,
                            personality: None,
                            security: None,
                            is_public: None,
                            is_user_operator: None,
                            operator_level: None,
                            delegation_rules: None,
                            workspace_id,
                            created_by_id: None,
                            operator_user_id: None,
                            parent_agent_id: None,
                            registry_id: None,
                        },
                    };
                    agent_manager_ref
                        .ask(create_msg)
                        .await
                        .map_err(|e| e.into())
                })
            })
            .collect();

        // Wait for all agents to be created
        let created_agents: Vec<_> = futures::future::try_join_all(create_tasks)
            .await
            .expect("Failed to create agents concurrently")?;

        assert_eq!(created_agents.len(), 10);

        // Verify all agents were created
        let list_msg = ListAgents {
            filter: AgentListFilter {
                workspace_id: Some(workspace_id),
                status: None,
                type_: None,
                is_public: None,
                is_user_operator: None,
                created_by_id: None,
                operator_user_id: None,
                parent_agent_id: None,
                limit: None,
                offset: None,
            },
        };

        let all_agents = agent_manager_ref.ask(list_msg).await?;
        assert_eq!(all_agents.len(), 10);

        // Update all agents concurrently
        let update_tasks: Vec<_> = created_agents
            .iter()
            .enumerate()
            .map(|(i, agent)| {
                let agent_manager_ref = agent_manager_ref.clone();
                let agent_id = agent.uuid().unwrap();
                tokio::spawn(async move {
                    let update_msg = UpdateAgent {
                        request: UpdateAgentRequest {
                            id: agent_id,
                            name: Some(format!("Updated Concurrent Agent {}", i)),
                            description: Some(format!("Updated concurrent test agent {}", i)),
                            avatar_url: None,
                            status: None,
                            config: None,
                            capabilities: Some(vec![format!("updated_task_{}", i)]),
                            memory_config: None,
                            planning_config: None,
                            context_window: None,
                            tool_config: None,
                            personality: None,
                            security: None,
                            is_public: None,
                            delegation_rules: None,
                            metadata: None,
                        },
                    };
                    agent_manager_ref
                        .ask(update_msg)
                        .await
                        .map_err(|e| e.into())
                })
            })
            .collect();

        let updated_agents: Vec<_> = futures::future::try_join_all(update_tasks)
            .await
            .expect("Failed to update agents concurrently")?;

        assert_eq!(updated_agents.len(), 10);

        // Verify all agents were updated
        for (i, agent) in updated_agents.iter().enumerate() {
            assert_eq!(agent.name, format!("Updated Concurrent Agent {}", i));
            let capabilities = agent.capabilities()?;
            assert_eq!(capabilities, vec![format!("updated_task_{}", i)]);
        }

        Ok(())
    }
}

