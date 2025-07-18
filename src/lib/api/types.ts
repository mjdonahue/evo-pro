// Auto-generated types from Rust entities
// DO NOT EDIT MANUALLY - Use the type generation script

/**
 * Represents a UUID used for entity identification throughout the system.
 * This is a branded type to ensure type safety when working with IDs.
 */
export interface Uuid {
  readonly __brand: 'Uuid';
}

/**
 * Represents a date and time value in ISO 8601 format.
 * Example: "2023-01-01T12:00:00Z"
 */
export type DateTime = string; // ISO 8601 string

// Core Entity Types
/**
 * Represents a user in the system.
 * Users are the primary actors who can interact with the application.
 */
export interface User {
  id: Uuid;
  email?: string;
  username?: string;
  display_name: string;
  first_name?: string;
  last_name?: string;
  mobile_phone?: string;
  workspace_id: Uuid;
  participant_id: Uuid;
  avatar?: string;
  bio?: string;
  status: UserStatus;
  timezone?: string;
  language?: string;
  theme?: string;
  email_verified: boolean;
  phone_verified: boolean;
  last_seen: DateTime;
  roles: string; // JSON array
  preferences: string; // JSON object
  metadata?: string; // JSON object
  created_at: DateTime;
  updated_at: DateTime;
}

/**
 * Represents the possible statuses of a user in the system.
 */
export enum UserStatus {
  /** User is active and can use the system normally */
  Active = "Active",
  /** User is temporarily inactive */
  Inactive = "Inactive", 
  /** User has been deleted from the system */
  Deleted = "Deleted"
}

/**
 * Represents a conversation between participants.
 * Conversations can be of different types and contain messages.
 */
export interface Conversation {
  id: Uuid;
  title?: string;
  type: ConversationType;
  status: ConversationStatus;
  parent_conversation_id?: Uuid;
  metadata?: string; // JSON object
  created_at: DateTime;
  updated_at: DateTime;
}

export enum ConversationType {
  Private = "Private",
  Group = "Group",
  Channel = "Channel",
  System = "System"
}

export enum ConversationStatus {
  Active = "Active",
  Archived = "Archived",
  Deleted = "Deleted"
}

export interface Message {
  id: Uuid;
  conversation_id: Uuid;
  workspace_id: Uuid;
  sender_id: Uuid;
  parent_id?: Uuid;
  type: MessageType;
  content: string; // JSON object
  status: MessageStatus;
  refs?: string; // JSON array
  related_episode_id?: Uuid;
  branch_conversation_id?: Uuid;
  metadata?: string; // JSON object
  reply_to_id?: Uuid;
  created_at: DateTime;
  updated_at: DateTime;
}

export enum MessageType {
  Text = "Text",
  Command = "Command",
  System = "System",
  Error = "Error",
  Image = "Image",
  Audio = "Audio",
  Video = "Video",
  File = "File",
  Link = "Link"
}

export enum MessageStatus {
  Pending = "Pending",
  Sent = "Sent",
  Delivered = "Delivered",
  Read = "Read",
  Failed = "Failed"
}

// Task Management Types
export interface Plan {
  id: Uuid;
  participant_id: Uuid;
  plan_type: PlanType;
  plan_status: PlanStatus;
  plan_metadata?: string; // JSON
  created_at: DateTime;
  updated_at: DateTime;
}

export enum PlanType {
  Task = "Task",
  Goal = "Goal",
  Other = "Other"
}

export enum PlanStatus {
  Pending = "Pending",
  InProgress = "InProgress",
  Completed = "Completed",
  Failed = "Failed"
}

export interface Task {
  id: Uuid;
  plan_id: Uuid;
  participant_id: Uuid;
  workspace_id: Uuid;
  title: string;
  description?: string;
  start_time: DateTime;
  end_time?: DateTime;
  due_date?: DateTime;
  priority: TaskPriority;
  urgency: TaskUrgency;
  importance: TaskImportance;
  status: TaskStatus;
  metadata?: string; // JSON
  conversation_id?: Uuid;
  memory_id?: Uuid;
  memory_type: MemoryType;
  document_id?: Uuid;
  file_id?: Uuid;
  url?: string;
  primary_assignee_id?: Uuid;
  created_by_id: Uuid;
  created_at: DateTime;
  updated_at: DateTime;
}

export enum TaskPriority {
  Low = "Low",
  Medium = "Medium", 
  High = "High"
}

export enum TaskUrgency {
  Low = "Low",
  Medium = "Medium",
  High = "High"
}

export enum TaskImportance {
  Low = "Low",
  Medium = "Medium",
  High = "High"
}

export enum TaskStatus {
  Pending = "Pending",
  InProgress = "InProgress",
  Completed = "Completed",
  Failed = "Failed"
}

export enum MemoryType {
  Message = "Message",
  Memory = "Memory",
  Document = "Document",
  File = "File",
  Url = "Url"
}

export interface TaskAssignee {
  id: Uuid;
  workspace_id: Uuid;
  task_id: Uuid;
  participant_id: Uuid;
  role: TaskAssigneeRole;
  status: TaskAssigneeStatus;
  metadata?: string; // JSON
  created_at: DateTime;
  updated_at: DateTime;
}

export enum TaskAssigneeRole {
  Primary = "Primary",
  Secondary = "Secondary",
  Other = "Other"
}

export enum TaskAssigneeStatus {
  Pending = "Pending",
  InProgress = "InProgress", 
  Completed = "Completed",
  Failed = "Failed"
}

// Agent Types
export interface Agent {
  id: Uuid;
  workspace_id: Uuid;
  name: string;
  description?: string;
  agent_type: AgentType;
  status: AgentStatus;
  config?: string; // JSON object
  capabilities: string; // JSON array
  tools: string; // JSON array
  model_id?: Uuid;
  system_prompt?: string;
  temperature?: number;
  max_tokens?: number;
  metadata?: string; // JSON object
  created_at: DateTime;
  updated_at: DateTime;
}

export enum AgentType {
  Assistant = "Assistant",
  Specialist = "Specialist", 
  System = "System",
  Tool = "Tool",
  Other = "Other"
}

export enum AgentStatus {
  Active = "Active",
  Inactive = "Inactive",
  Training = "Training",
  Archived = "Archived",
  Deleted = "Deleted"
}

// Filter Types
export interface PaginationParams {
  limit?: number;
  offset?: number;
}

export interface ConversationFilter extends PaginationParams {
  status?: ConversationStatus;
  type?: ConversationType;
  user_id?: Uuid;
  agent_id?: Uuid;
  contact_id?: Uuid;
  search_term?: string;
}

export interface MessageFilter extends PaginationParams {
  conversation_id?: Uuid;
  sender_id?: Uuid;
  type?: MessageType;
  status?: MessageStatus;
  created_after?: DateTime;
  created_before?: DateTime;
}

export interface TaskFilter extends PaginationParams {
  plan_id?: Uuid;
  participant_id?: Uuid;
  workspace_id?: Uuid;
  status?: TaskStatus;
  priority?: TaskPriority;
  urgency?: TaskUrgency;
  importance?: TaskImportance;
  primary_assignee_id?: Uuid;
  created_by_id?: Uuid;
  conversation_id?: Uuid;
  memory_type?: MemoryType;
  active_only?: boolean;
  overdue_only?: boolean;
  due_today?: boolean;
  due_this_week?: boolean;
  high_priority_only?: boolean;
  urgent_only?: boolean;
  important_only?: boolean;
  created_after?: DateTime;
  created_before?: DateTime;
  due_after?: DateTime;
  due_before?: DateTime;
}

export interface PlanFilter extends PaginationParams {
  participant_id?: Uuid;
  plan_type?: PlanType;
  plan_status?: PlanStatus;
  active_only?: boolean;
  incomplete_only?: boolean;
  completed_only?: boolean;
  failed_only?: boolean;
  created_after?: DateTime;
  created_before?: DateTime;
  updated_after?: DateTime;
  updated_before?: DateTime;
}

// Statistics Types
export interface TaskStats {
  pending_tasks: number;
  in_progress_tasks: number;
  completed_tasks: number;
  failed_tasks: number;
  high_priority_tasks: number;
  urgent_tasks: number;
  important_tasks: number;
  overdue_tasks: number;
  total_tasks: number;
}

export interface PlanStats {
  pending_plans: number;
  in_progress_plans: number;
  completed_plans: number;
  failed_plans: number;
  task_plans: number;
  goal_plans: number;
  other_plans: number;
  total_plans: number;
}

// API Response Types
/**
 * Generic API response wrapper for all API calls.
 * @template T The type of data contained in the response
 */
export interface ApiResponse<T> {
  /** The response data of type T */
  data: T;
  /** Indicates whether the request was successful */
  success: boolean;
  /** Optional error message if the request failed */
  error?: string;
}

/**
 * Response structure for paginated list requests.
 * @template T The type of items in the list
 */
export interface ListResponse<T> {
  /** Array of items of type T */
  items: T[];
  /** Total number of items available (may be more than returned in this response) */
  total: number;
  /** Optional limit on the number of items returned */
  limit?: number;
  /** Optional offset for pagination */
  offset?: number;
}

// Command Input Types
export interface CreateConversationInput {
  title?: string;
  type: ConversationType;
  metadata?: Record<string, any>;
}

export interface CreateMessageInput {
  conversation_id: Uuid;
  content: Record<string, any>;
  type?: MessageType;
  parent_id?: Uuid;
  reply_to_id?: Uuid;
  metadata?: Record<string, any>;
}

export interface CreateTaskInput {
  plan_id: Uuid;
  participant_id: Uuid;
  workspace_id: Uuid;
  title: string;
  description?: string;
  start_time: DateTime;
  end_time?: DateTime;
  due_date?: DateTime;
  priority: TaskPriority;
  urgency: TaskUrgency;
  importance: TaskImportance;
  conversation_id?: Uuid;
  memory_id?: Uuid;
  memory_type: MemoryType;
  document_id?: Uuid;
  file_id?: Uuid;
  url?: string;
  primary_assignee_id?: Uuid;
  metadata?: Record<string, any>;
}

export interface CreatePlanInput {
  participant_id: Uuid;
  plan_type: PlanType;
  plan_metadata?: Record<string, any>;
}

export interface UpdateTaskStatusInput {
  id: Uuid;
  status: TaskStatus;
}

export interface UpdatePlanStatusInput {
  id: Uuid; 
  status: PlanStatus;
}

// Error Types
/**
 * Represents an error returned by the API.
 * Contains information about what went wrong and additional details.
 */
export interface ApiError {
  /** Error code identifying the type of error */
  code: string;
  /** Human-readable error message */
  message: string;
  /** Optional additional details about the error */
  details?: Record<string, any>;
} 
