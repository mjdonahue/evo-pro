import { Node } from 'reactflow';
import { useAvailableAgents } from '../../db/hooks';

interface NodeInspectorProps {
  node: Node;
  onClose: () => void;
  onChange: (data: any) => void;
}

export function NodeInspector({ node, onClose, onChange }: NodeInspectorProps) {
  const agents = useAvailableAgents();

  const renderAgentConfig = () => (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium mb-1">Select Agent</label>
        <select
          value={node.data.agentId || ''}
          onChange={(e) => onChange({ agentId: e.target.value })}
          className="w-full p-2 rounded-md border border-input bg-background"
        >
          <option value="">Select an agent...</option>
          {agents?.map((agent) => (
            <option key={agent.id} value={agent.id}>
              {agent.name}
            </option>
          ))}
        </select>
      </div>

      <div>
        <label className="block text-sm font-medium mb-1">Input Mapping</label>
        <div className="space-y-2">
          {Object.entries(node.data.inputMapping || {}).map(([key, value]) => (
            <div key={key} className="flex gap-2">
              <input
                value={key}
                onChange={(e) => {
                  const newMapping = { ...node.data.inputMapping };
                  delete newMapping[key];
                  newMapping[e.target.value] = value;
                  onChange({ inputMapping: newMapping });
                }}
                placeholder="Input key"
                className="flex-1 p-2 rounded-md border border-input bg-background"
              />
              <input
                value={String(value)}
                onChange={(e) => {
                  onChange({
                    inputMapping: {
                      ...node.data.inputMapping,
                      [key]: e.target.value,
                    },
                  });
                }}
                placeholder="Source path"
                className="flex-1 p-2 rounded-md border border-input bg-background"
              />
              <button
                onClick={() => {
                  const newMapping = { ...node.data.inputMapping };
                  delete newMapping[key];
                  onChange({ inputMapping: newMapping });
                }}
                className="p-2 text-destructive hover:bg-destructive/10 rounded-md"
              >
                ×
              </button>
            </div>
          ))}
          <button
            onClick={() => {
              onChange({
                inputMapping: {
                  ...node.data.inputMapping,
                  '': '',
                },
              });
            }}
            className="w-full p-2 rounded-md border border-dashed border-input hover:border-primary transition-colors text-sm"
          >
            + Add Input Mapping
          </button>
        </div>
      </div>

      <div>
        <label className="block text-sm font-medium mb-1">Output Mapping</label>
        <div className="space-y-2">
          {Object.entries(node.data.outputMapping || {}).map(([key, value]) => (
            <div key={key} className="flex gap-2">
              <input
                value={key}
                onChange={(e) => {
                  const newMapping = { ...node.data.outputMapping };
                  delete newMapping[key];
                  newMapping[e.target.value] = value;
                  onChange({ outputMapping: newMapping });
                }}
                placeholder="Output key"
                className="flex-1 p-2 rounded-md border border-input bg-background"
              />
              <input
                value={String(value)}
                onChange={(e) => {
                  onChange({
                    outputMapping: {
                      ...node.data.outputMapping,
                      [key]: e.target.value,
                    },
                  });
                }}
                placeholder="Target path"
                className="flex-1 p-2 rounded-md border border-input bg-background"
              />
              <button
                onClick={() => {
                  const newMapping = { ...node.data.outputMapping };
                  delete newMapping[key];
                  onChange({ outputMapping: newMapping });
                }}
                className="p-2 text-destructive hover:bg-destructive/10 rounded-md"
              >
                ×
              </button>
            </div>
          ))}
          <button
            onClick={() => {
              onChange({
                outputMapping: {
                  ...node.data.outputMapping,
                  '': '',
                },
              });
            }}
            className="w-full p-2 rounded-md border border-dashed border-input hover:border-primary transition-colors text-sm"
          >
            + Add Output Mapping
          </button>
        </div>
      </div>
    </div>
  );

  const renderTriggerConfig = () => (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium mb-1">Trigger Type</label>
        <select
          value={node.data.triggerType}
          onChange={(e) => onChange({ triggerType: e.target.value })}
          className="w-full p-2 rounded-md border border-input bg-background"
        >
          <option value="schedule">Schedule</option>
          <option value="webhook">Webhook</option>
          <option value="event">Event</option>
        </select>
      </div>

      {node.data.triggerType === 'schedule' && (
        <div>
          <label className="block text-sm font-medium mb-1">Schedule</label>
          <input
            type="text"
            value={node.data.config.schedule || ''}
            onChange={(e) =>
              onChange({
                config: { ...node.data.config, schedule: e.target.value },
              })
            }
            placeholder="*/5 * * * *"
            className="w-full p-2 rounded-md border border-input bg-background"
          />
          <p className="text-xs text-muted-foreground mt-1">
            Enter a cron expression
          </p>
        </div>
      )}

      {node.data.triggerType === 'webhook' && (
        <div>
          <label className="block text-sm font-medium mb-1">Webhook URL</label>
          <div className="flex gap-2">
            <input
              type="text"
              value={node.data.config.url || ''}
              readOnly
              className="flex-1 p-2 rounded-md border border-input bg-background"
            />
            <button className="p-2 text-primary hover:bg-primary/10 rounded-md">
              Copy
            </button>
          </div>
        </div>
      )}
    </div>
  );

  const renderConditionConfig = () => (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium mb-1">Condition Type</label>
        <select
          value={node.data.type}
          onChange={(e) => onChange({ type: e.target.value })}
          className="w-full p-2 rounded-md border border-input bg-background"
        >
          <option value="expression">Expression</option>
          <option value="script">Script</option>
        </select>
      </div>

      {node.data.type === 'expression' && (
        <div>
          <label className="block text-sm font-medium mb-1">Expression</label>
          <textarea
            value={node.data.expression || ''}
            onChange={(e) => onChange({ expression: e.target.value })}
            placeholder="data.value > 100"
            rows={3}
            className="w-full p-2 rounded-md border border-input bg-background resize-none"
          />
          <p className="text-xs text-muted-foreground mt-1">
            Use JavaScript expressions to define conditions
          </p>
        </div>
      )}

      {node.data.type === 'script' && (
        <div>
          <label className="block text-sm font-medium mb-1">Script</label>
          <textarea
            value={node.data.script || ''}
            onChange={(e) => onChange({ script: e.target.value })}
            placeholder="return data.value > 100;"
            rows={5}
            className="w-full p-2 rounded-md border border-input bg-background resize-none font-mono"
          />
        </div>
      )}
    </div>
  );

  const renderTransformConfig = () => (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium mb-1">Transform Type</label>
        <select
          value={node.data.transformType}
          onChange={(e) => onChange({ transformType: e.target.value })}
          className="w-full p-2 rounded-md border border-input bg-background"
        >
          <option value="map">Map</option>
          <option value="filter">Filter</option>
          <option value="reduce">Reduce</option>
          <option value="custom">Custom</option>
        </select>
      </div>

      {node.data.transformType === 'map' && (
        <div>
          <label className="block text-sm font-medium mb-1">Mapping</label>
          <textarea
            value={node.data.config.mapping || ''}
            onChange={(e) =>
              onChange({
                config: { ...node.data.config, mapping: e.target.value },
              })
            }
            placeholder="{ result: data.value * 2 }"
            rows={3}
            className="w-full p-2 rounded-md border border-input bg-background resize-none font-mono"
          />
        </div>
      )}

      {node.data.transformType === 'custom' && (
        <div>
          <label className="block text-sm font-medium mb-1">
            Transform Function
          </label>
          <textarea
            value={node.data.config.function || ''}
            onChange={(e) =>
              onChange({
                config: { ...node.data.config, function: e.target.value },
              })
            }
            placeholder="(data) => { return { result: data.value * 2 }; }"
            rows={5}
            className="w-full p-2 rounded-md border border-input bg-background resize-none font-mono"
          />
        </div>
      )}
    </div>
  );

  return (
    <div className="w-80 border-l border-border bg-card p-4 overflow-y-auto">
      <div className="flex items-center justify-between mb-4">
        <h2 className="font-semibold">Node Configuration</h2>
        <button
          onClick={onClose}
          className="p-1 text-muted-foreground hover:text-foreground"
        >
          ×
        </button>
      </div>

      <div className="space-y-6">
        {/* Common Fields */}
        <div>
          <label className="block text-sm font-medium mb-1">Label</label>
          <input
            type="text"
            value={node.data.label}
            onChange={(e) => onChange({ label: e.target.value })}
            className="w-full p-2 rounded-md border border-input bg-background"
          />
        </div>

        {/* Node-specific Configuration */}
        {node.type === 'agent' && renderAgentConfig()}
        {node.type === 'trigger' && renderTriggerConfig()}
        {node.type === 'condition' && renderConditionConfig()}
        {node.type === 'transform' && renderTransformConfig()}
      </div>
    </div>
  );
} 