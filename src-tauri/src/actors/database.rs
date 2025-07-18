use chrono::Utc;
use kameo::prelude::{ActorRef as LocalActorRef, *};
use uuid::Uuid;

use crate::{
    entities::{
        Agent, AgentFilter, Conversation, ConversationFilter, ConversationParticipant, CreateAgent,
        CreateConversation, CreateConversationParticipant, CreateP2pNode, CreateParticipant,
        CreateTask, CreateUser, P2pNode, Participant, ParticipantFilter, ParticipantType, Task,
        TaskFilter, User, UserFilter,
    },
    error::Result,
    repositories::RepositoryFactory,
    storage::db::DatabaseManager,
};

#[derive(Actor)]
pub struct DatabaseActor {
    pub db: DatabaseManager,
    pub repo_factory: RepositoryFactory,
}

impl Message<GetConversationParticipantIds> for DatabaseActor {
    type Reply = Result<Vec<ParticipantType>>;

    async fn handle(
        &mut self,
        msg: GetConversationParticipantIds,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        Ok(self
            .db
            .get_conversation_participant_ids(&msg.0, false)
            .await?)
    }
}

impl Message<CreateConversation> for DatabaseActor {
    type Reply = Result<Conversation>;

    async fn handle(
        &mut self,
        msg: CreateConversation,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        Ok(self.db.create_conversation(&msg).await?)
    }
}

impl Message<CreateBatchParticipants> for DatabaseActor {
    type Reply = Result<Vec<ConversationParticipant>>;

    async fn handle(
        &mut self,
        msg: CreateBatchParticipants,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        Ok(self.db.create_batch_participants(msg.0).await?)
    }
}

impl Message<CreateAgent> for DatabaseActor {
    type Reply = Result<Agent>;

    async fn handle(
        &mut self,
        msg: CreateAgent,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        Ok(self.db.create_agent(&msg).await?)
    }
}

impl Message<UpdateAgent> for DatabaseActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: UpdateAgent,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        Ok(self.db.update_agent(&msg.0).await?)
    }
}

impl Message<DeleteAgent> for DatabaseActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: DeleteAgent,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        Ok(self.db.delete_agent(&msg.0).await?)
    }
}

impl Message<ListTasks> for DatabaseActor {
    type Reply = Result<Vec<Task>>;

    async fn handle(
        &mut self,
        msg: ListTasks,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let task_repo = self.repo_factory.create_task_repository();
        task_repo.list(&msg.0).await
    }
}

impl Message<CreateTask> for DatabaseActor {
    type Reply = Result<Task>;

    async fn handle(
        &mut self,
        msg: CreateTask,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let task = Task {
            id: Uuid::new_v4(),
            title: msg.title,
            description: msg.description,
            status: msg.status,
            start_time: msg.start_time,
            end_time: msg.end_time,
            due_date: msg.due_date,
            priority: msg.priority,
            importance: msg.importance,
            tags: msg.tags,
            url: msg.url,
            metadata: msg.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by_id: msg.created_by_id,
            assignee_participant_id: msg.assignee_participant_id,
            workspace_id: msg.workspace_id,
            conversation_id: msg.conversation_id,
            memory_id: msg.memory_id,
            plan_id: msg.plan_id,
            document_id: msg.document_id,
            file_id: msg.file_id,
        };
        let task_repo = self.repo_factory.create_task_repository();
        task_repo.create(&task).await
    }
}

impl Message<UpdateTask> for DatabaseActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: UpdateTask,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let task_repo = self.repo_factory.create_task_repository();
        task_repo.update(&msg.0).await
    }
}

impl Message<DeleteTask> for DatabaseActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: DeleteTask,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let task_repo = self.repo_factory.create_task_repository();
        task_repo.delete(&msg.0).await
    }
}

impl Message<ListUsers> for DatabaseActor {
    type Reply = Result<Vec<User>>;

    async fn handle(
        &mut self,
        msg: ListUsers,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        Ok(self.db.list_users(&msg.0).await?)
    }
}

impl Message<UpdateUser> for DatabaseActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: UpdateUser,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        Ok(self.db.update_user(&msg.0).await?)
    }
}

impl Message<DeleteUser> for DatabaseActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: DeleteUser,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Ok(self.db.delete_user(&msg).await?)
        todo!()
    }
}

impl Message<CreateUser> for DatabaseActor {
    type Reply = Result<User>;

    async fn handle(
        &mut self,
        msg: CreateUser,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let user = User {
            id: Uuid::new_v4(),
            contact_id: msg.contact_id,
            email: msg.email,
            username: msg.username,
            operator_agent_id: msg.operator_agent_id,
            display_name: msg.display_name,
            first_name: msg.first_name,
            last_name: msg.last_name,
            mobile_phone: msg.mobile_phone,
            avatar_url: msg.avatar_url,
            bio: msg.bio,
            status: msg.status,
            email_verified: msg.email_verified,
            phone_verified: msg.phone_verified,
            last_seen: msg.last_seen,
            primary_role: msg.primary_role,
            roles: msg.roles,
            preferences: msg.preferences,
            metadata: msg.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            workspace_id: msg.workspace_id,
            public_key: msg.public_key,
        };
        Ok(self.db.create_user(&user).await?)
    }
}

impl Message<ListAgents> for DatabaseActor {
    type Reply = Result<Vec<Agent>>;

    async fn handle(
        &mut self,
        msg: ListAgents,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        Ok(self.db.list_agents(&msg.0).await?)
    }
}

impl Message<CreateP2pNode> for DatabaseActor {
    type Reply = Result<P2pNode>;

    async fn handle(
        &mut self,
        msg: CreateP2pNode,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        Ok(self.db.create_p2p_node(&msg).await?)
    }
}

impl Message<UpdateP2pNode> for DatabaseActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: UpdateP2pNode,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        Ok(self.db.update_p2p_node(&msg.0).await?)
    }
}

impl Message<DeleteP2pNode> for DatabaseActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: DeleteP2pNode,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        Ok(self.db.delete_p2p_node(&msg.0, &msg.1).await?)
    }
}

impl Message<CreateParticipant> for DatabaseActor {
    type Reply = Result<Participant>;

    async fn handle(
        &mut self,
        msg: CreateParticipant,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        Ok(self.db.create_participant(&msg).await?)
    }
}

impl Message<UpdateParticipant> for DatabaseActor {
    type Reply = Result<Participant>;

    async fn handle(
        &mut self,
        msg: UpdateParticipant,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        Ok(self.db.update_participant(&msg.0).await?)
    }
}

impl Message<DeleteParticipant> for DatabaseActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: DeleteParticipant,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        Ok(self.db.delete_participant(&msg.0).await?)
    }
}

impl Message<ListParticipants> for DatabaseActor {
    type Reply = Result<Vec<Participant>>;

    async fn handle(
        &mut self,
        msg: ListParticipants,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.db.list_participants(&msg.0).await
    }
}

impl Message<ListConversations> for DatabaseActor {
    type Reply = Result<Vec<Conversation>>;

    async fn handle(
        &mut self,
        msg: ListConversations,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.db.list_conversations(&msg.0).await
    }
}

pub struct GetConversationParticipantIds(pub Uuid);
pub struct CreateBatchParticipants(pub Vec<CreateConversationParticipant>);
pub struct ListAgents(pub AgentFilter);
pub struct UpdateAgent(pub Agent);
pub struct DeleteAgent(pub Uuid);
pub struct ListTasks(pub TaskFilter);
pub struct UpdateTask(pub Task);
pub struct DeleteTask(pub Uuid);
pub struct ListUsers(pub UserFilter);
pub struct UpdateUser(pub User);
pub struct DeleteUser(pub Uuid);
pub struct UpdateP2pNode(pub P2pNode);
pub struct DeleteP2pNode(pub Uuid, pub Uuid);
pub struct UpdateParticipant(pub Participant);
pub struct DeleteParticipant(pub Uuid);
pub struct ListParticipants(pub ParticipantFilter);
pub struct ListConversations(pub ConversationFilter);
