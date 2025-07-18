import React from 'react';
import { Handle, Position, NodeProps } from 'reactflow';

export function DataTransformNode({ data, isConnectable }: NodeProps) {
  const [isProcessing, setIsProcessing] = React.useState(false);
  const [lastResult, setLastResult] = React.useState<any>(null);
  const [lastProcessed, setLastProcessed] = React.useState<Date | null>(null);

  React.useEffect(() => {
    // TODO: Implement actual data transformation logic
    const processData = async () => {
      if (!data.transformType) return;
      
      setIsProcessing(true);
      try {
        // Simulate data transformation
        const result = {
          type: data.transformType,
          status: 'success',
          timestamp: new Date().toISOString(),
        };
        setLastResult(result);
        setLastProcessed(new Date());
      } catch (error) {
        setLastResult({ error: error instanceof Error ? error.message : 'Unknown error' });
      } finally {
        setIsProcessing(false);
      }
    };

    processData();
  }, [data.transformType, data.config]);

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
          {/* Transform Icon */}
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
                isProcessing
                  ? 'bg-warning animate-pulse'
                  : lastResult?.status === 'success'
                  ? 'bg-success'
                  : 'bg-destructive'
              }`}
            />
          </div>

          {/* Transform Info */}
          <div>
            <h3 className="font-medium text-sm">
              {data.label || 'Data Transform'}
            </h3>
            <p className="text-xs text-muted-foreground">
              {data.transformType || 'No type specified'}
            </p>
          </div>
        </div>

        {/* Configuration Preview */}
        <div className="space-y-2 text-sm">
          {data.config && Object.keys(data.config).length > 0 && (
            <div className="bg-muted rounded p-2">
              <div className="text-xs font-medium mb-1">Configuration</div>
              {Object.entries(data.config).map(([key, value]) => (
                <div key={key} className="flex justify-between text-xs">
                  <span className="text-muted-foreground">{key}:</span>
                  <span>{String(value)}</span>
                </div>
              ))}
            </div>
          )}

          {/* Last Result */}
          {lastResult && (
            <div className="mt-2 p-2 bg-muted rounded text-xs">
              <div className="font-medium mb-1">Last Result</div>
              <pre className="whitespace-pre-wrap break-words">
                {JSON.stringify(lastResult, null, 2)}
              </pre>
            </div>
          )}

          {/* Last Processed Time */}
          {lastProcessed && (
            <div className="mt-2 p-2 bg-muted rounded text-xs">
              <div className="font-medium mb-1">Last Processed</div>
              <div>{lastProcessed.toLocaleTimeString()}</div>
            </div>
          )}
        </div>
      </div>

      {/* Output Handle */}
      <Handle
        type="source"
        position={Position.Right}
        isConnectable={isConnectable}
        className="w-3 h-3 bg-primary border-2 border-background"
      />
    </div>
  );
}
