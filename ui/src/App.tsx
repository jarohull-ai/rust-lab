import React, { useState, useEffect, useCallback, useMemo } from 'react';
import {
  ReactFlow,
  MiniMap,
  Controls,
  Background,
  useNodesState,
  useEdgesState,
  addEdge,
  Connection,
  Edge,
  Node,
  BackgroundVariant,
} from '@xyflow/react';
import { CustomNode } from './components/CustomNode';
import { Sidebar } from './components/Sidebar';
import { CodeView } from './components/CodeView';
import { ConsoleOutput } from './components/ConsoleOutput';
import { Toolbar } from './components/Toolbar';
import {
  GraphSpec,
  NodeSpec,
  ValidateResponse,
  ExecutionReport,
  ExportResponse,
  UiLog,
} from './types';

let idCounter = 1;
let logCounter = 1;

const INITIAL_NODES: Node[] = [
  {
    id: 'cleaner',
    type: 'custom',
    position: { x: 80, y: 150 },
    data: {
      kind: 'text_clean',
      config: { text: '   HELLO RUST LAB!   ', to_lower: true, trim: true },
      status: 'idle',
    },
  },
  {
    id: 'logger',
    type: 'custom',
    position: { x: 420, y: 150 },
    data: {
      kind: 'log_output',
      config: {},
      status: 'idle',
    },
  },
];

const INITIAL_EDGES: Edge[] = [
  {
    id: 'e-cleaner-logger',
    source: 'cleaner',
    sourceHandle: 'cleaned',
    target: 'logger',
    targetHandle: 'text',
    animated: true,
    style: { stroke: '#10b981', strokeWidth: 2 },
  },
];

export const App: React.FC = () => {
  const [nodes, setNodes, onNodesChange] = useNodesState(INITIAL_NODES);
  const [edges, setEdges, onEdgesChange] = useEdgesState(INITIAL_EDGES);

  const [nodeSpecs, setNodeSpecs] = useState<NodeSpec[]>([]);
  const [apiConnected, setApiConnected] = useState<boolean>(false);
  const [validationReport, setValidationReport] = useState<ValidateResponse | null>(null);
  const [executionReport, setExecutionReport] = useState<ExecutionReport | null>(null);
  const [exportData, setExportData] = useState<ExportResponse | null>(null);
  const [logs, setLogs] = useState<UiLog[]>([]);
  const [consoleOpen, setConsoleOpen] = useState<boolean>(true);

  const [isValidating, setIsValidating] = useState<boolean>(false);
  const [isExecuting, setIsExecuting] = useState<boolean>(false);
  const [isExporting, setIsExporting] = useState<boolean>(false);
  const [isCodeLoading, setIsCodeLoading] = useState<boolean>(false);

  const appendLog = useCallback(
    (key: string, params: Record<string, string | number> = {}, icon = '') => {
      setLogs((previous) => [
        ...previous,
        { id: logCounter++, timestamp: Date.now(), icon, key, params },
      ]);
    },
    []
  );

  // 1. Fetch available nodes and test API connection
  const fetchSpecs = useCallback(async () => {
    try {
      const res = await fetch('/api/nodes');
      if (res.ok) {
        const data: NodeSpec[] = await res.json();
        setNodeSpecs(data);
        setApiConnected(true);
      } else {
        setApiConnected(false);
      }
    } catch {
      setApiConnected(false);
    }
  }, []);

  useEffect(() => {
    fetchSpecs();
    const interval = setInterval(fetchSpecs, 5000);
    return () => clearInterval(interval);
  }, [fetchSpecs]);

  // 2. Helper to build GraphSpec from current ReactFlow nodes & edges
  const buildGraphSpec = useCallback((): GraphSpec => {
    const activeNodeIds = new Set(nodes.map((n) => n.id));
    return {
      graph_version: '0.1',
      nodes: nodes.map((n) => ({
        id: n.id,
        kind: n.data.kind as string,
        config: (n.data.config as Record<string, any>) || {},
      })),
      edges: edges
        .filter((e) => activeNodeIds.has(e.source) && activeNodeIds.has(e.target) && e.sourceHandle && e.targetHandle)
        .map((e) => ({
          from: { node: e.source, port: e.sourceHandle as string },
          to: { node: e.target, port: e.targetHandle as string },
        })),
    };
  }, [nodes, edges]);

  // 3. Debounced Live Code Compilation from API
  const refreshCode = useCallback(async () => {
    setIsCodeLoading(true);
    try {
      const graph = buildGraphSpec();
      const res = await fetch('/api/export', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ graph, project_name: 'rust_lab_demo' }),
      });
      if (res.ok) {
        const data: ExportResponse = await res.json();
        setExportData(data);
      }
    } catch (e) {
      console.error('Failed to sync code:', e);
    } finally {
      setIsCodeLoading(false);
    }
  }, [buildGraphSpec]);

  useEffect(() => {
    const timer = setTimeout(() => {
      refreshCode();
    }, 400);
    return () => clearTimeout(timer);
  }, [refreshCode]);

  // 4. Update Node Configuration
  const handleConfigChange = useCallback((nodeId: string, key: string, value: any) => {
    setNodes((nds) =>
      nds.map((node) => {
        if (node.id === nodeId) {
          return {
            ...node,
            data: {
              ...node.data,
              config: {
                ...(node.data.config as Record<string, any>),
                [key]: value,
              },
            },
          };
        }
        return node;
      })
    );
  }, [setNodes]);

  // 5. Delete Node
  const handleDeleteNode = useCallback((nodeId: string) => {
    setNodes((nds) => nds.filter((n) => n.id !== nodeId));
    setEdges((eds) => eds.filter((e) => e.source !== nodeId && e.target !== nodeId));
  }, [setNodes, setEdges]);

  const handleFocusNode = useCallback((nodeId: string) => {
    setNodes((nds) =>
      nds.map((n) => ({
        ...n,
        selected: n.id === nodeId,
      }))
    );
  }, [setNodes]);

  const onNodesDelete = useCallback((deletedNodes: Node[]) => {
    const deletedIds = new Set(deletedNodes.map((n) => n.id));
    setEdges((eds) => eds.filter((e) => !deletedIds.has(e.source) && !deletedIds.has(e.target)));
  }, [setEdges]);

  // 6. Connect Handles
  const onConnect = useCallback(
    (params: Connection) => {
      setEdges((eds) =>
        addEdge(
          {
            ...params,
            animated: true,
            style: { stroke: '#38bdf8', strokeWidth: 2 },
          },
          eds
        )
      );
    },
    [setEdges]
  );

  // 7. Drag & Drop Add Node
  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  const addNodeWithKind = useCallback(
    (kind: string, position = { x: 250, y: 200 }) => {
      const newId = `${kind}_${idCounter++}`;
      const newNode: Node = {
        id: newId,
        type: 'custom',
        position,
        data: {
          kind,
          config: {},
          status: 'idle',
        },
      };
      setNodes((nds) => nds.concat(newNode));
    },
    [setNodes]
  );

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();
      const kind = event.dataTransfer.getData('application/reactflow');
      if (!kind) return;

      const reactFlowBounds = event.currentTarget.getBoundingClientRect();
      const position = {
        x: event.clientX - reactFlowBounds.left - 100,
        y: event.clientY - reactFlowBounds.top - 50,
      };

      addNodeWithKind(kind, position);
    },
    [addNodeWithKind]
  );

  // 8. Validate Graph Action
  const handleValidate = async () => {
    setIsValidating(true);
    setConsoleOpen(true);
    try {
      const graph = buildGraphSpec();
      const res = await fetch('/api/validate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(graph),
      });
      const data: ValidateResponse = await res.json();
      setValidationReport(data);
      if (data.valid) {
        appendLog('log.validationPassed', { order: data.execution_order.join(' → ') }, '✅');
      } else {
        appendLog('log.validationFailed', { count: data.errors.length }, '❌');
      }
    } catch (err: any) {
      appendLog('log.apiError', { message: err.message }, '❌');
    } finally {
      setIsValidating(false);
    }
  };

  // 9. Execute Graph Action
  const handleExecute = async () => {
    setIsExecuting(true);
    setConsoleOpen(true);
    try {
      const graph = buildGraphSpec();
      const res = await fetch('/api/execute', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(graph),
      });

      if (!res.ok) {
        const errJson = await res.json();
        appendLog('log.executionError', { message: errJson.error }, '❌');
        return;
      }

      const report: ExecutionReport = await res.json();
      setExecutionReport(report);

      // Update node visual status (executed vs skipped)
      setNodes((nds) =>
        nds.map((node) => {
          let st: 'idle' | 'executed' | 'skipped' | 'error' = 'idle';
          if (report.executed.includes(node.id)) st = 'executed';
          else if (report.skipped.includes(node.id)) st = 'skipped';
          return {
            ...node,
            data: {
              ...node.data,
              status: st,
              outputValue: report.outputs[node.id],
            },
          };
        })
      );

      appendLog(
        'log.executionFinished',
        { executed: report.executed.join(', '), skipped: report.skipped.join(', ') },
        '🚀'
      );
    } catch (err: any) {
      appendLog('log.apiError', { message: err.message }, '❌');
    } finally {
      setIsExecuting(false);
    }
  };

  // 10. Export to Disk
  const handleExportDisk = async () => {
    setIsExporting(true);
    setConsoleOpen(true);
    try {
      const graph = buildGraphSpec();
      const targetDir = 'target/exported-visual-app';
      const res = await fetch('/api/export/disk', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          graph,
          project_name: 'exported_visual_app',
          output_dir: targetDir,
        }),
      });

      if (res.ok) {
        const data = await res.json();
        appendLog('log.exported', { path: data.output_dir }, '🎉');
        appendLog('log.runExport', { path: data.output_dir }, '👉');
      } else {
        const errJson = await res.json();
        appendLog('log.exportFailed', { message: errJson.error }, '❌');
      }
    } catch (err: any) {
      appendLog('log.apiError', { message: err.message }, '❌');
    } finally {
      setIsExporting(false);
    }
  };

  // 11. Clear Canvas
  const handleClear = () => {
    setNodes([]);
    setEdges([]);
    setExecutionReport(null);
    setValidationReport(null);
    setExportData(null);
    appendLog('log.canvasCleared', {}, '🧹');
  };

  // 12. Load Ready-to-Run Examples
  const handleLoadExample = (example: 'condition' | 'text' | 'http') => {
    if (example === 'text') {
      setNodes([
        {
          id: 'cleaner',
          type: 'custom',
          position: { x: 80, y: 150 },
          data: {
            kind: 'text_clean',
            config: { text: '   HELLO RUST LAB!   ', to_lower: true, trim: true },
            status: 'idle',
          },
        },
        {
          id: 'logger',
          type: 'custom',
          position: { x: 420, y: 150 },
          data: { kind: 'log_output', config: {}, status: 'idle' },
        },
      ]);
      setEdges([
        {
          id: 'e1',
          source: 'cleaner',
          sourceHandle: 'cleaned',
          target: 'logger',
          targetHandle: 'text',
          animated: true,
          style: { stroke: '#10b981', strokeWidth: 2 },
        },
      ]);
    } else if (example === 'condition') {
      setNodes([
        {
          id: 'cond',
          type: 'custom',
          position: { x: 60, y: 180 },
          data: {
            kind: 'condition',
            config: { value: true, true_branch: 'true_log', false_branch: 'false_log' },
            status: 'idle',
          },
        },
        {
          id: 'true_log',
          type: 'custom',
          position: { x: 400, y: 80 },
          data: {
            kind: 'log_output',
            config: { text: 'Branch TRUE was taken!' },
            status: 'idle',
          },
        },
        {
          id: 'false_log',
          type: 'custom',
          position: { x: 400, y: 280 },
          data: {
            kind: 'log_output',
            config: { text: 'Branch FALSE was taken!' },
            status: 'idle',
          },
        },
      ]);
      setEdges([]);
    } else if (example === 'http') {
      setNodes([
        {
          id: 'fetcher',
          type: 'custom',
          position: { x: 60, y: 150 },
          data: {
            kind: 'http_get',
            config: { url: 'https://httpbin.org/json' },
            status: 'idle',
          },
        },
        {
          id: 'parser',
          type: 'custom',
          position: { x: 380, y: 150 },
          data: {
            kind: 'json_parse',
            config: { path: 'slideshow.title' },
            status: 'idle',
          },
        },
        {
          id: 'to_str',
          type: 'custom',
          position: { x: 700, y: 150 },
          data: {
            kind: 'json_to_string',
            config: {},
            status: 'idle',
          },
        },
        {
          id: 'logger',
          type: 'custom',
          position: { x: 1000, y: 150 },
          data: {
            kind: 'log_output',
            config: {},
            status: 'idle',
          },
        },
      ]);
      setEdges([
        {
          id: 'e1',
          source: 'fetcher',
          sourceHandle: 'body',
          target: 'parser',
          targetHandle: 'json_str',
          animated: true,
          style: { stroke: '#06b6d4', strokeWidth: 2 },
        },
        {
          id: 'e2',
          source: 'parser',
          sourceHandle: 'value',
          target: 'to_str',
          targetHandle: 'value',
          animated: true,
          style: { stroke: '#f59e0b', strokeWidth: 2 },
        },
        {
          id: 'e3',
          source: 'to_str',
          sourceHandle: 'text',
          target: 'logger',
          targetHandle: 'text',
          animated: true,
          style: { stroke: '#10b981', strokeWidth: 2 },
        },
      ]);
    }

    appendLog('log.exampleLoaded', { example }, '📂');
  };

  // Node types registration with injected callbacks and specs
  const nodeTypes = useMemo(() => ({ custom: CustomNode }), []);

  const enhancedNodes = useMemo(() => {
    const specMap = new Map<string, NodeSpec>();
    nodeSpecs.forEach((s) => specMap.set(s.name, s));

    return nodes.map((node) => ({
      ...node,
      data: {
        ...node.data,
        spec: specMap.get(node.data.kind as string),
        onConfigChange: handleConfigChange,
        onDelete: handleDeleteNode,
      },
    }));
  }, [nodes, nodeSpecs, handleConfigChange, handleDeleteNode]);

  return (
    <div className="flex flex-col h-screen w-screen bg-slate-950 font-sans text-slate-100 select-none">
      {/* Top Navigation & Actions */}
      <Toolbar
        graph={buildGraphSpec()}
        onValidate={handleValidate}
        onExecute={handleExecute}
        onExportDisk={handleExportDisk}
        onClear={handleClear}
        onLoadExample={handleLoadExample}
        onFocusNode={handleFocusNode}
        apiConnected={apiConnected}
        isValidating={isValidating}
        isExecuting={isExecuting}
        isExporting={isExporting}
      />

      {/* Main Workspace */}
      <div className="flex flex-1 overflow-hidden">
        {/* Left Sidebar: Node Palette */}
        <Sidebar nodeSpecs={nodeSpecs} onAddNode={addNodeWithKind} />

        {/* Center: Interactive Visual Canvas */}
        <div className="flex-1 flex flex-col h-full relative" onDrop={onDrop} onDragOver={onDragOver}>
          <div className="flex-1 w-full h-full">
            <ReactFlow
              nodes={enhancedNodes}
              edges={edges}
              onNodesChange={onNodesChange}
              onEdgesChange={onEdgesChange}
              onNodesDelete={onNodesDelete}
              onConnect={onConnect}
              nodeTypes={nodeTypes}
              fitView
              colorMode="dark"
            >
              <Background variant={BackgroundVariant.Dots} gap={16} size={1} color="#334155" />
              <Controls className="bg-slate-900 border border-slate-800 text-slate-200 fill-slate-200" />
              <MiniMap
                nodeColor="#475569"
                maskColor="rgba(15, 23, 42, 0.75)"
                className="bg-slate-900 border border-slate-800 rounded-lg"
              />
            </ReactFlow>
          </div>

          {/* Bottom Execution & Diagnostic Console */}
          <ConsoleOutput
            logs={logs}
            executionReport={executionReport}
            validationReport={validationReport}
            isOpen={consoleOpen}
            onToggle={() => setConsoleOpen(!consoleOpen)}
            onClear={() => setLogs([])}
          />
        </div>

        {/* Right Pane: Live Generated Rust Code */}
        <CodeView exportData={exportData} loading={isCodeLoading} onRefresh={refreshCode} />
      </div>
    </div>
  );
};

export default App;
