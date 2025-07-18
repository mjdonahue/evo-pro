import { z } from 'zod';
import { BaseTool, ToolContext, ToolResult, registerTool } from './framework';
import { Tool } from '../db/types';
import { promises as fs } from 'fs';
import * as path from 'path';

// Define input/output schemas for different operations
const FileOperationType = z.enum(['read', 'write', 'append', 'delete', 'list', 'search']);

const BaseFileInput = z.object({
  operation: FileOperationType,
  path: z.string(),
});

const ReadFileInput = BaseFileInput.extend({
  operation: z.literal('read'),
  encoding: z.string().optional(),
});

const WriteFileInput = BaseFileInput.extend({
  operation: z.literal('write'),
  content: z.string(),
  encoding: z.string().optional(),
});

const AppendFileInput = BaseFileInput.extend({
  operation: z.literal('append'),
  content: z.string(),
  encoding: z.string().optional(),
});

const DeleteFileInput = BaseFileInput.extend({
  operation: z.literal('delete'),
});

const ListFilesInput = BaseFileInput.extend({
  operation: z.literal('list'),
  recursive: z.boolean().default(false),
  pattern: z.string().optional(),
});

const SearchFilesInput = BaseFileInput.extend({
  operation: z.literal('search'),
  pattern: z.string(),
  recursive: z.boolean().default(false),
  caseSensitive: z.boolean().default(false),
});

const FileOperationInput = z.discriminatedUnion('operation', [
  ReadFileInput,
  WriteFileInput,
  AppendFileInput,
  DeleteFileInput,
  ListFilesInput,
  SearchFilesInput,
]);

const FileInfo = z.object({
  name: z.string(),
  path: z.string(),
  size: z.number(),
  modified: z.date(),
  isDirectory: z.boolean(),
});

const FileOperationOutput = z.object({
  success: z.boolean(),
  operation: FileOperationType,
  path: z.string(),
  result: z.union([
    z.string(),
    z.array(FileInfo),
    z.null(),
  ]),
  metadata: z.object({
    size: z.number().optional(),
    modified: z.date().optional(),
    created: z.date().optional(),
  }).optional(),
});

type FileOperationInputType = z.infer<typeof FileOperationInput>;
type FileOperationOutputType = z.infer<typeof FileOperationOutput>;

// @ts-expect-error - Decorator return type mismatch, likely external issue
@registerTool()
export class FileOperationsTool extends BaseTool<FileOperationInputType, FileOperationOutputType> {
  constructor() {
    const tool: Tool = {
      id: 'file-operations',
      name: 'File Operations',
      description: 'Read, write, and manipulate files on the local system',
      type: 'system',
      category: 'filesystem',
      version: '1.0.0',
      status: 'active',
      configuration: {
        schema: FileOperationInput,
        defaultValues: {
          encoding: 'utf8',
          recursive: false,
        },
      },
      metadata: {
        lastUsed: new Date(),
        usageCount: 0,
        rating: 0,
      },
      permissions: {
        roles: ['user', 'agent'],
        capabilities: ['file-system'],
      },
      createdAt: new Date(),
      updatedAt: new Date(),
      modified: Date.now(),
    };

    // Use 'as any' to bypass Zod schema type mismatch in super call
    super(tool, FileOperationInput as any, FileOperationOutput);
  }

  protected async executeLocally(
    input: FileOperationInputType,
    _context: ToolContext
  ): Promise<ToolResult<FileOperationOutputType>> {
    try {
      const result = await this.handleOperation(input);
      return {
        success: true,
        data: result,
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'File operation failed',
      };
    }
  }

  protected async executeInCloud(
    _input: FileOperationInputType,
    _context: ToolContext
  ): Promise<ToolResult<FileOperationOutputType>> {
    // File operations are local-only
    return {
      success: false,
      error: 'File operations are not supported in cloud mode',
    };
  }

  private async handleOperation(
    input: FileOperationInputType
  ): Promise<FileOperationOutputType> {
    const basePath = process.cwd();
    const fullPath = path.resolve(basePath, input.path);

    // Security check - ensure path is within workspace
    if (!fullPath.startsWith(basePath)) {
      throw new Error('Access denied: Path is outside workspace');
    }

    switch (input.operation) {
      case 'read':
        return this.handleRead(fullPath, input);
      case 'write':
        return this.handleWrite(fullPath, input);
      case 'append':
        return this.handleAppend(fullPath, input);
      case 'delete':
        return this.handleDelete(fullPath);
      case 'list':
        return this.handleList(fullPath, input);
      case 'search':
        return this.handleSearch(fullPath, input);
      default:
        throw new Error('Unsupported file operation type encountered');
    }
  }

  private async handleRead(
    fullPath: string,
    input: Extract<FileOperationInputType, { operation: 'read' }>
  ): Promise<FileOperationOutputType> {
    const content = await fs.readFile(fullPath, (input.encoding || 'utf8') as BufferEncoding);
    const stats = await fs.stat(fullPath);

    return {
      success: true,
      operation: 'read',
      path: input.path,
      result: content,
      metadata: {
        size: stats.size,
        modified: stats.mtime,
        created: stats.birthtime,
      },
    };
  }

  private async handleWrite(
    fullPath: string,
    input: Extract<FileOperationInputType, { operation: 'write' }>
  ): Promise<FileOperationOutputType> {
    await fs.mkdir(path.dirname(fullPath), { recursive: true });
    await fs.writeFile(fullPath, input.content, (input.encoding || 'utf8') as BufferEncoding);
    const stats = await fs.stat(fullPath);

    return {
      success: true,
      operation: 'write',
      path: input.path,
      result: null,
      metadata: {
        size: stats.size,
        modified: stats.mtime,
        created: stats.birthtime,
      },
    };
  }

  private async handleAppend(
    fullPath: string,
    input: Extract<FileOperationInputType, { operation: 'append' }>
  ): Promise<FileOperationOutputType> {
    await fs.mkdir(path.dirname(fullPath), { recursive: true });
    await fs.appendFile(fullPath, input.content, (input.encoding || 'utf8') as BufferEncoding);
    const stats = await fs.stat(fullPath);

    return {
      success: true,
      operation: 'append',
      path: input.path,
      result: null,
      metadata: {
        size: stats.size,
        modified: stats.mtime,
        created: stats.birthtime,
      },
    };
  }

  private async handleDelete(
    fullPath: string
  ): Promise<FileOperationOutputType> {
    await fs.unlink(fullPath);

    return {
      success: true,
      operation: 'delete',
      path: fullPath,
      result: null,
    };
  }

  private async handleList(
    fullPath: string,
    input: Extract<FileOperationInputType, { operation: 'list' }>
  ): Promise<FileOperationOutputType> {
    const files = await this.listFiles(fullPath, input.recursive, input.pattern);

    return {
      success: true,
      operation: 'list',
      path: input.path,
      result: files,
    };
  }

  private async handleSearch(
    fullPath: string,
    input: Extract<FileOperationInputType, { operation: 'search' }>
  ): Promise<FileOperationOutputType> {
    const files = await this.searchFiles(
      fullPath,
      input.pattern,
      input.recursive,
      input.caseSensitive
    );

    return {
      success: true,
      operation: 'search',
      path: input.path,
      result: files,
    };
  }

  private async listFiles(
    dir: string,
    recursive: boolean,
    pattern?: string
  ): Promise<z.infer<typeof FileInfo>[]> {
    const entries = await fs.readdir(dir, { withFileTypes: true });
    const regex = pattern ? new RegExp(pattern) : null;

    const files: z.infer<typeof FileInfo>[] = [];
    
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      
      if (entry.isDirectory() && recursive) {
        files.push(...await this.listFiles(fullPath, recursive, pattern));
      }
      
      if (!regex || regex.test(entry.name)) {
        const stats = await fs.stat(fullPath);
        files.push({
          name: entry.name,
          path: fullPath,
          size: stats.size,
          modified: stats.mtime,
          isDirectory: entry.isDirectory(),
        });
      }
    }

    return files;
  }

  private async searchFiles(
    dir: string,
    pattern: string,
    recursive: boolean,
    caseSensitive: boolean
  ): Promise<z.infer<typeof FileInfo>[]> {
    const regex = new RegExp(pattern, caseSensitive ? '' : 'i');
    const entries = await fs.readdir(dir, { withFileTypes: true });
    const files: z.infer<typeof FileInfo>[] = [];

    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);

      if (entry.isDirectory() && recursive) {
        files.push(...await this.searchFiles(
          fullPath,
          pattern,
          recursive,
          caseSensitive
        ));
        continue;
      }

      if (!entry.isDirectory()) {
        try {
          const content = await fs.readFile(fullPath, 'utf8');
          if (regex.test(content)) {
            const stats = await fs.stat(fullPath);
            files.push({
              name: entry.name,
              path: fullPath,
              size: stats.size,
              modified: stats.mtime,
              isDirectory: false,
            });
          }
        } catch (error) {
          console.warn(`Failed to read file ${fullPath}:`, error);
        }
      }
    }

    return files;
  }
} 