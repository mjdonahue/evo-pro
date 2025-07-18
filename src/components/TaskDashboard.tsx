/** @jsxImportSource react */
import React, { useState, useCallback } from 'react';
import { 
  useTasks, 
  useTaskStats, 
  useCreateTask, 
  useUpdateTaskStatus, 
  useStartTask,
  useCompleteTask,
  TaskStatus, 
  TaskPriority, 
  TaskUrgency, 
  TaskImportance,
  MemoryType,
  type Task,
  type CreateTaskInput,
  type TaskStats as ITaskStats,
  type Uuid
} from '../lib/api';

import { faker } from '@faker-js/faker';

interface TaskCardProps {
  task: Task;
  onStatusChange: (taskId: Uuid, status: TaskStatus) => void;
  onStart: (taskId: Uuid) => void;
  onComplete: (taskId: Uuid) => void;
}

const TaskCard: React.FC<TaskCardProps> = ({ task, onStatusChange, onStart, onComplete }) => {
  const getStatusColor = (status: TaskStatus) => {
    switch (status) {
      case TaskStatus.Pending: return 'bg-gray-100 text-gray-800';
      case TaskStatus.InProgress: return 'bg-blue-100 text-blue-800';
      case TaskStatus.Completed: return 'bg-green-100 text-green-800';
      case TaskStatus.Failed: return 'bg-red-100 text-red-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const getPriorityColor = (priority: TaskPriority) => {
    switch (priority) {
      case TaskPriority.Low: return 'text-green-600';
      case TaskPriority.Medium: return 'text-yellow-600';
      case TaskPriority.High: return 'text-red-600';
      default: return 'text-gray-600';
    }
  };

  return (
    <div className="bg-white rounded-lg shadow-md p-6 border border-gray-200">
      <div className="flex justify-between items-start mb-4">
        <h3 className="text-lg font-semibold text-gray-900">{task.title}</h3>
        <span className={`px-2 py-1 rounded-full text-sm font-medium ${getStatusColor(task.status)}`}>
          {task.status}
        </span>
      </div>
      
      {task.description && (
        <p className="text-gray-600 mb-4">{task.description}</p>
      )}
      
      <div className="flex items-center justify-between text-sm text-gray-500 mb-4">
        <div className="flex space-x-4">
          <span className={`font-medium ${getPriorityColor(task.priority)}`}>
            Priority: {task.priority}
          </span>
          <span>Urgency: {task.urgency}</span>
          <span>Importance: {task.importance}</span>
        </div>
        {task.due_date && (
          <span>Due: {new Date(task.due_date).toLocaleDateString()}</span>
        )}
      </div>
      
      <div className="flex space-x-2">
        {task.status === TaskStatus.Pending && (
          <button
            onClick={() => onStart(task.id)}
            className="px-3 py-1 bg-blue-600 text-white rounded text-sm hover:bg-blue-700"
          >
            Start
          </button>
        )}
        
        {task.status === TaskStatus.InProgress && (
          <button
            onClick={() => onComplete(task.id)}
            className="px-3 py-1 bg-green-600 text-white rounded text-sm hover:bg-green-700"
          >
            Complete
          </button>
        )}
        
        <select
          value={task.status}
          onChange={(e) => onStatusChange(task.id, e.target.value as TaskStatus)}
          className="px-2 py-1 border border-gray-300 rounded text-sm"
        >
          <option value={TaskStatus.Pending}>Pending</option>
          <option value={TaskStatus.InProgress}>In Progress</option>
          <option value={TaskStatus.Completed}>Completed</option>
          <option value={TaskStatus.Failed}>Failed</option>
        </select>
      </div>
    </div>
  );
};

interface CreateTaskFormProps {
  onSubmit: (task: CreateTaskInput) => void;
  onCancel: () => void;
}

const CreateTaskForm: React.FC<CreateTaskFormProps> = ({ onSubmit, onCancel }) => {
  const [formData, setFormData] = useState({
    title: '',
    description: '',
    priority: TaskPriority.Medium,
    urgency: TaskUrgency.Medium,
    importance: TaskImportance.Medium,
    due_date: '',
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    const task: CreateTaskInput = {
      plan_id: faker.string.uuid() as unknown as Uuid,
      participant_id: faker.string.uuid() as unknown as Uuid,
      workspace_id: faker.string.uuid() as unknown as Uuid,
      title: formData.title,
      description: formData.description || undefined,
      start_time: new Date().toISOString(),
      due_date: formData.due_date ? new Date(formData.due_date).toISOString() : undefined,
      priority: formData.priority,
      urgency: formData.urgency,
      importance: formData.importance,
      memory_type: MemoryType.Message,
      metadata: undefined,  
      end_time: undefined,
      conversation_id: undefined,
      memory_id: undefined,
      document_id: undefined,
      file_id: undefined,
      url: undefined,
      primary_assignee_id: undefined,
    };
    
    onSubmit(task);
  };

  return (
    <form onSubmit={handleSubmit} className="bg-white rounded-lg shadow-md p-6 border border-gray-200">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">Create New Task</h3>
      
      <div className="grid grid-cols-1 gap-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Title</label>
          <input
            type="text"
            value={formData.title}
            onChange={(e) => setFormData({ ...formData, title: e.target.value })}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            required
          />
        </div>
        
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
          <textarea
            value={formData.description}
            onChange={(e) => setFormData({ ...formData, description: e.target.value })}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            rows={3}
          />
        </div>
        
        <div className="grid grid-cols-3 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Priority</label>
            <select
              value={formData.priority}
              onChange={(e) => setFormData({ ...formData, priority: e.target.value as TaskPriority })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value={TaskPriority.Low}>Low</option>
              <option value={TaskPriority.Medium}>Medium</option>
              <option value={TaskPriority.High}>High</option>
            </select>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Urgency</label>
            <select
              value={formData.urgency}
              onChange={(e) => setFormData({ ...formData, urgency: e.target.value as TaskUrgency })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value={TaskUrgency.Low}>Low</option>
              <option value={TaskUrgency.Medium}>Medium</option>
              <option value={TaskUrgency.High}>High</option>
            </select>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Importance</label>
            <select
              value={formData.importance}
              onChange={(e) => setFormData({ ...formData, importance: e.target.value as TaskImportance })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value={TaskImportance.Low}>Low</option>
              <option value={TaskImportance.Medium}>Medium</option>
              <option value={TaskImportance.High}>High</option>
            </select>
          </div>
        </div>
        
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Due Date</label>
          <input
            type="datetime-local"
            value={formData.due_date}
            onChange={(e) => setFormData({ ...formData, due_date: e.target.value })}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
      </div>
      
      <div className="flex space-x-3 mt-6">
        <button
          type="submit"
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          Create Task
        </button>
        <button
          type="button"
          onClick={onCancel}
          className="px-4 py-2 border border-gray-300 text-gray-700 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          Cancel
        </button>
      </div>
    </form>
  );
};

const TaskStats: React.FC<{ stats: ITaskStats }> = ({ stats }) => (
  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
    <div className="bg-white rounded-lg shadow p-4">
      <div className="text-2xl font-bold text-blue-600">{stats.total_tasks}</div>
      <div className="text-sm text-gray-600">Total Tasks</div>
    </div>
    <div className="bg-white rounded-lg shadow p-4">
      <div className="text-2xl font-bold text-yellow-600">{stats.pending_tasks}</div>
      <div className="text-sm text-gray-600">Pending</div>
    </div>
    <div className="bg-white rounded-lg shadow p-4">
      <div className="text-2xl font-bold text-blue-600">{stats.in_progress_tasks}</div>
      <div className="text-sm text-gray-600">In Progress</div>
    </div>
    <div className="bg-white rounded-lg shadow p-4">
      <div className="text-2xl font-bold text-green-600">{stats.completed_tasks}</div>
      <div className="text-sm text-gray-600">Completed</div>
    </div>
  </div>
);

export const TaskDashboard: React.FC = () => {
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [statusFilter, setStatusFilter] = useState<TaskStatus | 'all'>('all');

  // API hooks
  const { data: tasks, loading: tasksLoading, error: tasksError, refetch: refetchTasks } = useTasks();
  const { data: stats, loading: statsLoading, refetch: refetchStats } = useTaskStats();
  const { mutate: createTask } = useCreateTask({
    onSuccess: () => {
      refetchTasks();
      refetchStats();
      setShowCreateForm(false);
    }
  });
  const { mutate: updateStatus } = useUpdateTaskStatus({
    onSuccess: () => {
      refetchTasks();
      refetchStats();
    }
  });
  const { mutate: startTask } = useStartTask({
    onSuccess: () => {
      refetchTasks();
      refetchStats();
    }
  });
  const { mutate: completeTask } = useCompleteTask({
    onSuccess: () => {
      refetchTasks();
      refetchStats();
    }
  });

  const handleStatusChange = useCallback((taskId: Uuid, status: TaskStatus) => {
    updateStatus({ id: taskId, status });
  }, [updateStatus]);

  const handleStartTask = useCallback((taskId: Uuid) => {
    startTask(taskId);
  }, [startTask]);

  const handleCompleteTask = useCallback((taskId: Uuid) => {
    completeTask(taskId);
  }, [completeTask]);

  const handleCreateTask = useCallback((task: CreateTaskInput) => {
    createTask(task);
  }, [createTask]);

  const filteredTasks = tasks?.filter(task => 
    statusFilter === 'all' || task.status === statusFilter
  ) || [];

  if (tasksError) {
    return (
      <div className="p-6 bg-red-50 border border-red-200 rounded-lg">
        <p className="text-red-800">Error loading tasks: {tasksError.message}</p>
      </div>
    );
  }

  return (
    <div className="max-w-6xl mx-auto p-6">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-3xl font-bold text-gray-900">Task Dashboard</h1>
        <button
          onClick={() => setShowCreateForm(true)}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          Create Task
        </button>
      </div>

      {/* Statistics */}
      {stats && <TaskStats stats={stats} />}

      {/* Filters */}
      <div className="flex space-x-4 mb-6">
        <select
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value as TaskStatus | 'all')}
          className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          <option value="all">All Tasks</option>
          <option value={TaskStatus.Pending}>Pending</option>
          <option value={TaskStatus.InProgress}>In Progress</option>
          <option value={TaskStatus.Completed}>Completed</option>
          <option value={TaskStatus.Failed}>Failed</option>
        </select>
      </div>

      {/* Create Task Form */}
      {showCreateForm && (
        <div className="mb-6">
          <CreateTaskForm
            onSubmit={handleCreateTask}
            onCancel={() => setShowCreateForm(false)}
          />
        </div>
      )}

      {/* Tasks Grid */}
      {tasksLoading ? (
        <div className="text-center py-12">
          <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          <p className="mt-2 text-gray-600">Loading tasks...</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {filteredTasks.map((task) => (
            <TaskCard
              key={task.id.toString()}
              task={task}
              onStatusChange={handleStatusChange}
              onStart={handleStartTask}
              onComplete={handleCompleteTask}
            />
          ))}
        </div>
      )}

      {filteredTasks.length === 0 && !tasksLoading && (
        <div className="text-center py-12">
          <p className="text-gray-500">No tasks found</p>
        </div>
      )}
    </div>
  );
}; 