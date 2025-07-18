import React, { useCallback } from 'react';
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  NodeTypes,
  Connection,
  Edge,
  Node as ReactFlowNode,
  useNodesState,
  useEdgesState,
  addEdge,
} from 'reactflow';
import 'reactflow/dist/style.css';

import { AgentNode } from './nodes/AgentNode';
import { TriggerNode } from './nodes/TriggerNode';
import { ConditionNode } from './nodes/ConditionNode';
import { DataTransformNode } from './nodes/DataTransformNode';
import { useWorkflow, useWorkflowMutations } from '../../db/hooks';
import { Toolbar } from './Toolbar';
import { NodeInspector } from './NodeInspector';

const nodeTypes: NodeTypes = {
  agent: AgentNode,
  trigger: TriggerNode,
  condition: ConditionNode,
  transform: DataTransformNode,
};

interface WorkflowEditorProps {
  workflowId: string;
}

export function WorkflowEditor({ workflowId }: WorkflowEditorProps) {
  const workflow = useWorkflow(workflowId);
  const { updateWorkflow } = useWorkflowMutations();
  const [nodes, setNodes, onNodesChange] = useNodesState<ReactFlowNode>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);
  const [selectedNode, setSelectedNode] = React.useState<ReactFlowNode | null>(null);

  // Load workflow data
  React.useEffect(() => {
    if (workflow) {
      setNodes(workflow.nodes as unknown as ReactFlowNode[]);
      setEdges(workflow.edges as unknown as Edge[]);
    }
  }, [workflow]);

  // Save workflow changes
  const saveWorkflow = useCallback(async () => {
    if (!workflow) return;
    await updateWorkflow(workflowId, {
      nodes: nodes as unknown as any[],
      edges: edges as unknown as any[],
      lastModified: new Date(),
    });
  }, [workflowId, nodes, edges]);

  // Auto-save on changes
  React.useEffect(() => {
    const timeoutId = setTimeout(saveWorkflow, 1000);
    return () => clearTimeout(timeoutId);
  }, [nodes, edges]);

  const onConnect = useCallback(
    (connection: Connection) => {
      // Validate connection
      const sourceNode = nodes.find(n => n.id === connection.source);
      const targetNode = nodes.find(n => n.id === connection.target);
      
      if (!sourceNode || !targetNode) return;
      
      // Check if connection is valid based on node types
      const isValid = validateConnection(sourceNode, targetNode);
      if (!isValid) return;

      setEdges(eds => addEdge({
        ...connection,
        type: 'smoothstep',
        animated: true,
        style: { stroke: '#64748b' },
      }, eds));
    },
    [nodes]
  );

  const onNodeClick = useCallback((_: React.MouseEvent, node: ReactFlowNode) => {
    setSelectedNode(node);
  }, []);

  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();

      const type = event.dataTransfer.getData('application/reactflow-type');
      if (!type) return;

      // Get the position of the drop
      const reactFlowBounds = document.querySelector('.react-flow')?.getBoundingClientRect();
      if (!reactFlowBounds) return;

      const position = {
        x: event.clientX - reactFlowBounds.left,
        y: event.clientY - reactFlowBounds.top,
      };

      // Create a new node
      const newNode: ReactFlowNode = {
        id: `${type}-${Date.now()}`,
        type,
        position,
        data: {
          label: `New ${type.charAt(0).toUpperCase() + type.slice(1)}`,
          // Add default data based on node type
          ...getDefaultNodeData(type),
        },
      };

      setNodes(nds => [...nds, newNode]);
    },
    []
  );

  if (!workflow) {
    return <div>Loading...</div>;
  }

  return (
    <div className="h-screen flex">
      {/* Toolbar */}
      <Toolbar workflowId={workflowId} />

      {/* Flow Editor */}
      <div className="flex-1 h-full">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onNodeClick={onNodeClick}
          onDragOver={onDragOver}
          onDrop={onDrop}
          nodeTypes={nodeTypes}
          fitView
        >
          <Background />
          <Controls />
          <MiniMap />
        </ReactFlow>
      </div>

      {/* Node Inspector */}
      {selectedNode && (
        <NodeInspector
          node={selectedNode}
          onClose={() => setSelectedNode(null)}
          onChange={(updatedData) => {
            setNodes(nds =>
              nds.map(n =>
                n.id === selectedNode.id
                  ? { ...n, data: { ...n.data, ...updatedData } }
                  : n
              )
            );
          }}
        />
      )}
    </div>
  );
}

// Helper functions
function validateConnection(sourceNode: ReactFlowNode, targetNode: ReactFlowNode): boolean {
  // Define valid connections between node types
  const validConnections: Record<string, string[]> = {
    trigger: ['agent', 'condition', 'transform'],
    agent: ['agent', 'condition', 'transform'],
    condition: ['agent', 'transform'],
    transform: ['agent', 'condition'],
  };

  const sourceType = sourceNode.type || '';
  return validConnections[sourceType]?.includes(targetNode.type || '') || false;
}

function getDefaultNodeData(type: string) {
  switch (type) {
    case 'agent':
      return {
        agentId: '',
        inputMapping: {},
        outputMapping: {},
      };
    case 'trigger':
      return {
        triggerType: 'schedule',
        config: {},
      };
    case 'condition':
      return {
        type: 'expression',
        expression: '',
      };
    case 'transform':
      return {
        transformType: 'map',
        config: {},
      };
    default:
      return {};
  }
} 