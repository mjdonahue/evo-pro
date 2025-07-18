import { z } from 'zod';
import { BaseTool, ToolContext, ToolResult, registerTool } from './framework';
import { Tool } from '../db/types';
import { Command } from '@tauri-apps/plugin-shell';
import { writeTextFile, create, remove } from '@tauri-apps/plugin-fs';
import * as path from 'path';
import { generateId } from '../lib/utils';

// Define supported languages and their configurations
const LANGUAGE_CONFIGS = {
  python: {
    fileExtension: '.py',
    command: 'python',
    defaultTimeout: 30000,
  },
  node: {
    fileExtension: '.js',
    command: 'node',
    defaultTimeout: 30000,
  },
  typescript: {
    fileExtension: '.ts',
    command: 'ts-node',
    defaultTimeout: 30000,
  },
  shell: {
    fileExtension: '.sh',
    command: 'bash',
    defaultTimeout: 30000,
  },
} as const;

// Define input/output schemas
const CodeExecutionInput = z.object({
  code: z.string(),
  language: z.enum(['python', 'node', 'typescript', 'shell']),
  timeout: z.number().optional(),
  args: z.array(z.string()).optional(),
  env: z.record(z.string()).optional(),
  workingDir: z.string().optional(),
});

const CodeExecutionOutput = z.object({
  stdout: z.string(),
  stderr: z.string(),
  exitCode: z.number(),
  executionTime: z.number(),
  language: z.enum(['python', 'node', 'typescript', 'shell']),
  tempFile: z.string().optional(),
});

type CodeExecutionInputType = z.infer<typeof CodeExecutionInput>;
type CodeExecutionOutputType = z.infer<typeof CodeExecutionOutput>;

// @ts-expect-error - Decorator return type mismatch, likely external issue
@registerTool()
class _CodeExecutionTool extends BaseTool<CodeExecutionInputType, CodeExecutionOutputType> {
  // Define all members as public class field properties
  public tempDir: string = path.join(process.cwd(), '.temp');

  public buildCommand = (
    tempFile: string,
    input: CodeExecutionInputType
  ): string => {
    const config = LANGUAGE_CONFIGS[input.language];
    const args = input.args || [];
    
    const sanitizedArgs = args.map(arg => 
      arg.replace(/[;&|`$]/g, '')
    );

    let command = `${config.command} "${tempFile}"`;
    if (sanitizedArgs.length > 0) {
      command += ` ${sanitizedArgs.join(' ')}`;
    }
    return command;
  };

  public checkLanguageRuntime = async (
    language: keyof typeof LANGUAGE_CONFIGS
  ): Promise<boolean> => {
    try {
      const config = LANGUAGE_CONFIGS[language];
      const output = await Command.create(config.command, ['--version']).execute();
      return output.code === 0;
    } catch {
      return false;
    }
  };

  constructor() {
    const tool: Tool = {
      id: 'code-execution',
      name: 'Code Execution',
      description: 'Execute code snippets in various programming languages',
      type: 'system',
      category: 'development',
      version: '1.0.0',
      status: 'active',
      configuration: {
        schema: CodeExecutionInput.shape,
        defaultValues: {
          language: 'python',
          timeout: 30000,
          args: [],
        },
      },
      metadata: {
        lastUsed: new Date(),
        usageCount: 0,
        rating: 0,
      },
      permissions: {
        roles: ['developer', 'agent'],
        capabilities: ['code-execution'],
      },
      createdAt: new Date(),
      updatedAt: new Date(),
      modified: Date.now(),
    };

    super(tool, CodeExecutionInput, CodeExecutionOutput);
    // Properties are initialized above
  }

  protected async executeLocally(
    input: CodeExecutionInputType,
    _context: ToolContext
  ): Promise<ToolResult<CodeExecutionOutputType>> {
    try {
      // Use class field property
      await create(this.tempDir);

      // Create temporary file
      const config = LANGUAGE_CONFIGS[input.language];
      const tempFile = path.join(
        this.tempDir,
        `${generateId()}${config.fileExtension}`
      );

      await writeTextFile(tempFile, input.code);

      // Use class field method property
      const command = this.buildCommand(tempFile, input);
      const [cmd, ...args] = command.split(' ');

      const startTime = Date.now();
      const output = await Command.create(cmd, args, {
        cwd: input.workingDir || process.cwd(),
        env: input.env || {},
      }).execute();
      const executionTime = Date.now() - startTime;

      await remove(tempFile).catch(console.warn);

      return {
        success: true,
        data: {
          stdout: output.stdout.trim(),
          stderr: output.stderr.trim(),
          exitCode: output.code || 0,
          executionTime,
          language: input.language,
          tempFile,
        },
      };
    } catch (error) {
      if (error instanceof Error && 'code' in error) {
        const execError = error as unknown as { code: string | number; killed?: boolean; signal?: string | number }; 
        const errorMessage = execError.killed
          ? 'Execution timed out' // Need timeout mechanism re-evaluation
          : error.message;

        return {
          success: false,
          error: errorMessage,
          data: {
            stdout: '',
            stderr: errorMessage,
            exitCode: typeof execError.code === 'string' && execError.code === 'ETIMEDOUT' ? 124 : 1,
            executionTime: input.timeout || LANGUAGE_CONFIGS[input.language].defaultTimeout, // Placeholder, timeout not implemented
            language: input.language,
          },
        };
      }

      return {
        success: false,
        error: error instanceof Error ? error.message : 'Code execution failed',
      };
    }
  }

  protected async executeInCloud(
    _input: CodeExecutionInputType,
    _context: ToolContext
  ): Promise<ToolResult<CodeExecutionOutputType>> {
    // Code execution is local-only for security
    return {
      success: false,
      error: 'Code execution is not supported in cloud mode',
    };
  }

  // Methods are defined as class field properties above
}

// Export the class with the original name and apply the type assertion here
export const CodeExecutionTool = _CodeExecutionTool as any; // Workaround for persistent decorator type error 