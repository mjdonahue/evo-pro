import { BaseService } from './baseService';
import type {
  Task,
  TaskFilter,
  CreateTaskInput,
  UpdateTaskStatusInput,
  TaskStats,
  Plan,
  PlanFilter,
  CreatePlanInput,
  UpdatePlanStatusInput,
  PlanStats,
  TaskAssignee,
  Uuid,
  ListResponse
} from '../api/types';

/**
 * Service for managing tasks, plans, and task assignments.
 * Provides higher-level abstractions for task-related operations.
 */
export class TaskService extends BaseService {
  // Task operations
  
  /**
   * Retrieves a list of tasks based on filter criteria
   * @param filter - Filter criteria for tasks
   * @returns A promise that resolves to a list of tasks
   */
  async getTasks(filter?: TaskFilter): Promise<Task[]> {
    try {
      return await this.api.tasks.list(filter);
    } catch (error) {
      this.handleError(error, { operation: 'getTasks', filter });
    }
  }

  /**
   * Retrieves a single task by ID
   * @param id - The ID of the task to retrieve
   * @returns A promise that resolves to the task or null if not found
   */
  async getTask(id: Uuid): Promise<Task | null> {
    try {
      return await this.api.tasks.get(id);
    } catch (error) {
      this.handleError(error, { operation: 'getTask', id });
    }
  }

  /**
   * Creates a new task
   * @param input - The data for the new task
   * @returns A promise that resolves to the created task
   */
  async createTask(input: CreateTaskInput): Promise<Task> {
    try {
      return await this.api.tasks.create(input);
    } catch (error) {
      this.handleError(error, { operation: 'createTask', input });
    }
  }

  /**
   * Updates an existing task
   * @param task - The updated task data
   * @returns A promise that resolves to the updated task
   */
  async updateTask(task: Task): Promise<Task> {
    try {
      return await this.api.tasks.update(task);
    } catch (error) {
      this.handleError(error, { operation: 'updateTask', task });
    }
  }

  /**
   * Deletes a task
   * @param id - The ID of the task to delete
   * @returns A promise that resolves when the task is deleted
   */
  async deleteTask(id: Uuid): Promise<void> {
    try {
      await this.api.tasks.delete(id);
    } catch (error) {
      this.handleError(error, { operation: 'deleteTask', id });
    }
  }

  /**
   * Updates the status of a task
   * @param id - The ID of the task
   * @param status - The new status
   * @returns A promise that resolves when the status is updated
   */
  async updateTaskStatus(id: Uuid, status: string): Promise<void> {
    try {
      await this.api.tasks.updateStatus({ id, status: status as any });
    } catch (error) {
      this.handleError(error, { operation: 'updateTaskStatus', id, status });
    }
  }

  /**
   * Starts a task (changes status to InProgress)
   * @param id - The ID of the task
   * @returns A promise that resolves when the task is started
   */
  async startTask(id: Uuid): Promise<void> {
    try {
      await this.api.tasks.start(id);
    } catch (error) {
      this.handleError(error, { operation: 'startTask', id });
    }
  }

  /**
   * Completes a task (changes status to Completed)
   * @param id - The ID of the task
   * @returns A promise that resolves when the task is completed
   */
  async completeTask(id: Uuid): Promise<void> {
    try {
      await this.api.tasks.complete(id);
    } catch (error) {
      this.handleError(error, { operation: 'completeTask', id });
    }
  }

  /**
   * Fails a task (changes status to Failed)
   * @param id - The ID of the task
   * @returns A promise that resolves when the task is failed
   */
  async failTask(id: Uuid): Promise<void> {
    try {
      await this.api.tasks.fail(id);
    } catch (error) {
      this.handleError(error, { operation: 'failTask', id });
    }
  }

  /**
   * Retrieves task statistics
   * @param workspaceId - Optional workspace ID to filter by
   * @returns A promise that resolves to task statistics
   */
  async getTaskStats(workspaceId?: Uuid): Promise<TaskStats> {
    try {
      return await this.api.tasks.getStats(workspaceId);
    } catch (error) {
      this.handleError(error, { operation: 'getTaskStats', workspaceId });
    }
  }

  /**
   * Retrieves overdue tasks
   * @returns A promise that resolves to a list of overdue tasks
   */
  async getOverdueTasks(): Promise<Task[]> {
    try {
      return await this.api.tasks.getOverdue();
    } catch (error) {
      this.handleError(error, { operation: 'getOverdueTasks' });
    }
  }

  /**
   * Retrieves high priority tasks
   * @returns A promise that resolves to a list of high priority tasks
   */
  async getHighPriorityTasks(): Promise<Task[]> {
    try {
      return await this.api.tasks.getHighPriority();
    } catch (error) {
      this.handleError(error, { operation: 'getHighPriorityTasks' });
    }
  }

  // Plan operations

  /**
   * Retrieves a list of plans based on filter criteria
   * @param filter - Filter criteria for plans
   * @returns A promise that resolves to a list of plans
   */
  async getPlans(filter?: PlanFilter): Promise<ListResponse<Plan>> {
    try {
      return await this.api.plans.list(filter);
    } catch (error) {
      this.handleError(error, { operation: 'getPlans', filter });
    }
  }

  /**
   * Retrieves a single plan by ID
   * @param id - The ID of the plan to retrieve
   * @returns A promise that resolves to the plan or null if not found
   */
  async getPlan(id: Uuid): Promise<Plan | null> {
    try {
      return await this.api.plans.get(id);
    } catch (error) {
      this.handleError(error, { operation: 'getPlan', id });
    }
  }

  /**
   * Creates a new plan
   * @param input - The data for the new plan
   * @returns A promise that resolves to the created plan
   */
  async createPlan(input: CreatePlanInput): Promise<Plan> {
    try {
      return await this.api.plans.create(input);
    } catch (error) {
      this.handleError(error, { operation: 'createPlan', input });
    }
  }

  /**
   * Updates an existing plan
   * @param plan - The updated plan data
   * @returns A promise that resolves to the updated plan
   */
  async updatePlan(plan: Plan): Promise<Plan> {
    try {
      return await this.api.plans.update(plan);
    } catch (error) {
      this.handleError(error, { operation: 'updatePlan', plan });
    }
  }

  /**
   * Deletes a plan
   * @param id - The ID of the plan to delete
   * @returns A promise that resolves when the plan is deleted
   */
  async deletePlan(id: Uuid): Promise<void> {
    try {
      await this.api.plans.delete(id);
    } catch (error) {
      this.handleError(error, { operation: 'deletePlan', id });
    }
  }

  /**
   * Updates the status of a plan
   * @param id - The ID of the plan
   * @param status - The new status
   * @returns A promise that resolves when the status is updated
   */
  async updatePlanStatus(id: Uuid, status: string): Promise<void> {
    try {
      await this.api.plans.updateStatus({ id, status: status as any });
    } catch (error) {
      this.handleError(error, { operation: 'updatePlanStatus', id, status });
    }
  }

  /**
   * Retrieves plan statistics
   * @param participantId - Optional participant ID to filter by
   * @returns A promise that resolves to plan statistics
   */
  async getPlanStats(participantId?: Uuid): Promise<PlanStats> {
    try {
      return await this.api.plans.getStats(participantId);
    } catch (error) {
      this.handleError(error, { operation: 'getPlanStats', participantId });
    }
  }

  /**
   * Retrieves tasks for a plan
   * @param planId - The ID of the plan
   * @returns A promise that resolves to a list of tasks
   */
  async getTasksByPlan(planId: Uuid): Promise<Task[]> {
    try {
      return await this.api.tasks.getByPlan(planId);
    } catch (error) {
      this.handleError(error, { operation: 'getTasksByPlan', planId });
    }
  }

  // Task Assignee operations

  /**
   * Retrieves assignees for a task
   * @param taskId - The ID of the task
   * @returns A promise that resolves to a list of task assignees
   */
  async getTaskAssignees(taskId: Uuid): Promise<TaskAssignee[]> {
    try {
      return await this.api.taskAssignees.getByTask(taskId);
    } catch (error) {
      this.handleError(error, { operation: 'getTaskAssignees', taskId });
    }
  }

  /**
   * Retrieves tasks assigned to a participant
   * @param participantId - The ID of the participant
   * @returns A promise that resolves to a list of task assignees
   */
  async getAssignedTasks(participantId: Uuid): Promise<TaskAssignee[]> {
    try {
      return await this.api.taskAssignees.getByParticipant(participantId);
    } catch (error) {
      this.handleError(error, { operation: 'getAssignedTasks', participantId });
    }
  }

  /**
   * Assigns a task to a participant
   * @param taskId - The ID of the task
   * @param participantId - The ID of the participant
   * @param role - The role of the assignee
   * @returns A promise that resolves to the created task assignee
   */
  async assignTask(taskId: Uuid, participantId: Uuid, role: string): Promise<TaskAssignee> {
    try {
      return await this.api.taskAssignees.addAssignee(taskId, participantId, role);
    } catch (error) {
      this.handleError(error, { operation: 'assignTask', taskId, participantId, role });
    }
  }

  /**
   * Unassigns a task from a participant
   * @param taskId - The ID of the task
   * @param participantId - The ID of the participant
   * @returns A promise that resolves when the assignee is removed
   */
  async unassignTask(taskId: Uuid, participantId: Uuid): Promise<void> {
    try {
      await this.api.taskAssignees.removeAssignee(taskId, participantId);
    } catch (error) {
      this.handleError(error, { operation: 'unassignTask', taskId, participantId });
    }
  }

  /**
   * Creates a plan with an initial task
   * @param participantId - The ID of the participant
   * @param planType - The type of plan
   * @param taskTitle - The title of the initial task
   * @param taskDetails - Additional details for the task
   * @returns A promise that resolves to an object containing the created plan and task
   */
  async createPlanWithTask(
    participantId: Uuid,
    planType: string,
    taskTitle: string,
    taskDetails?: {
      description?: string;
      priority?: string;
      urgency?: string;
      importance?: string;
      dueDate?: string;
      metadata?: Record<string, any>;
    }
  ): Promise<{ plan: Plan; task: Task }> {
    try {
      // Create the plan
      const plan = await this.api.plans.create({
        participant_id: participantId,
        plan_type: planType as any,
        plan_metadata: taskDetails?.metadata
      });

      // Create the initial task
      const task = await this.api.tasks.create({
        plan_id: plan.id,
        participant_id: participantId,
        workspace_id: participantId, // Assuming workspace_id is the same as participant_id
        title: taskTitle,
        description: taskDetails?.description,
        start_time: new Date().toISOString(),
        due_date: taskDetails?.dueDate,
        priority: (taskDetails?.priority || 'Medium') as any,
        urgency: (taskDetails?.urgency || 'Medium') as any,
        importance: (taskDetails?.importance || 'Medium') as any,
        memory_type: 'Memory' // Default memory type
      });

      return { plan, task };
    } catch (error) {
      this.handleError(error, { operation: 'createPlanWithTask', participantId, planType, taskTitle, taskDetails });
    }
  }
}

// Singleton instance
export const taskService = new TaskService();