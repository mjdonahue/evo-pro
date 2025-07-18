import React from 'react';
import { useWorkflowExecutor } from '../../db/hooks';

const nodeTypes = [
  {
    type: 'agent',
    label: 'Agent',
    icon: '🤖',
    description: 'Add an AI agent to process data or make decisions',
  },
  {
    type: 'trigger',
    label: 'Trigger',
    icon: '⚡',
    description: 'Start the workflow based on events or schedules',
  },
  {
    type: 'condition',
    label: 'Condition',
    icon: '🔀',
    description: 'Add branching logic based on conditions',
  },
  {
    type: 'transform',
    label: 'Transform',
    icon: '🔄',
    description: 'Transform data between nodes',
  },
];

interface ToolbarProps {
  workflowId: string;
}

export function Toolbar({ workflowId }: ToolbarProps) {
  const { isExecuting, startExecution, stopExecution } = useWorkflowExecutor();

  const onDragStart = (event: React.DragEvent, nodeType: string) => {
    event.dataTransfer.setData('application/reactflow-type', nodeType);
    event.dataTransfer.effectAllowed = 'move';
  };

  return (
    <div className="w-64 border-r border-border bg-card p-4 flex flex-col">
      <div className="mb-6">
        <h2 className="font-semibold mb-2">Workflow Controls</h2>
        <div className="space-y-2">
          <button
            onClick={() => isExecuting ? stopExecution() : startExecution(workflowId)}
            className={`w-full px-4 py-2 rounded-lg flex items-center justify-center gap-2 ${
              isExecuting
                ? 'bg-destructive text-destructive-foreground'
                : 'bg-primary text-primary-foreground'
            }`}
          >
            <span className={isExecuting ? 'animate-pulse' : ''}>
              {isExecuting ? '⏹️ Stop' : '▶️ Start'}
            </span>
          </button>
          <button
            className="w-full px-4 py-2 rounded-lg bg-secondary text-secondary-foreground"
          >
            💾 Save
          </button>
        </div>
      </div>

      <div>
        <h2 className="font-semibold mb-2">Available Nodes</h2>
        <div className="space-y-2">
          {nodeTypes.map((nodeType) => (
            <div
              key={nodeType.type}
              draggable
              onDragStart={(e) => onDragStart(e, nodeType.type)}
              className="p-3 rounded-lg bg-background border border-border hover:border-primary cursor-move transition-colors"
            >
              <div className="flex items-center gap-2 mb-1">
                <span className="text-xl">{nodeType.icon}</span>
                <span className="font-medium">{nodeType.label}</span>
              </div>
              <p className="text-xs text-muted-foreground">
                {nodeType.description}
              </p>
            </div>
          ))}
        </div>
      </div>

      {/* Execution Stats */}
      <div className="mt-auto pt-4 border-t border-border">
        <h3 className="font-medium text-sm mb-2">Execution Stats</h3>
        <div className="space-y-1 text-sm">
          <div className="flex justify-between">
            <span className="text-muted-foreground">Status:</span>
            <span className={isExecuting ? 'text-success' : 'text-muted-foreground'}>
              {isExecuting ? 'Running' : 'Stopped'}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">Last Run:</span>
            <span>2 mins ago</span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">Success Rate:</span>
            <span>98%</span>
          </div>
        </div>
      </div>
    </div>
  );
} 