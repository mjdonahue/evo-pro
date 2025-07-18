import { useState, useCallback } from 'react';
import { ToolRegistry, ToolContext, ToolResult, ToolExecutionOptions } from './framework';
import { useUserProfile } from '../db/hooks';

export interface UseToolOptions extends ToolExecutionOptions {
  context?: Partial<ToolContext>;
}

export function useTool<TInput, TOutput>(toolId: string, options: UseToolOptions = {}) {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastResult, setLastResult] = useState<ToolResult<TOutput> | null>(null);

  const userProfile = useUserProfile(options.context?.userId || '');

  const execute = useCallback(
    async (input: TInput) => {
      setIsLoading(true);
      setError(null);

      try {
        const tool = ToolRegistry.getInstance().getTool(toolId);
        if (!tool) {
          throw new Error(`Tool not found: ${toolId}`);
        }

        // Build execution context
        const context: ToolContext = {
          userId: options.context?.userId || userProfile?.id || '',
          agentId: options.context?.agentId,
          conversationId: options.context?.conversationId,
          workflowId: options.context?.workflowId,
          executionId: options.context?.executionId,
          env: options.context?.env || {},
        };

        // Execute tool
        const result = await tool.execute(input, context, options);
        setLastResult(result as ToolResult<TOutput>);

        if (!result.success) {
          setError(result.error || 'Tool execution failed');
        }

        return result;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Unknown error';
        setError(errorMessage);
        return {
          success: false,
          error: errorMessage,
        } as ToolResult<TOutput>;
      } finally {
        setIsLoading(false);
      }
    },
    [toolId, options, userProfile]
  );

  return {
    execute,
    isLoading,
    error,
    lastResult,
    reset: useCallback(() => {
      setError(null);
      setLastResult(null);
    }, []),
  };
}

// Example usage:
/*
function MyComponent() {
  const webSearch = useTool<WebSearchInputType, WebSearchOutputType>('web-search', {
    fallbackToCloud: true,
    validateInput: true,
    validateOutput: true,
    context: {
      env: {
        API_KEY: process.env.SEARCH_API_KEY,
      },
    },
  });

  const handleSearch = async (query: string) => {
    const result = await webSearch.execute({
      query,
      maxResults: 5,
    });

    if (result.success && result.data) {
      // Handle search results
      console.log(result.data.results);
    }
  };

  return (
    <div>
      {webSearch.isLoading && <div>Searching...</div>}
      {webSearch.error && <div>Error: {webSearch.error}</div>}
      <button onClick={() => handleSearch('example query')}>Search</button>
    </div>
  );
}
*/ 
