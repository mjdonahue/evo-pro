import { z } from 'zod';
import { BaseTool, ToolContext, ToolResult, registerTool } from './framework';
import { Tool } from '../db/types';
import { Command } from '@tauri-apps/plugin-shell';
import * as path from 'path';
import { promises as fs } from 'fs';
import { generateId } from '../lib/utils';

// Define input/output schemas
const STTInput = z.object({
  // Audio input can be either a file path or a base64-encoded audio string
  audioSource: z.union([
    z.object({ type: z.literal('file'), path: z.string() }),
    z.object({ type: z.literal('base64'), data: z.string() })
  ]),
  // Whisper model options
  model: z.enum(['tiny', 'base', 'small', 'medium', 'large']).default('base'),
  language: z.string().optional(), // ISO 639-1 code, optional for auto-detection
  // Processing options
  timestamps: z.boolean().optional().default(false), // Get word-level timestamps
  translate: z.boolean().optional().default(false), // Translate to English
  saveToFile: z.boolean().optional().default(false), // Save transcription to file
});

const Timestamp = z.object({
  word: z.string(),
  start: z.number(),
  end: z.number(),
  confidence: z.number(),
});

const STTOutput = z.object({
  text: z.string(),
  language: z.string(), // Detected or specified language
  timestamps: z.array(Timestamp).optional(),
  metadata: z.object({
    duration: z.number(),
    wordCount: z.number(),
    confidence: z.number(),
    model: z.string(),
  }),
  savedToFile: z.string().optional(),
});

type STTInputType = z.infer<typeof STTInput>;
type STTOutputType = z.infer<typeof STTOutput>;

// @ts-expect-error - Decorator return type mismatch, likely external issue
@registerTool()
export class SpeechToTextTool extends BaseTool<STTInputType, STTOutputType> {
  private tempDir: string;
  private modelDir: string;

  constructor() {
    const tool: Tool = {
      id: 'speech-to-text',
      name: 'Speech to Text',
      description: 'Convert speech to text using Whisper',
      type: 'system',
      category: 'audio',
      version: '1.0.0',
      status: 'active',
      configuration: {
        schema: STTInput.shape,
        defaultValues: {
          model: 'base',
          timestamps: false,
          translate: false,
          saveToFile: false,
        },
      },
      metadata: {
        lastUsed: new Date(),
        usageCount: 0,
        rating: 0,
      },
      permissions: {
        roles: ['user', 'agent'],
        capabilities: ['audio-processing'],
      },
      createdAt: new Date(),
      updatedAt: new Date(),
      modified: Date.now(),
    };

    // Use 'as any' to bypass Zod schema type mismatch in super call
    super(tool, STTInput as any, STTOutput);
    this.tempDir = path.join(process.cwd(), '.temp', 'stt');
    this.modelDir = path.join(process.cwd(), '.cache', 'whisper');
  }

  protected async executeLocally(
    input: STTInputType,
    _context: ToolContext
  ): Promise<ToolResult<STTOutputType>> {
    try {
      // Create necessary directories
      await fs.mkdir(this.tempDir, { recursive: true });
      await fs.mkdir(this.modelDir, { recursive: true });

      // Prepare audio file
      const audioFile = await this.prepareAudioFile(input.audioSource);

      // Build whisper command
      const commandString = this.buildWhisperCommand(audioFile, input);
      const args = commandString.split(' ').slice(1); // Remove 'whisper' command name

      // Execute transcription
      const output = await Command.create('whisper', args).execute();

      if (output.code !== 0) {
        throw new Error(`Whisper failed: ${output.stderr}`);
      }

      // Parse whisper output
      const result = this.parseWhisperOutput(output.stdout);

      // Save transcription if requested
      let savedToFile: string | undefined;
      if (input.saveToFile) {
        savedToFile = path.join(this.tempDir, `transcription-${generateId()}.json`);
        await fs.writeFile(savedToFile, JSON.stringify(result, null, 2), 'utf8');
      }

      // Clean up temporary audio file
      if (input.audioSource.type === 'base64') {
        await fs.unlink(audioFile).catch(console.warn);
      }

      return {
        success: true,
        data: {
          ...result,
          ...(savedToFile && { savedToFile }),
        },
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Speech to text conversion failed',
      };
    }
  }

  protected async executeInCloud(
    _input: STTInputType,
    _context: ToolContext
  ): Promise<ToolResult<STTOutputType>> {
    // Cloud execution could be implemented using services like OpenAI's Whisper API
    return {
      success: false,
      error: 'Cloud execution not yet implemented',
    };
  }

  private async prepareAudioFile(audioSource: STTInputType['audioSource']): Promise<string> {
    if (audioSource.type === 'file') {
      // Verify file exists and is accessible
      await fs.access(audioSource.path);
      return audioSource.path;
    } else {
      // Write base64 data to temporary file
      const tempFile = path.join(this.tempDir, `audio-${generateId()}.wav`);
      const buffer = Buffer.from(audioSource.data, 'base64');
      await fs.writeFile(tempFile, buffer);
      return tempFile;
    }
  }

  private buildWhisperCommand(audioFile: string, input: STTInputType): string {
    const args = [
      `whisper "${audioFile}"`,
      `--model ${input.model}`,
      '--device cpu', // Use CPU by default for compatibility
      '--output_dir "${this.tempDir}"',
      '--output_format json',
    ];

    if (input.language) {
      args.push(`--language ${input.language}`);
    }

    if (input.timestamps) {
      args.push('--word_timestamps True');
    }

    if (input.translate) {
      args.push('--task translate');
    }

    return args.join(' ');
  }

  private parseWhisperOutput(stdout: string): Omit<STTOutputType, 'savedToFile'> {
    // Parse Whisper JSON output
    // This is a simplified example - adjust based on actual Whisper output format
    const data = JSON.parse(stdout);

    return {
      text: data.text,
      language: data.language,
      ...(data.word_timestamps && {
        timestamps: data.word_timestamps.map((t: any) => ({
          word: t.word,
          start: t.start,
          end: t.end,
          confidence: t.confidence,
        })),
      }),
      metadata: {
        duration: data.duration,
        wordCount: data.text.split(/\s+/).length,
        confidence: data.confidence,
        model: data.model,
      },
    };
  }

  // Helper method to check if Whisper is installed
  /*
  private async checkWhisperInstallation(): Promise<boolean> {
    try {
      await Command.create('whisper').execute();
      return true;
    } catch {
      return false;
    }
  }
  */
} 