import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react';
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
import { GuidedLessons } from './components/GuidedLessons';
import { WelcomeScreen } from './components/WelcomeScreen';
import { createLessonSetup, LessonId } from './lessons';
import {
  AUTOSAVE_KEY,
  createProjectDocument,
  normalizeProjectName,
  parseProjectDocument,
  projectFileName,
  projectToWorkspace,
} from './project';
import { createStoredZip } from './zip';
import { useI18n } from './i18n';
import {
  GraphSpec,
  NodeSpec,
  ValidateResponse,
  ExecutionReport,
  ExportResponse,
  CompileResponse,
  LanguageTarget,
  UiLog,
} from './types';

const WorkspaceLab = React.lazy(async () => {
  const module = await import('./components/WorkspaceLab');
  return { default: module.WorkspaceLab };
});

let idCounter = 1;
let logCounter = 1;
const WELCOME_KEY = 'rust-lab.welcome.0.2.dismissed';
const MAX_PROJECT_FILE_BYTES = 1_000_000;

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

function loadInitialWorkspace(): { name: string; nodes: Node[]; edges: Edge[] } {
  const fallback = { name: 'Untitled Project', nodes: INITIAL_NODES, edges: INITIAL_EDGES };
  try {
    const stored = window.localStorage.getItem(AUTOSAVE_KEY);
    if (!stored) return fallback;
    const project = parseProjectDocument(stored);
    return { name: project.name, ...projectToWorkspace(project) };
  } catch {
    window.localStorage.removeItem(AUTOSAVE_KEY);
    return fallback;
  }
}

function downloadBlob(blob: Blob, fileName: string): void {
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = fileName;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}

export const App: React.FC = () => {
  const { t } = useI18n();
  const initialWorkspace = useMemo(loadInitialWorkspace, []);
  const [nodes, setNodes, onNodesChange] = useNodesState(initialWorkspace.nodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialWorkspace.edges);
  const [projectName, setProjectName] = useState(initialWorkspace.name);

  const [nodeSpecs, setNodeSpecs] = useState<NodeSpec[]>([]);
  const [apiConnected, setApiConnected] = useState<boolean>(false);
  const [validationReport, setValidationReport] = useState<ValidateResponse | null>(null);
  const [executionReport, setExecutionReport] = useState<ExecutionReport | null>(null);
  const [executionRevision, setExecutionRevision] = useState<string | null>(null);
  const [exportData, setExportData] = useState<ExportResponse | null>(null);
  const [compileReport, setCompileReport] = useState<CompileResponse | null>(null);
  const [logs, setLogs] = useState<UiLog[]>([]);
  const [consoleOpen, setConsoleOpen] = useState<boolean>(true);
  const [lessonsOpen, setLessonsOpen] = useState<boolean>(false);
  const [activeLesson, setActiveLesson] = useState<LessonId | null>(null);
  const [welcomeOpen, setWelcomeOpen] = useState(
    () => window.localStorage.getItem(WELCOME_KEY) !== '1'
  );
  const [mode, setMode] = useState<'graph' | 'lab'>('graph');
  const [languageTarget, setLanguageTarget] = useState<LanguageTarget>('rust');

  const [isValidating, setIsValidating] = useState<boolean>(false);
  const [isExecuting, setIsExecuting] = useState<boolean>(false);
  const [isExporting, setIsExporting] = useState<boolean>(false);
  const [isCodeLoading, setIsCodeLoading] = useState<boolean>(false);
  const [isCompiling, setIsCompiling] = useState<boolean>(false);

  const projectDocument = useMemo(
    () => createProjectDocument(projectName, nodes, edges, languageTarget),
    [edges, languageTarget, nodes, projectName],
  );
  const projectRevision = useMemo(() => JSON.stringify(projectDocument), [projectDocument]);
  const historyRef = useRef({
    current: projectRevision,
    past: [] as string[],
    future: [] as string[],
    pending: null as string | null,
  });
  const historyTimerRef = useRef<number | null>(null);
  const [historyVersion, setHistoryVersion] = useState(0);

  const appendLog = useCallback(
    (key: string, params: Record<string, string | number> = {}, icon = '') => {
      setLogs((previous) => [
        ...previous,
        { id: logCounter++, timestamp: Date.now(), icon, key, params },
      ]);
    },
    []
  );

  const clearRuntimeState = useCallback(() => {
    setExecutionReport(null);
    setExecutionRevision(null);
    setValidationReport(null);
    setExportData(null);
    setCompileReport(null);
  }, []);

  const setWorkspaceFromProject = useCallback(
    (project: ReturnType<typeof parseProjectDocument>) => {
      const workspace = projectToWorkspace(project);
      setProjectName(project.name);
      setNodes(workspace.nodes);
      setEdges(workspace.edges);
      setActiveLesson(null);
      clearRuntimeState();
    },
    [clearRuntimeState, setEdges, setNodes],
  );

  const resetHistory = useCallback((revision: string) => {
    if (historyTimerRef.current !== null) window.clearTimeout(historyTimerRef.current);
    historyRef.current = { current: revision, past: [], future: [], pending: null };
    setHistoryVersion((version) => version + 1);
  }, []);

  const commitPendingHistory = useCallback(() => {
    const history = historyRef.current;
    if (!history.pending || history.pending === history.current) return;
    history.past.push(history.current);
    if (history.past.length > 100) history.past.shift();
    history.current = history.pending;
    history.pending = null;
    history.future = [];
    setHistoryVersion((version) => version + 1);
  }, []);

  useEffect(() => {
    if (projectRevision === historyRef.current.current) return;
    historyRef.current.pending = projectRevision;
    if (historyTimerRef.current !== null) window.clearTimeout(historyTimerRef.current);
    historyTimerRef.current = window.setTimeout(() => {
      historyTimerRef.current = null;
      commitPendingHistory();
    }, 300);
    return () => {
      if (historyTimerRef.current !== null) window.clearTimeout(historyTimerRef.current);
    };
  }, [commitPendingHistory, projectRevision]);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      window.localStorage.setItem(AUTOSAVE_KEY, projectRevision);
    }, 500);
    return () => window.clearTimeout(timer);
  }, [projectRevision]);

  const restoreHistoryRevision = useCallback(
    (revision: string) => {
      const project = parseProjectDocument(revision);
      setWorkspaceFromProject(project);
    },
    [setWorkspaceFromProject],
  );

  const handleUndo = useCallback(() => {
    if (historyTimerRef.current !== null) window.clearTimeout(historyTimerRef.current);
    commitPendingHistory();
    const history = historyRef.current;
    const previous = history.past.pop();
    if (!previous) return;
    history.future.push(history.current);
    history.current = previous;
    history.pending = null;
    restoreHistoryRevision(previous);
    setHistoryVersion((version) => version + 1);
  }, [commitPendingHistory, restoreHistoryRevision]);

  const handleRedo = useCallback(() => {
    if (historyTimerRef.current !== null) window.clearTimeout(historyTimerRef.current);
    commitPendingHistory();
    const history = historyRef.current;
    const next = history.future.pop();
    if (!next) return;
    history.past.push(history.current);
    history.current = next;
    history.pending = null;
    restoreHistoryRevision(next);
    setHistoryVersion((version) => version + 1);
  }, [commitPendingHistory, restoreHistoryRevision]);

  const canUndo = useMemo(
    () => historyRef.current.past.length > 0 || historyRef.current.pending !== null,
    [historyVersion, projectRevision],
  );
  const canRedo = useMemo(
    () => historyRef.current.future.length > 0,
    [historyVersion],
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
  const buildGraphSpec = useCallback((): GraphSpec => projectDocument.graph, [projectDocument]);

  const graphRevision = useMemo(() => JSON.stringify(buildGraphSpec()), [buildGraphSpec]);

  useEffect(() => {
    setExecutionReport(null);
    setExecutionRevision(null);
    setValidationReport(null);
    setCompileReport(null);
  }, [graphRevision]);

  // 3. Debounced Live Code Compilation from API
  const refreshCode = useCallback(async () => {
    setIsCodeLoading(true);
    try {
      const graph = buildGraphSpec();
      const res = await fetch('/api/export', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          graph,
          project_name: 'polyglot_lab_demo',
          target: languageTarget,
        }),
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
  }, [buildGraphSpec, languageTarget]);

  useEffect(() => {
    const timer = setTimeout(() => {
      refreshCode();
    }, 400);
    return () => clearTimeout(timer);
  }, [refreshCode]);

  const handleCompile = useCallback(async () => {
    setIsCompiling(true);
    setCompileReport(null);
    try {
      const response = await fetch('/api/compile', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ graph: buildGraphSpec(), project_name: projectName }),
      });
      if (response.ok) {
        setCompileReport(await response.json() as CompileResponse);
      } else {
        const error = await response.json() as { error?: string };
        setCompileReport({ success: false, stdout: '', stderr: error.error ?? 'Compile failed.' });
      }
    } catch (error) {
      setCompileReport({
        success: false,
        stdout: '',
        stderr: error instanceof Error ? error.message : 'Compile request failed.',
      });
    } finally {
      setIsCompiling(false);
    }
  }, [buildGraphSpec, projectName]);

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
    setExecutionReport(null);
    setExecutionRevision(null);
    try {
      const graph = buildGraphSpec();
      const revision = JSON.stringify(graph);
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
      setExecutionRevision(revision);

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

  const handleNewProject = useCallback(() => {
    const project = createProjectDocument('Untitled Project', [], [], languageTarget);
    setWorkspaceFromProject(project);
    resetHistory(JSON.stringify(project));
    appendLog('log.projectCreated', {}, '📄');
  }, [appendLog, languageTarget, resetHistory, setWorkspaceFromProject]);

  const handleOpenProject = useCallback(async (file: File) => {
    try {
      if (file.size > MAX_PROJECT_FILE_BYTES) throw new Error('project-too-large');
      const project = parseProjectDocument(await file.text());
      if (project.target) {
        setLanguageTarget(project.target);
      }
      const workspace = projectToWorkspace(project);
      const normalized = createProjectDocument(project.name, workspace.nodes, workspace.edges, project.target || languageTarget);
      setWorkspaceFromProject(normalized);
      resetHistory(JSON.stringify(normalized));
      window.localStorage.setItem(AUTOSAVE_KEY, JSON.stringify(normalized));
      appendLog('log.projectOpened', { name: normalized.name }, '📂');
    } catch {
      appendLog('log.projectOpenFailed', {}, '❌');
    }
  }, [appendLog, languageTarget, resetHistory, setWorkspaceFromProject]);

  const handleSaveProject = useCallback(() => {
    const normalizedName = normalizeProjectName(projectName);
    const project = createProjectDocument(normalizedName, nodes, edges, languageTarget);
    setProjectName(normalizedName);
    downloadBlob(
      new Blob([`${JSON.stringify(project, null, 2)}\n`], { type: 'application/json' }),
      projectFileName(normalizedName),
    );
    appendLog('log.projectSaved', { name: normalizedName }, '💾');
  }, [appendLog, edges, languageTarget, nodes, projectName]);

  // 10. Download a complete standalone project for the active language target.
  const handleExportCargo = async () => {
    setIsExporting(true);
    setConsoleOpen(true);
    try {
      const graph = buildGraphSpec();
      const res = await fetch('/api/export', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          graph,
          project_name: projectName,
          target: languageTarget,
        }),
      });

      if (res.ok) {
        const data: ExportResponse = await res.json();
        const root = data.project_name;
        const zipFiles: Record<string, string> = {};

        if (data.files && data.files.length > 0) {
          for (const file of data.files) {
            zipFiles[`${root}/${file.path}`] = file.content;
          }
        } else {
          zipFiles[`${root}/Cargo.toml`] = data.cargo_toml || '';
          zipFiles[`${root}/src/main.rs`] = data.main_rs || '';
          zipFiles[`${root}/src/nodes.rs`] = data.nodes_rs || '';
        }

        zipFiles[`${root}/README.md`] = `# ${projectName}\n\nGenerated by Polyglot Lab (${languageTarget.toUpperCase()}).\n`;

        const archive = createStoredZip(zipFiles);
        const archiveBuffer = archive.buffer.slice(
          archive.byteOffset,
          archive.byteOffset + archive.byteLength,
        ) as ArrayBuffer;
        downloadBlob(
          new Blob([archiveBuffer], { type: 'application/zip' }),
          `${root}.zip`,
        );
        appendLog('log.exportDownloaded', { file: `${root}.zip` }, '🎉');
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

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey)) return;
      const target = event.target as HTMLElement | null;
      const editingText =
        target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA' || target?.isContentEditable;

      if (event.key.toLowerCase() === 's') {
        event.preventDefault();
        handleSaveProject();
      } else if (!editingText && event.key.toLowerCase() === 'z') {
        event.preventDefault();
        if (event.shiftKey) handleRedo();
        else handleUndo();
      } else if (!editingText && event.key.toLowerCase() === 'y') {
        event.preventDefault();
        handleRedo();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [handleRedo, handleSaveProject, handleUndo]);

  const handleStartLesson = useCallback((lesson: LessonId) => {
    const setup = createLessonSetup(lesson);
    setNodes(setup.nodes);
    setEdges(setup.edges);
    setExecutionReport(null);
    setExecutionRevision(null);
    setValidationReport(null);
    setExportData(null);
    setActiveLesson(lesson);
    setLessonsOpen(true);
  }, [setEdges, setNodes]);

  const handleResetLesson = useCallback(() => {
    if (activeLesson) handleStartLesson(activeLesson);
  }, [activeLesson, handleStartLesson]);

  const dismissWelcome = useCallback(() => {
    window.localStorage.setItem(WELCOME_KEY, '1');
    setWelcomeOpen(false);
  }, []);

  // 12. Load Ready-to-Run Examples
  const handleLoadExample = (example: 'condition' | 'text' | 'http') => {
    setActiveLesson(null);
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

  const currentGraph = buildGraphSpec();
  const currentExecutionReport =
    executionRevision === graphRevision ? executionReport : null;

  return (
    <div className="flex flex-col h-screen w-screen bg-slate-950 font-sans text-slate-100 select-none">
      {welcomeOpen && mode === 'graph' && (
        <WelcomeScreen
          onExplore={dismissWelcome}
          onStartLesson={() => {
            dismissWelcome();
            handleStartLesson('text_pipeline');
          }}
        />
      )}

      {/* Top Navigation & Actions */}
      <Toolbar
        mode={mode}
        onModeChange={setMode}
        target={languageTarget}
        onTargetChange={setLanguageTarget}
        graph={currentGraph}
        projectName={projectName}
        onProjectNameChange={setProjectName}
        onNewProject={handleNewProject}
        onOpenProject={handleOpenProject}
        onSaveProject={handleSaveProject}
        onUndo={handleUndo}
        onRedo={handleRedo}
        canUndo={canUndo}
        canRedo={canRedo}
        onValidate={handleValidate}
        onExecute={handleExecute}
        onExportCargo={handleExportCargo}
        onToggleLessons={() => setLessonsOpen((open) => !open)}
        onLoadExample={handleLoadExample}
        onFocusNode={handleFocusNode}
        apiConnected={apiConnected}
        isValidating={isValidating}
        isExecuting={isExecuting}
        isExporting={isExporting}
        lessonsOpen={lessonsOpen}
      />

      {/* Main Workspace */}
      {mode === 'lab' ? (
        <React.Suspense fallback={<div className="flex flex-1 items-center justify-center bg-slate-950 text-sm text-slate-400">{t('lab.loading')}</div>}>
          <WorkspaceLab />
        </React.Suspense>
      ) : <div className="flex flex-1 overflow-hidden">
        {/* Left Sidebar: Node Palette */}
        <Sidebar nodeSpecs={nodeSpecs} onAddNode={addNodeWithKind} />

        {/* Center: Interactive Visual Canvas */}
        <div className="flex-1 flex flex-col h-full relative" onDrop={onDrop} onDragOver={onDragOver}>
          {lessonsOpen && (
            <GuidedLessons
              activeLesson={activeLesson}
              graph={currentGraph}
              executionReport={currentExecutionReport}
              onStart={handleStartLesson}
              onReset={handleResetLesson}
              onExit={() => setActiveLesson(null)}
              onClose={() => setLessonsOpen(false)}
            />
          )}
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

        {/* Right Pane: Live Generated Polyglot Code */}
        <CodeView
          target={languageTarget}
          exportData={exportData}
          compileReport={compileReport}
          loading={isCodeLoading}
          compiling={isCompiling}
          onRefresh={refreshCode}
          onCompile={handleCompile}
        />
      </div>}
    </div>
  );
};

export default App;
