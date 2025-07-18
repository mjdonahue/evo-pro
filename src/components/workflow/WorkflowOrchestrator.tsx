import { useCallback, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import ReactFlow, {
  Node,
  Edge,
  Controls,
  Background,
  useNodesState,
  useEdgesState,
  addEdge,
  Connection,
} from 'reactflow';
import 'reactflow/dist/style.css';
import { Button } from '@evo/ui/button';
import { Plus, Save } from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@evo/ui/dialog';
import { Input } from '@evo/ui/input';
import { Label } from '@evo/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@evo/ui/select';

const initialNodes: Node[] = [
  {
    id: '1',
    type: 'input',
    data: { label: 'Start' },
    position: { x: 250, y: 25 },
  },
];

const initialEdges: Edge[] = [];

export function WorkflowOrchestrator() {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [newNodeData, setNewNodeData] = useState({
    label: '',
    type: 'agent',
  });

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  const handleAddNode = () => {
    const newNode: Node = {
      id: (nodes.length + 1).toString(),
      data: { label: newNodeData.label },
      position: {
        x: Math.random() * 500,
        y: Math.random() * 500,
      },
    };

    setNodes((nds) => [...nds, newNode]);
    setIsDialogOpen(false);
    setNewNodeData({ label: '', type: 'agent' });
  };

  const handleSaveWorkflow = async () => {
    try {
      await invoke('save_workflow', {
        nodes,
        edges,
      });
    } catch (error) {
      console.error('Failed to save workflow:', error);
    }
  };

  return (
    <div className="h-full">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-2xl font-bold">Workflow Orchestrator</h2>
        <div className="flex gap-2">
          <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
            <DialogTrigger asChild>
              <Button>
                <Plus className="h-4 w-4 mr-2" />
                Add Node
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Add New Node</DialogTitle>
              </DialogHeader>
              <div className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="label">Node Label</Label>
                  <Input
                    id="label"
                    value={newNodeData.label}
                    onChange={(e) =>
                      setNewNodeData({ ...newNodeData, label: e.target.value })
                    }
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="type">Node Type</Label>
                  <Select
                    value={newNodeData.type}
                    onValueChange={(value) =>
                      setNewNodeData({ ...newNodeData, type: value })
                    }
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select node type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="agent">Agent</SelectItem>
                      <SelectItem value="condition">Condition</SelectItem>
                      <SelectItem value="action">Action</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <Button onClick={handleAddNode}>Add Node</Button>
              </div>
            </DialogContent>
          </Dialog>
          <Button onClick={handleSaveWorkflow}>
            <Save className="h-4 w-4 mr-2" />
            Save Workflow
          </Button>
        </div>
      </div>

      <div className="h-[calc(100%-4rem)] border rounded-lg">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          fitView
        >
          <Background />
          <Controls />
        </ReactFlow>
      </div>
    </div>
  );
} 