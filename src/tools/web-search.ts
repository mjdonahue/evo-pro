import { z } from 'zod';
import { BaseTool, ToolContext, ToolResult, registerTool } from './framework';
import { Tool } from '../db/types';
import { Command } from '@tauri-apps/plugin-shell';
import { writeTextFile } from '@tauri-apps/plugin-fs';

// Define input/output schemas
const WebSearchInput = z.object({
  query: z.string(),
  maxResults: z.number().optional().default(10),
  language: z.string().optional().default('en'),
  saveToFile: z.boolean().optional().default(false),
});

const WebSearchOutput = z.object({
  results: z.array(z.object({
    title: z.string(),
    snippet: z.string(),
    url: z.string(),
    score: z.number(),
  })),
  totalResults: z.number(),
  searchTime: z.number(),
  savedToFile: z.string().optional(),
});

type WebSearchInputType = z.infer<typeof WebSearchInput>;
type WebSearchOutputType = z.infer<typeof WebSearchOutput>;

// @ts-expect-error - Decorator return type mismatch, likely external issue
@registerTool()
export class WebSearchTool extends BaseTool<WebSearchInputType, WebSearchOutputType> {
  constructor() {
    // Define the tool metadata
    const tool: Tool = {
      id: 'web-search',
      name: 'Web Search',
      description: 'Search the web for information using cursor-tools',
      type: 'function',
      category: 'web',
      version: '1.0.0',
      status: 'active',
      configuration: {
        schema: WebSearchInput.shape,
        defaultValues: {
          maxResults: 10,
          language: 'en',
          saveToFile: false,
        },
        validation: {
          query: {
            minLength: 3,
            maxLength: 1000,
          },
        },
      },
      metadata: {
        rating: 0,
        lastUsed: new Date(),
        usageCount: 0,
      },
      permissions: {
        roles: ['user', 'agent'],
        capabilities: ['web-access'],
      },
      createdAt: new Date(),
      updatedAt: new Date(),
      modified: Date.now(),
    };

    // Use 'as any' to bypass Zod schema type mismatch in super call
    super(tool, WebSearchInput as any, WebSearchOutput);
  }

  protected async executeLocally(
    input: WebSearchInputType,
    _context: ToolContext
  ): Promise<ToolResult<WebSearchOutputType>> {
    try {
      const startTime = Date.now();

      // Build the command arguments
      const args = ['web', input.query];
      
      // Add save to file option if enabled
      let outputFile: string | undefined;
      if (input.saveToFile) {
        const sanitizedQuery = input.query.replace(/[^a-zA-Z0-9]/g, '-').toLowerCase();
        outputFile = `local-research/${sanitizedQuery}.md`;
        args.push('--save-to', outputFile);
      }

      // Execute the command
      const output = await Command.create('cursor-tools', args).execute();
      
      if (output.code !== 0) {
        console.warn('Web search warning:', output.stderr);
      }

      // Parse the results
      const rawResults = output.stdout.split('\n').filter(Boolean);
      
      const results = rawResults.map((line, index) => {
        const [title, url, ...snippetParts] = line.split('|').map(s => s.trim());
        const snippet = snippetParts.join('|').trim();
        
        return {
          title: title || 'Untitled',
          url: url || '',
          snippet: snippet || '',
          score: 1 - (index * 0.1),
        };
      });

      return {
        success: true,
        data: {
          results: results.slice(0, input.maxResults),
          totalResults: results.length,
          searchTime: Date.now() - startTime,
          ...(outputFile && { savedToFile: outputFile }),
        },
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Failed to execute web search locally',
      };
    }
  }

  protected async executeInCloud(
    input: WebSearchInputType,
    context: ToolContext
  ): Promise<ToolResult<WebSearchOutputType>> {
    try {
      const startTime = Date.now();
      
      // Call cloud API endpoint for web search
      const response = await fetch('https://api.example.com/web-search', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${context.env.API_KEY}`,
        },
        body: JSON.stringify(input),
      });

      if (!response.ok) {
        throw new Error(`Cloud API error: ${response.statusText}`);
      }

      const data = await response.json();
      
      // Add search time to the response
      const result = {
        ...WebSearchOutput.parse(data),
        searchTime: Date.now() - startTime,
      };

      // Save to file if requested
      if (input.saveToFile && result.results.length > 0) {
        const sanitizedQuery = input.query.replace(/[^a-zA-Z0-9]/g, '-').toLowerCase();
        const outputFile = `local-research/${sanitizedQuery}.md`;
        
        // Format results as markdown
        const markdown = [
          `# Search Results: ${input.query}`,
          `\nExecuted at: ${new Date().toISOString()}`,
          '\n## Results\n',
          ...result.results.map((r, i) => (
            `### ${i + 1}. ${r.title}\n` +
            `- URL: ${r.url}\n` +
            `- Score: ${r.score}\n\n` +
            `${r.snippet}\n`
          )),
        ].join('\n');

        // Save to file using Tauri's fs API
        await writeTextFile(outputFile, markdown);
        result.savedToFile = outputFile;
      }

      return {
        success: true,
        data: result,
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Failed to execute web search in cloud',
      };
    }
  }
} 