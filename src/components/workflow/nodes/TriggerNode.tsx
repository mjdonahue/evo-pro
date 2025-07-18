import React from 'react';
import { Handle, Position, NodeProps } from 'reactflow';

export function TriggerNode({ data, isConnectable }: NodeProps) {
  const [isActive, setIsActive] = React.useState(false);
  const [lastTriggered, setLastTriggered] = React.useState<Date | null>(null);

  React.useEffect(() => {
    // TODO: Implement actual trigger logic based on data.triggerType
    const interval = setInterval(() => {
      if (data.triggerType === 'schedule' && data.config?.interval) {
        setIsActive(true);
        setLastTriggered(new Date());
        setTimeout(() => setIsActive(false), 1000);
      }
    }, 5000);

    return () => clearInterval(interval);
  }, [data.triggerType, data.config]);

  return (
    <div className="bg-card border border-border rounded-lg shadow-sm min-w-[200px]">
      {/* Output Handle */}
      <Handle
        type="source"
        position={Position.Right}
        isConnectable={isConnectable}
        className="w-3 h-3 bg-primary border-2 border-background"
      />

      {/* Node Content */}
      <div className="p-4">
        <div className="flex items-center gap-3 mb-2">
          {/* Trigger Icon */}
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
                isActive ? 'bg-success animate-pulse' : 'bg-muted'
              }`}
            />
          </div>

          {/* Trigger Info */}
          <div>
            <h3 className="font-medium text-sm">
              {data.label || 'Trigger'}
            </h3>
            <p className="text-xs text-muted-foreground">
              {data.triggerType || 'No type specified'}
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

          {/* Last Triggered Time */}
          {lastTriggered && (
            <div className="mt-2 p-2 bg-muted rounded text-xs">
              <div className="font-medium mb-1">Last Triggered</div>
              <div>{lastTriggered.toLocaleTimeString()}</div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
