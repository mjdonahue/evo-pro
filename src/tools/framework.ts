import { z } from 'zod';
import { Tool } from '../db/types';

// Core interfaces
export interface ToolContext {
  userId: string;
  agentId?: string;
  conversationId?: string;
  workflowId?: string;
  executionId?: string;
  env: Record<string, string>;
}

export interface ToolResult<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
  metrics?: {
    startTime: number;
    endTime: number;
    duration: number;
    retryCount: number;
  };
}

// Tool execution options
export interface ToolExecutionOptions {
  timeout?: number;
  retryPolicy?: {
    maxAttempts: number;
    backoffMs: number;
  };
  fallbackToCloud?: boolean;
  useCache?: boolean;
  validateInput?: boolean;
  validateOutput?: boolean;
}

// Base tool implementation
export abstract class BaseTool<TInput = unknown, TOutput = unknown> {
  protected tool: Tool;
  protected inputSchema: z.ZodType<TInput>;
  protected outputSchema: z.ZodType<TOutput>;

  constructor(
    tool: Tool,
    inputSchema: z.ZodType<TInput>,
    outputSchema: z.ZodType<TOutput>
  ) {
    this.tool = tool;
    this.inputSchema = inputSchema;
    this.outputSchema = outputSchema;
  }

  // Core execution method
  async execute(
    input: TInput,
    context: ToolContext,
    options: ToolExecutionOptions = {}
  ): Promise<ToolResult<TOutput>> {
    const startTime = Date.now();
    let retryCount = 0;

    try {
      // Validate input if enabled
      if (options.validateInput) {
        this.inputSchema.parse(input);
      }

      // Check permissions
      await this.checkPermissions(context);

      // Try local execution first
      let result = await this.executeLocally(input, context);

      // Fall back to cloud if enabled and local execution failed
      if (!result.success && options.fallbackToCloud) {
        result = await this.executeInCloud(input, context);
      }

      // Validate output if enabled
      if (options.validateOutput && result.success && result.data) {
        this.outputSchema.parse(result.data);
      }

      return {
        ...result,
        metrics: {
          startTime,
          endTime: Date.now(),
          duration: Date.now() - startTime,
          retryCount
        }
      };

    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        metrics: {
          startTime,
          endTime: Date.now(),
          duration: Date.now() - startTime,
          retryCount
        }
      };
    }
  }

  // Abstract methods to be implemented by specific tools
  protected abstract executeLocally(
    input: TInput,
    context: ToolContext
  ): Promise<ToolResult<TOutput>>;

  protected abstract executeInCloud(
    input: TInput,
    context: ToolContext
  ): Promise<ToolResult<TOutput>>;

  // Permission checking
  protected async checkPermissions(context: ToolContext): Promise<void> {
    // Check if user has required roles
    if (this.tool.permissions.roles.length > 0) {
      const hasRole = await this.userHasRole(context.userId, this.tool.permissions.roles);
      if (!hasRole) {
        throw new Error('User does not have required roles');
      }
    }

    // Check if user/agent has required capabilities
    if (this.tool.permissions.capabilities.length > 0) {
      const hasCapabilities = await this.hasCapabilities(
        context.userId,
        context.agentId,
        this.tool.permissions.capabilities
      );
      if (!hasCapabilities) {
        throw new Error('Missing required capabilities');
      }
    }
  }

  // Helper methods
  protected async userHasRole(_userId: string, _roles: string[]): Promise<boolean> {
    // Implementation to check user roles against required roles
    return true; // TODO: Implement actual role checking
  }

  protected async hasCapabilities(
    _userId: string,
    _agentId: string | undefined,
    _capabilities: string[]
  ): Promise<boolean> {
    // Implementation to check capabilities
    return true; // TODO: Implement actual capability checking
  }

  // Metadata methods
  getMetadata() {
    return {
      id: this.tool.id,
      name: this.tool.name,
      description: this.tool.description,
      type: this.tool.type,
      category: this.tool.category,
      version: this.tool.version,
      status: this.tool.status,
      configuration: this.tool.configuration,
      permissions: this.tool.permissions,
    };
  }
}

// Tool registry for discovery
export class ToolRegistry {
  private static instance: ToolRegistry;
  private tools: Map<string, BaseTool> = new Map();

  private constructor() {}

  static getInstance(): ToolRegistry {
    if (!ToolRegistry.instance) {
      ToolRegistry.instance = new ToolRegistry();
    }
    return ToolRegistry.instance;
  }

  registerTool(tool: BaseTool) {
    this.tools.set(tool.getMetadata().id, tool);
  }

  getTool(id: string): BaseTool | undefined {
    return this.tools.get(id);
  }

  listTools(filter?: {
    type?: string;
    category?: string;
    status?: string;
  }): BaseTool[] {
    let tools = Array.from(this.tools.values());

    if (filter) {
      tools = tools.filter(tool => {
        const metadata = tool.getMetadata();
        return (
          (!filter.type || metadata.type === filter.type) &&
          (!filter.category || metadata.category === filter.category) &&
          (!filter.status || metadata.status === filter.status)
        );
      });
    }

    return tools;
  }
}

// Tool decorator for easy registration
export function registerTool() {
  return function (constructor: new (...args: any[]) => BaseTool) {
    // @ts-expect-error - Anonymous class inherits abstract methods, but TS requires explicit implementation here.
    return class extends constructor {
      constructor(...args: any[]) {
        super(...args);
        ToolRegistry.getInstance().registerTool(this);
      }
    };
  };
} 
