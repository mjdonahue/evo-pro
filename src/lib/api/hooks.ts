import { useState, useEffect, useCallback, useRef } from 'react';
import { apiClient, ApiClientError } from './client';
import type {
  Conversation,
  ConversationFilter,
  CreateConversationInput,
  Message,
  MessageFilter,
  CreateMessageInput,
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
  ListResponse,
  Uuid,
} from './types';

// Base hook interfaces
export interface UseQueryResult<T> {
  data: T | null;
  loading: boolean;
  error: ApiClientError | null;
  refetch: () => Promise<void>;
}

export interface UseListQueryResult<T> {
  data: T[];
  total: number;
  loading: boolean;
  error: ApiClientError | null;
  refetch: () => Promise<void>;
  hasMore: boolean;
  loadMore: () => Promise<void>;
}

export interface UseMutationResult<T, TVariables = any> {
  mutate: (variables: TVariables) => Promise<T>;
  loading: boolean;
  error: ApiClientError | null;
  data: T | null;
  reset: () => void;
}

// Base query hook
function useQuery<T>(
  queryFn: () => Promise<T>,
  dependencies: any[] = [],
  options: { enabled?: boolean; refetchInterval?: number } = {}
): UseQueryResult<T> {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<ApiClientError | null>(null);
  const intervalRef = useRef<NodeJS.Timeout>();

  const fetchData = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await queryFn();
      setData(result);
    } catch (err) {
      setError(err instanceof ApiClientError ? err : new ApiClientError('unknown', 'Unknown error'));
    } finally {
      setLoading(false);
    }
  }, dependencies);

  const refetch = useCallback(async () => {
    await fetchData();
  }, [fetchData]);

  useEffect(() => {
    if (options.enabled !== false) {
      fetchData();
    }
  }, [fetchData, options.enabled]);

  useEffect(() => {
    if (options.refetchInterval && options.refetchInterval > 0) {
      intervalRef.current = setInterval(fetchData, options.refetchInterval);
      return () => {
        if (intervalRef.current) {
          clearInterval(intervalRef.current);
        }
      };
    }
  }, [fetchData, options.refetchInterval]);

  useEffect(() => {
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, []);

  return { data, loading, error, refetch };
}

// Base list query hook with pagination
function useListQuery<T>(
  queryFn: (offset: number, limit: number) => Promise<ListResponse<T>>,
  dependencies: any[] = [],
  options: { enabled?: boolean; pageSize?: number } = {}
): UseListQueryResult<T> {
  const [data, setData] = useState<T[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<ApiClientError | null>(null);
  const [offset, setOffset] = useState(0);
  const pageSize = options.pageSize || 20;

  const fetchData = useCallback(async (resetData = true) => {
    try {
      setLoading(true);
      setError(null);
      const currentOffset = resetData ? 0 : offset;
      const result = await queryFn(currentOffset, pageSize);
      
      if (resetData) {
        setData(result.items);
        setOffset(pageSize);
      } else {
        setData(prev => [...prev, ...result.items]);
        setOffset(prev => prev + pageSize);
      }
      
      setTotal(result.total);
    } catch (err) {
      setError(err instanceof ApiClientError ? err : new ApiClientError('unknown', 'Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [...dependencies, offset, pageSize]);

  const refetch = useCallback(async () => {
    setOffset(0);
    await fetchData(true);
  }, [fetchData]);

  const loadMore = useCallback(async () => {
    if (data.length < total) {
      await fetchData(false);
    }
  }, [fetchData, data.length, total]);

  useEffect(() => {
    if (options.enabled !== false) {
      fetchData(true);
    }
  }, dependencies);

  const hasMore = data.length < total;

  return { data, total, loading, error, refetch, hasMore, loadMore };
}

// Base mutation hook
function useMutation<T, TVariables = any>(
  mutationFn: (variables: TVariables) => Promise<T>,
  options: { onSuccess?: (data: T) => void; onError?: (error: ApiClientError) => void } = {}
): UseMutationResult<T, TVariables> {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<ApiClientError | null>(null);
  const [data, setData] = useState<T | null>(null);

  const mutate = useCallback(async (variables: TVariables): Promise<T> => {
    try {
      setLoading(true);
      setError(null);
      const result = await mutationFn(variables);
      setData(result);
      options.onSuccess?.(result);
      return result;
    } catch (err) {
      const apiError = err instanceof ApiClientError ? err : new ApiClientError('unknown', 'Unknown error');
      setError(apiError);
      options.onError?.(apiError);
      throw apiError;
    } finally {
      setLoading(false);
    }
  }, [mutationFn, options.onSuccess, options.onError]);

  const reset = useCallback(() => {
    setLoading(false);
    setError(null);
    setData(null);
  }, []);

  return { mutate, loading, error, data, reset };
}

// Conversation hooks
export function useConversations(filter?: ConversationFilter, options?: { enabled?: boolean }) {
  return useListQuery(
    (offset, limit) => apiClient.conversations.list({ ...filter, offset, limit }),
    [filter],
    options
  );
}

export function useConversation(id: Uuid | null, options?: { enabled?: boolean }) {
  return useQuery(
    () => id ? apiClient.conversations.get(id) : Promise.resolve(null),
    [id],
    { enabled: Boolean(id) && options?.enabled !== false }
  );
}

export function useCreateConversation(options?: { onSuccess?: (data: Conversation) => void }) {
  return useMutation(
    (input: CreateConversationInput) => apiClient.conversations.create(input),
    options
  );
}

export function useUpdateConversation(options?: { onSuccess?: (data: Conversation) => void }) {
  return useMutation(
    (conversation: Conversation) => apiClient.conversations.update(conversation),
    options
  );
}

export function useDeleteConversation(options?: { onSuccess?: () => void }) {
  return useMutation(
    (id: Uuid) => apiClient.conversations.delete(id),
    options
  );
}

// Message hooks
export function useMessages(filter?: MessageFilter, options?: { enabled?: boolean }) {
  return useListQuery(
    (offset, limit) => apiClient.messages.list({ ...filter, offset, limit }),
    [filter],
    options
  );
}

export function useMessage(id: Uuid | null, options?: { enabled?: boolean }) {
  return useQuery(
    () => id ? apiClient.messages.get(id) : Promise.resolve(null),
    [id],
    { enabled: Boolean(id) && options?.enabled !== false }
  );
}

export function useCreateMessage(options?: { onSuccess?: (data: Message) => void }) {
  return useMutation(
    (input: CreateMessageInput) => apiClient.messages.create(input),
    options
  );
}

export function useMarkMessageRead(options?: { onSuccess?: () => void }) {
  return useMutation(
    (id: Uuid) => apiClient.messages.markAsRead(id),
    options
  );
}

// Task hooks
export function useTasks(filter?: TaskFilter, options?: { enabled?: boolean }) {
  return useQuery(
    () => apiClient.tasks.list(filter),
    [filter],
    options
  );
}

export function useTask(id: Uuid | null, options?: { enabled?: boolean }) {
  return useQuery(
    () => id ? apiClient.tasks.get(id) : Promise.resolve(null),
    [id],
    { enabled: Boolean(id) && options?.enabled !== false }
  );
}

export function useCreateTask(options?: { onSuccess?: (data: Task) => void }) {
  return useMutation(
    (input: CreateTaskInput) => apiClient.tasks.create(input),
    options
  );
}

export function useUpdateTask(options?: { onSuccess?: (data: Task) => void }) {
  return useMutation(
    (task: Task) => apiClient.tasks.update(task),
    options
  );
}

export function useUpdateTaskStatus(options?: { onSuccess?: () => void }) {
  return useMutation(
    (input: UpdateTaskStatusInput) => apiClient.tasks.updateStatus(input),
    options
  );
}

export function useStartTask(options?: { onSuccess?: () => void }) {
  return useMutation(
    (id: Uuid) => apiClient.tasks.start(id),
    options
  );
}

export function useCompleteTask(options?: { onSuccess?: () => void }) {
  return useMutation(
    (id: Uuid) => apiClient.tasks.complete(id),
    options
  );
}

export function useTaskStats(workspaceId?: Uuid, options?: { enabled?: boolean; refetchInterval?: number }) {
  return useQuery(
    () => apiClient.tasks.getStats(workspaceId),
    [workspaceId],
    options
  );
}

export function useOverdueTasks(options?: { enabled?: boolean }) {
  return useQuery(
    () => apiClient.tasks.getOverdue(),
    [],
    options
  );
}

export function useHighPriorityTasks(options?: { enabled?: boolean }) {
  return useQuery(
    () => apiClient.tasks.getHighPriority(),
    [],
    options
  );
}

export function useTasksByPlan(planId: Uuid | null, options?: { enabled?: boolean }) {
  return useQuery(
    () => planId ? apiClient.tasks.getByPlan(planId) : Promise.resolve([]),
    [planId],
    { enabled: Boolean(planId) && options?.enabled !== false }
  );
}

// Plan hooks
export function usePlans(filter?: PlanFilter, options?: { enabled?: boolean }) {
  return useListQuery(
    (offset, limit) => apiClient.plans.list({ ...filter, offset, limit }),
    [filter],
    options
  );
}

export function usePlan(id: Uuid | null, options?: { enabled?: boolean }) {
  return useQuery(
    () => id ? apiClient.plans.get(id) : Promise.resolve(null),
    [id],
    { enabled: Boolean(id) && options?.enabled !== false }
  );
}

export function useCreatePlan(options?: { onSuccess?: (data: Plan) => void }) {
  return useMutation(
    (input: CreatePlanInput) => apiClient.plans.create(input),
    options
  );
}

export function useUpdatePlan(options?: { onSuccess?: (data: Plan) => void }) {
  return useMutation(
    (plan: Plan) => apiClient.plans.update(plan),
    options
  );
}

export function useUpdatePlanStatus(options?: { onSuccess?: () => void }) {
  return useMutation(
    (input: UpdatePlanStatusInput) => apiClient.plans.updateStatus(input),
    options
  );
}

export function usePlanStats(participantId?: Uuid, options?: { enabled?: boolean; refetchInterval?: number }) {
  return useQuery(
    () => apiClient.plans.getStats(participantId),
    [participantId],
    options
  );
}

export function usePlansByParticipant(participantId: Uuid | null, options?: { enabled?: boolean }) {
  return useQuery(
    () => participantId ? apiClient.plans.getByParticipant(participantId) : Promise.resolve([]),
    [participantId],
    { enabled: Boolean(participantId) && options?.enabled !== false }
  );
}

export function useActivePlansByParticipant(participantId: Uuid | null, options?: { enabled?: boolean }) {
  return useQuery(
    () => participantId ? apiClient.plans.getActive(participantId) : Promise.resolve([]),
    [participantId],
    { enabled: Boolean(participantId) && options?.enabled !== false }
  );
}

// Task Assignee hooks
export function useTaskAssignees(taskId: Uuid | null, options?: { enabled?: boolean }) {
  return useQuery(
    () => taskId ? apiClient.taskAssignees.getByTask(taskId) : Promise.resolve([]),
    [taskId],
    { enabled: Boolean(taskId) && options?.enabled !== false }
  );
}

export function useAssigneeTasks(participantId: Uuid | null, options?: { enabled?: boolean }) {
  return useQuery(
    () => participantId ? apiClient.taskAssignees.getByParticipant(participantId) : Promise.resolve([]),
    [participantId],
    { enabled: Boolean(participantId) && options?.enabled !== false }
  );
}

export function useAddTaskAssignee(options?: { onSuccess?: (data: TaskAssignee) => void }) {
  return useMutation(
    ({ taskId, participantId, role }: { taskId: Uuid; participantId: Uuid; role: string }) =>
      apiClient.taskAssignees.addAssignee(taskId, participantId, role),
    options
  );
}

export function useRemoveTaskAssignee(options?: { onSuccess?: () => void }) {
  return useMutation(
    ({ taskId, participantId }: { taskId: Uuid; participantId: Uuid }) =>
      apiClient.taskAssignees.removeAssignee(taskId, participantId),
    options
  );
}

// Real-time event hooks
export function useConversationEvents(callback?: (event: any) => void) {
  useEffect(() => {
    if (!callback) return;

    const setupListener = async () => {
      const unlisten = await apiClient.events.onConversationEvents(callback);
      return unlisten;
    };

    let unlisten: (() => void) | null = null;
    setupListener().then(fn => { unlisten = fn; });

    return () => {
      unlisten?.();
    };
  }, [callback]);
}

export function useMessageEvents(callback?: (event: any) => void) {
  useEffect(() => {
    if (!callback) return;

    const setupListener = async () => {
      const unlisten = await apiClient.events.onMessageEvents(callback);
      return unlisten;
    };

    let unlisten: (() => void) | null = null;
    setupListener().then(fn => { unlisten = fn; });

    return () => {
      unlisten?.();
    };
  }, [callback]);
}

export function useTaskEvents(callback?: (event: any) => void) {
  useEffect(() => {
    if (!callback) return;

    const setupListener = async () => {
      const unlisten = await apiClient.events.onTaskEvents(callback);
      return unlisten;
    };

    let unlisten: (() => void) | null = null;
    setupListener().then(fn => { unlisten = fn; });

    return () => {
      unlisten?.();
    };
  }, [callback]);
} 