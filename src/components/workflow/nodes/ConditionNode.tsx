import React from 'react';
import { Handle, Position, NodeProps } from 'reactflow';

export function ConditionNode({ data, isConnectable }: NodeProps) {
  const [result, setResult] = React.useState<boolean | null>(null);
  const [lastEvaluated, setLastEvaluated] = React.useState<Date | null>(null);

  React.useEffect(() => {
    // TODO: Implement actual condition evaluation logic
    const evaluateCondition = async () => {
      try {
        // Simulate condition evaluation
        const value = data.expression ? true : false;
        setResult(value);
        setLastEvaluated(new Date());
      } catch (error) {
        setResult(false);
      }
    };

    evaluateCondition();
  }, [data.expression]);

  return (
    <div className="bg-card border border-border rounded-lg shadow-sm min-w-[200px]">
      {/* Input Handle */}
      <Handle
        type="target"
        position={Position.Left}
        isConnectable={isConnectable}
        className="w-3 h-3 bg-primary border-2 border-background"
      />

      {/* Node Content */}
      <div className="p-4">
        <div className="flex items-center gap-3 mb-2">
          {/* Condition Icon */}
          <div className="relative">
            <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                className="w-4 h-4 text-primary"
              >
                <path d="M12 2v4" />
                <path d="M12 18v4" />
                <path d="M4.93 4.93l2.83 2.83" />
                <path d="M16.24 16.24l2.83 2.83" />
                <path d="M2 12h4" />
                <path d="M18 12h4" />
                <path d="M4.93 19.07l2.83-2.83" />
                <path d="M16.24 7.76l2.83-2.83" />
              </svg>
            </div>
            <span
              className={`absolute bottom-0 right-0 w-2 h-2 rounded-full border border-background ${
                result === null
                  ? 'bg-muted'
                  : result
                  ? 'bg-success'
                  : 'bg-destructive'
              }`}
            />
          </div>

          {/* Condition Info */}
          <div>
            <h3 className="font-medium text-sm">
              {data.label || 'Condition'}
            </h3>
            <p className="text-xs text-muted-foreground">
              {data.type || 'No type specified'}
            </p>
          </div>
        </div>

        {/* Condition Expression */}
        <div className="space-y-2 text-sm">
          {data.expression && (
            <div className="bg-muted rounded p-2">
              <div className="text-xs font-medium mb-1">Expression</div>
              <div className="font-mono text-xs break-all">
                {data.expression}
              </div>
            </div>
          )}

          {/* Result */}
          {result !== null && (
            <div className="mt-2 p-2 bg-muted rounded text-xs">
              <div className="font-medium mb-1">Result</div>
              <div className={result ? 'text-success' : 'text-destructive'}>
                {result ? 'True' : 'False'}
              </div>
            </div>
          )}

          {/* Last Evaluated Time */}
          {lastEvaluated && (
            <div className="mt-2 p-2 bg-muted rounded text-xs">
              <div className="font-medium mb-1">Last Evaluated</div>
              <div>{lastEvaluated.toLocaleTimeString()}</div>
            </div>
          )}
        </div>
      </div>

      {/* Output Handles */}
      <Handle
        type="source"
        position={Position.Right}
        id="true"
        isConnectable={isConnectable}
        className="w-3 h-3 bg-success border-2 border-background"
      />
      <Handle
        type="source"
        position={Position.Bottom}
        id="false"
        isConnectable={isConnectable}
        className="w-3 h-3 bg-destructive border-2 border-background"
      />
    </div>
  );
}
