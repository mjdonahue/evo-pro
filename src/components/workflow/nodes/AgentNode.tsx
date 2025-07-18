import React from 'react';
import { Handle, Position, NodeProps } from 'reactflow';
import { useAgent } from '../../../db/hooks';

export function AgentNode({ data, isConnectable }: NodeProps) {
  const agent = useAgent(data.agentId);
  const [isProcessing, setIsProcessing] = React.useState(false);
  const [lastResult, setLastResult] = React.useState<any>(null);

  React.useEffect(() => {
    const processAgent = async () => {
      if (!agent) return;
      
      setIsProcessing(true);
      try {
        // TODO: Implement actual agent processing logic here
        const result = await new Promise(resolve => setTimeout(() => resolve({ status: 'success' }), 1000));
        setLastResult(result);
      } catch (error) {
        setLastResult({ error: error instanceof Error ? error.message : 'Unknown error' });
      } finally {
        setIsProcessing(false);
      }
    };

    processAgent();
  }, [agent]);

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
          {/* Agent Avatar */}
          <div className="relative">
            <img
              src={agent?.avatarUrl || '/default-agent.png'}
              alt={agent?.name || 'Agent'}
              className="w-8 h-8 rounded-full"
            />
            <span
              className={`absolute bottom-0 right-0 w-2 h-2 rounded-full border border-background ${
                isProcessing
                  ? 'bg-warning animate-pulse'
                  : agent?.status === 'active'
                  ? 'bg-success'
                  : 'bg-destructive'
              }`}
            />
          </div>

          {/* Agent Info */}
          <div>
            <h3 className="font-medium text-sm">
              {agent?.name || 'Select Agent'}
            </h3>
            <p className="text-xs text-muted-foreground">
              {agent?.description || 'No description'}
            </p>
          </div>
        </div>

        {/* Configuration Preview */}
        <div className="space-y-2 text-sm">
          {/* Input Mapping */}
          {Object.keys(data.inputMapping || {}).length > 0 && (
            <div className="bg-muted rounded p-2">
              <div className="text-xs font-medium mb-1">Input Mapping</div>
              {Object.entries(data.inputMapping).map(([key, value]) => (
                <div key={key} className="flex justify-between text-xs">
                  <span className="text-muted-foreground">{key}:</span>
                  <span>{String(value)}</span>
                </div>
              ))}
            </div>
          )}

          {/* Output Mapping */}
          {Object.keys(data.outputMapping || {}).length > 0 && (
            <div className="bg-muted rounded p-2">
              <div className="text-xs font-medium mb-1">Output Mapping</div>
              {Object.entries(data.outputMapping).map(([key, value]) => (
                <div key={key} className="flex justify-between text-xs">
                  <span className="text-muted-foreground">{key}:</span>
                  <span>{String(value)}</span>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Last Result Preview */}
        {lastResult && (
          <div className="mt-2 p-2 bg-muted rounded text-xs">
            <div className="font-medium mb-1">Last Result</div>
            <pre className="whitespace-pre-wrap break-words">
              {JSON.stringify(lastResult, null, 2)}
            </pre>
          </div>
        )}
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