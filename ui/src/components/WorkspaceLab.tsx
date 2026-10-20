import React, { useCallback, useEffect, useMemo, useState } from 'react';
import CodeMirror from '@uiw/react-codemirror';
import { rust } from '@codemirror/lang-rust';
import { go } from '@codemirror/lang-go';
import { python } from '@codemirror/lang-python';
import { cpp } from '@codemirror/lang-cpp';
import {
  Beaker,
  Box,
  Check,
  ChevronRight,
  CircleAlert,
  FileCode2,
  FileText,
  Folder,
  GitFork,
  LoaderCircle,
  PackagePlus,
  Play,
  RefreshCw,
  Save,
  Search,
  ShieldCheck,
  TerminalSquare,
  TestTube2,
  WandSparkles,
} from 'lucide-react';
import { useI18n } from '../i18n';
import {
  CargoCommandResponse,
  CrateSummary,
  LabFile,
  LabProject,
  LanguageTarget,
  SandboxCapabilities,
  WorkspaceEntry,
  WorkspaceTree,
} from '../types';

type CargoAction = CargoCommandResponse['action'];

async function apiJson<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await fetch(url, init);
  const body = await response.json().catch(() => ({}));
  if (!response.ok) throw new Error((body as { error?: string }).error ?? `HTTP ${response.status}`);
  return body as T;
}

interface WorkspaceLabProps {
  target?: LanguageTarget;
}

export const WorkspaceLab: React.FC<WorkspaceLabProps> = ({ target = 'rust' }) => {
  const { t } = useI18n();
  const [projects, setProjects] = useState<LabProject[]>([]);
  const [activeProject, setActiveProject] = useState<LabProject | null>(null);
  const [entries, setEntries] = useState<WorkspaceEntry[]>([]);
  const [selectedPath, setSelectedPath] = useState('');
  const [content, setContent] = useState('');
  const [savedContent, setSavedContent] = useState('');

  const defaultNewName = useMemo(() => {
    switch (target) {
      case 'go': return 'Go Prototype';
      case 'python': return 'Python Script';
      case 'cpp': return 'C++ Project';
      case 'rust':
      default: return 'Rust Prototype';
    }
  }, [target]);

  const [newName, setNewName] = useState(defaultNewName);

  useEffect(() => {
    setNewName(defaultNewName);
  }, [defaultNewName]);

  const [githubUrl, setGithubUrl] = useState('');
  const [githubRef, setGithubRef] = useState('');
  const [crateQuery, setCrateQuery] = useState('');
  const [crateResults, setCrateResults] = useState<CrateSummary[]>([]);
  const [output, setOutput] = useState(t('lab.console.empty'));
  const [busy, setBusy] = useState('');
  const [error, setError] = useState('');
  const [capabilities, setCapabilities] = useState<SandboxCapabilities | null>();

  const dirty = content !== savedContent;
  const sourceFiles = useMemo(() => entries.filter((entry) => entry.kind === 'file'), [entries]);
  const offlineReady = capabilities?.offline_execution_available === true;
  const fetchReady = capabilities?.dependency_fetch_available === true;

  const refreshProjects = useCallback(async () => {
    const data = await apiJson<LabProject[]>('/api/lab/projects');
    setProjects(data);
    setActiveProject((current) => current ?? data[0] ?? null);
  }, []);

  const refreshCapabilities = useCallback(async () => {
    try {
      setCapabilities(await apiJson<SandboxCapabilities>('/api/lab/capabilities'));
    } catch {
      setCapabilities(null);
    }
  }, []);

  const refreshTree = useCallback(async (project: LabProject) => {
    const tree = await apiJson<WorkspaceTree>(`/api/lab/projects/${project.id}/tree`);
    setEntries(tree.entries);
    setActiveProject(tree.project);
  }, []);

  useEffect(() => {
    refreshProjects().catch((cause) => setError(cause instanceof Error ? cause.message : String(cause)));
    refreshCapabilities();
  }, [refreshCapabilities, refreshProjects]);

  useEffect(() => {
    if (!activeProject) return;
    setSelectedPath('');
    setContent('');
    setSavedContent('');
    refreshTree(activeProject).catch((cause) => setError(cause instanceof Error ? cause.message : String(cause)));
  }, [activeProject?.id, refreshTree]);

  const createProject = async () => {
    if (!newName.trim()) return;
    setBusy('create');
    setError('');
    try {
      const project = await apiJson<LabProject>('/api/lab/projects', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: newName, target }),
      });
      await refreshProjects();
      setActiveProject(project);
      setOutput(t('lab.console.created', { name: project.name }));
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy('');
    }
  };

  const importGithub = async () => {
    setBusy('github');
    setError('');
    try {
      const project = await apiJson<LabProject>('/api/lab/import/github', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ url: githubUrl, reference: githubRef }),
      });
      await refreshProjects();
      setActiveProject(project);
      setGithubUrl('');
      setOutput(t('lab.console.imported', { name: project.name }));
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy('');
    }
  };

  const openFile = async (path: string) => {
    if (!activeProject || (dirty && !window.confirm(t('lab.confirmDiscard')))) return;
    setBusy('file');
    setError('');
    try {
      const file = await apiJson<LabFile>(
        `/api/lab/file?project_id=${encodeURIComponent(activeProject.id)}&path=${encodeURIComponent(path)}`,
      );
      setSelectedPath(file.path);
      setContent(file.content);
      setSavedContent(file.content);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy('');
    }
  };

  const saveFile = async () => {
    if (!activeProject || !selectedPath) return;
    setBusy('save');
    setError('');
    try {
      const file = await apiJson<LabFile>('/api/lab/file', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ project_id: activeProject.id, path: selectedPath, content }),
      });
      setSavedContent(file.content);
      setOutput(t('lab.console.saved', { path: file.path }));
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy('');
    }
  };

  const getToolchainActions = () => {
    switch (target) {
      case 'go':
        return [
          { action: 'fetch' as CargoAction, label: 'tidy', icon: PackagePlus, cmd: 'go mod tidy' },
          { action: 'check' as CargoAction, label: 'build', icon: Check, cmd: 'go build ./...' },
          { action: 'test' as CargoAction, label: 'test', icon: TestTube2, cmd: 'go test ./...' },
          { action: 'run' as CargoAction, label: 'run', icon: Play, cmd: 'go run .' },
          { action: 'clippy' as CargoAction, label: 'vet', icon: ShieldCheck, cmd: 'go vet ./...' },
          { action: 'fmt' as CargoAction, label: 'fmt', icon: WandSparkles, cmd: 'gofmt -w .' },
        ];
      case 'python':
        return [
          { action: 'fetch' as CargoAction, label: 'sync', icon: PackagePlus, cmd: 'pip install -r requirements.txt' },
          { action: 'check' as CargoAction, label: 'check', icon: Check, cmd: 'python3 -m py_compile main.py' },
          { action: 'test' as CargoAction, label: 'test', icon: TestTube2, cmd: 'python3 -m unittest' },
          { action: 'run' as CargoAction, label: 'run', icon: Play, cmd: 'python3 main.py' },
          { action: 'clippy' as CargoAction, label: 'ruff', icon: ShieldCheck, cmd: 'ruff check .' },
          { action: 'fmt' as CargoAction, label: 'fmt', icon: WandSparkles, cmd: 'black .' },
        ];
      case 'cpp':
        return [
          { action: 'fetch' as CargoAction, label: 'cmake', icon: PackagePlus, cmd: 'cmake -B build' },
          { action: 'check' as CargoAction, label: 'build', icon: Check, cmd: 'cmake --build build' },
          { action: 'test' as CargoAction, label: 'test', icon: TestTube2, cmd: 'ctest --test-dir build' },
          { action: 'run' as CargoAction, label: 'run', icon: Play, cmd: './build/app' },
          { action: 'clippy' as CargoAction, label: 'tidy', icon: ShieldCheck, cmd: 'clang-tidy' },
          { action: 'fmt' as CargoAction, label: 'fmt', icon: WandSparkles, cmd: 'clang-format -i' },
        ];
      case 'rust':
      default:
        return [
          { action: 'fetch' as CargoAction, label: 'fetch', icon: PackagePlus, cmd: 'cargo fetch' },
          { action: 'check' as CargoAction, label: 'check', icon: Check, cmd: 'cargo check' },
          { action: 'test' as CargoAction, label: 'test', icon: TestTube2, cmd: 'cargo test' },
          { action: 'run' as CargoAction, label: 'run', icon: Play, cmd: 'cargo run' },
          { action: 'clippy' as CargoAction, label: 'clippy', icon: ShieldCheck, cmd: 'cargo clippy' },
          { action: 'fmt' as CargoAction, label: 'fmt', icon: WandSparkles, cmd: 'cargo fmt' },
        ];
    }
  };

  const runCommand = async (actionItem: { action: CargoAction; label: string; cmd: string }) => {
    if (!activeProject || !window.confirm(t('lab.executionApproval', { action: actionItem.label }))) return;
    setBusy(actionItem.action);
    setError('');
    setOutput(`$ ${actionItem.cmd}\n`);
    try {
      const result = await apiJson<CargoCommandResponse>('/api/lab/cargo', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ project_id: activeProject.id, action: actionItem.action, approved: true }),
      });
      setOutput(
        `$ ${actionItem.cmd}\n${result.stdout}${result.stderr}\n[${result.success ? 'ok' : 'failed'} · ${result.duration_ms} ms · sandbox: ${result.sandbox}]`,
      );
      await refreshTree(activeProject);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy('');
    }
  };

  const searchPackages = async () => {
    setBusy('search');
    setError('');
    try {
      const result = await apiJson<{ crates: CrateSummary[] }>(
        `/api/lab/crates/search?q=${encodeURIComponent(crateQuery)}&target=${target}`,
      );
      setCrateResults(result.crates);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy('');
    }
  };

  const addPackage = async (packageItem: CrateSummary) => {
    if (!activeProject || !window.confirm(t('lab.dependencyApproval', { name: packageItem.name, version: packageItem.max_version }))) return;
    setBusy(`pkg:${packageItem.name}`);
    setError('');
    try {
      const result = await apiJson<{ success: boolean; stdout: string; stderr: string }>(
        '/api/lab/dependencies',
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            project_id: activeProject.id,
            crate_name: packageItem.name,
            version: packageItem.max_version,
            features: [],
            default_features: true,
            approved: true,
          }),
        },
      );
      setOutput(`Added package: ${packageItem.name}@${packageItem.max_version}\n${result.stdout}${result.stderr}`);
      await refreshTree(activeProject);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy('');
    }
  };

  const getCodeMirrorExtension = (path: string) => {
    if (path.endsWith('.go')) return [go()];
    if (path.endsWith('.py')) return [python()];
    if (path.endsWith('.cpp') || path.endsWith('.hpp') || path.endsWith('.h')) return [cpp()];
    if (path.endsWith('.rs')) return [rust()];
    return [];
  };

  const getPackageRegistryInfo = () => {
    switch (target) {
      case 'go':
        return { title: 'pkg.go.dev (Go Modules)', placeholder: 'Search Go modules (e.g. gin, zap, gorm)...' };
      case 'python':
        return { title: 'pypi.org (Python Packages)', placeholder: 'Search PyPI (e.g. requests, fastapi)...' };
      case 'cpp':
        return { title: 'C++ Libraries (Registry)', placeholder: 'Search C++ libraries (e.g. json, fmt)...' };
      case 'rust':
      default:
        return { title: 'crates.io (Rust Crates)', placeholder: t('lab.searchCrates') };
    }
  };

  const packageInfo = getPackageRegistryInfo();
  const toolchainActions = getToolchainActions();

  return (
    <main className="flex min-h-0 flex-1 bg-slate-950 text-slate-100">
      <aside className="flex w-72 shrink-0 flex-col border-r border-slate-800 bg-slate-900/80">
        <div className="border-b border-slate-800 p-3">
          <div className="mb-2 flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-orange-300">
            <Beaker className="h-4 w-4" /> {t('lab.projects')} ({target.toUpperCase()})
          </div>
          <div className="flex gap-1.5">
            <input value={newName} onChange={(event) => setNewName(event.target.value)} maxLength={80} className="min-w-0 flex-1 rounded border border-slate-700 bg-slate-950 px-2 py-1.5 text-xs" aria-label={t('lab.newName')} />
            <button type="button" onClick={createProject} disabled={busy !== ''} className="rounded bg-orange-600 px-2.5 text-xs font-semibold hover:bg-orange-500 disabled:opacity-40">{t('lab.new')}</button>
          </div>
        </div>

        <div className="border-b border-slate-800 p-3">
          <div className="mb-2 flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wider text-slate-400"><GitFork className="h-3.5 w-3.5" />{t('lab.githubImport')}</div>
          <input value={githubUrl} onChange={(event) => setGithubUrl(event.target.value)} placeholder="https://github.com/owner/repo" className="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1.5 text-[11px]" />
          <div className="mt-1.5 flex gap-1.5">
            <input value={githubRef} onChange={(event) => setGithubRef(event.target.value)} placeholder={t('lab.branchOptional')} className="min-w-0 flex-1 rounded border border-slate-700 bg-slate-950 px-2 py-1.5 text-[11px]" />
            <button type="button" onClick={importGithub} disabled={!githubUrl || busy !== ''} className="rounded border border-slate-700 bg-slate-800 px-2 text-[10px] font-semibold hover:bg-slate-700 disabled:opacity-40">{busy === 'github' ? '…' : t('lab.import')}</button>
          </div>
        </div>

        <div className="border-b border-slate-800 p-2">
          <select value={activeProject?.id ?? ''} onChange={(event) => setActiveProject(projects.find((item) => item.id === event.target.value) ?? null)} className="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1.5 text-xs">
            <option value="">{t('lab.selectProject')}</option>
            {projects.map((project) => <option key={project.id} value={project.id}>{project.name}</option>)}
          </select>
        </div>

        <div className="min-h-0 flex-1 overflow-auto p-2 font-mono text-[11px]">
          {entries.map((entry) => (
            <button key={entry.path} type="button" disabled={entry.kind === 'directory'} onClick={() => openFile(entry.path)} className={`flex w-full items-center gap-1.5 rounded px-2 py-1 text-left ${selectedPath === entry.path ? 'bg-cyan-500/15 text-cyan-300' : 'text-slate-400 hover:bg-slate-800 hover:text-slate-100'} disabled:hover:bg-transparent`}>
              {entry.kind === 'directory' ? (
                <Folder className="h-3.5 w-3.5 text-yellow-500" />
              ) : entry.path.endsWith('.rs') || entry.path.endsWith('.go') || entry.path.endsWith('.py') || entry.path.endsWith('.cpp') ? (
                <FileCode2 className="h-3.5 w-3.5 text-cyan-400" />
              ) : (
                <FileText className="h-3.5 w-3.5 text-slate-500" />
              )}
              <span className="truncate">{entry.path}</span>
            </button>
          ))}
          {activeProject && sourceFiles.length === 0 && <p className="p-2 text-slate-500">{t('lab.noFiles')}</p>}
        </div>
      </aside>

      <section className="flex min-w-0 flex-1 flex-col">
        <div className="flex h-12 items-center justify-between border-b border-slate-800 bg-slate-900/60 px-3">
          <div className="flex min-w-0 items-center gap-2">
            <FileCode2 className="h-4 w-4 text-orange-400" />
            <span className="truncate font-mono text-xs text-slate-300">{selectedPath || t('lab.chooseFile')}</span>
            {dirty && <span className="rounded bg-yellow-500/15 px-1.5 py-0.5 text-[9px] font-bold text-yellow-300">{t('lab.modified')}</span>}
          </div>
          <button type="button" onClick={saveFile} disabled={!dirty || busy !== ''} className="flex items-center gap-1.5 rounded bg-cyan-600 px-2.5 py-1.5 text-xs font-semibold hover:bg-cyan-500 disabled:opacity-30"><Save className="h-3.5 w-3.5" />{t('lab.save')}</button>
        </div>
        <div className="min-h-0 flex-1 bg-slate-950">
          {selectedPath ? (
            <CodeMirror value={content} onChange={setContent} height="100%" theme="dark" extensions={getCodeMirrorExtension(selectedPath)} className="h-full text-sm" basicSetup={{ lineNumbers: true, foldGutter: true }} />
          ) : (
            <div className="flex h-full flex-col items-center justify-center text-slate-600"><WandSparkles className="mb-3 h-10 w-10 text-orange-500/40" /><p className="text-sm">{t('lab.emptyEditor')}</p></div>
          )}
        </div>
        <div className="h-52 shrink-0 border-t border-slate-800 bg-slate-950">
          <div className="flex h-8 items-center justify-between border-b border-slate-800 px-3 text-[10px] font-mono text-slate-400">
            <span className="flex items-center gap-1.5"><TerminalSquare className="h-3 w-3" />{t('lab.console')}</span>
            {busy && <span className="flex items-center gap-1 text-cyan-400"><LoaderCircle className="h-3 w-3 animate-spin" />{busy}…</span>}
          </div>
          <pre className="h-[calc(100%-2rem)] overflow-auto p-3 font-mono text-xs leading-relaxed text-slate-300 whitespace-pre-wrap">{output}</pre>
        </div>
      </section>

      <aside className="flex w-80 shrink-0 flex-col border-l border-slate-800 bg-slate-900/80">
        <div className="border-b border-slate-800 p-3">
          <div className="mb-2 flex items-center justify-between">
            <span className="text-xs font-bold uppercase tracking-wider text-slate-300">Toolchain ({target.toUpperCase()})</span>
            {error && <span className="flex items-center gap-1 text-[10px] text-rose-400" title={error}><CircleAlert className="h-3 w-3" />{error}</span>}
          </div>
          <div className={`mb-3 rounded border p-2 text-[10px] ${capabilities === undefined ? 'border-slate-700 bg-slate-950/60 text-slate-400' : offlineReady ? 'border-emerald-800/80 bg-emerald-950/30 text-emerald-300' : 'border-amber-800/80 bg-amber-950/30 text-amber-300'}`}>
            <div className="flex items-center justify-between gap-1">
              <span className="font-semibold">{capabilities === undefined ? t('lab.sandboxChecking') : offlineReady ? t('lab.sandboxReady', { backend: capabilities?.preferred_backend ?? '' }) : t('lab.sandboxUnavailable')}</span>
              <span className="rounded bg-black/30 px-1 py-0.5 text-[9px] font-mono">{t('lab.sandboxed')}</span>
            </div>
            <div className="mt-1 flex items-center justify-between gap-1 text-[9px] opacity-80">
              <span className="truncate">
                {capabilities === undefined
                  ? t('lab.sandboxChecking')
                  : offlineReady
                    ? t('lab.sandboxReadyDetail', { backend: capabilities?.preferred_backend ?? '' })
                    : t('lab.sandboxUnavailableHelp')}
              </span>
              {capabilities !== undefined && (
                <button type="button" onClick={() => { setCapabilities(undefined); refreshCapabilities(); }} className="rounded p-0.5 hover:bg-white/10" title={t('lab.sandboxRetry')}>
                  <RefreshCw className="h-3.5 w-3.5" />
                </button>
              )}
            </div>
            {capabilities && !fetchReady && <p className="mt-1 border-t border-current/10 pt-1 opacity-80">{t('lab.fetchUnavailable')}</p>}
          </div>
          <div className="grid grid-cols-3 gap-1.5">
            {toolchainActions.map((item) => {
              const Icon = item.icon;
              return (
                <button
                  key={item.label}
                  type="button"
                  onClick={() => runCommand(item)}
                  disabled={!activeProject || busy !== '' || (item.action === 'fetch' ? !fetchReady : !offlineReady)}
                  className="flex items-center justify-center gap-1 rounded border border-slate-700 bg-slate-800 px-2 py-2 text-[10px] font-semibold text-slate-200 hover:border-emerald-500/60 hover:bg-slate-700 disabled:opacity-35"
                  title={item.cmd}
                >
                  <Icon className={`h-3.5 w-3.5 ${busy === item.action ? 'animate-spin' : ''}`} />
                  {item.label}
                </button>
              );
            })}
            <button type="button" onClick={() => activeProject && refreshTree(activeProject)} disabled={!activeProject || busy !== ''} className="flex items-center justify-center gap-1 rounded border border-slate-700 bg-slate-800 px-2 py-2 text-[10px] font-semibold text-slate-200 hover:bg-slate-700 disabled:opacity-35"><RefreshCw className="h-3.5 w-3.5" />{t('lab.refresh')}</button>
          </div>
          <p className="mt-2 text-[9px] leading-relaxed text-slate-500">{t('lab.sandboxNote')}</p>
        </div>

        <div className="flex min-h-0 flex-1 flex-col p-3">
          <div className="mb-2 flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-cyan-300">
            <Box className="h-4 w-4" /> {packageInfo.title}
          </div>
          <div className="flex gap-1.5">
            <input
              value={crateQuery}
              onChange={(event) => setCrateQuery(event.target.value)}
              onKeyDown={(event) => event.key === 'Enter' && searchPackages()}
              placeholder={packageInfo.placeholder}
              className="min-w-0 flex-1 rounded border border-slate-700 bg-slate-950 px-2 py-1.5 text-xs"
            />
            <button
              type="button"
              onClick={searchPackages}
              disabled={crateQuery.length < 2 || busy !== ''}
              className="rounded bg-cyan-700 px-2 hover:bg-cyan-600 disabled:opacity-35"
              title={t('lab.search')}
            >
              <Search className={`h-4 w-4 ${busy === 'search' ? 'animate-spin' : ''}`} />
            </button>
          </div>
          <div className="mt-2 min-h-0 flex-1 overflow-auto">
            {crateResults.map((crateItem) => (
              <article key={crateItem.name} className="mb-1.5 rounded border border-slate-800 bg-slate-950/60 p-2">
                <div className="flex items-center justify-between gap-2">
                  <div className="min-w-0">
                    <div className="truncate font-mono text-xs font-semibold text-cyan-300">{crateItem.name}</div>
                    <div className="text-[9px] text-slate-500">v{crateItem.max_version} · {crateItem.downloads.toLocaleString()} downloads</div>
                  </div>
                  <button
                    type="button"
                    onClick={() => addPackage(crateItem)}
                    disabled={!activeProject || busy !== ''}
                    className="flex shrink-0 items-center gap-1 rounded bg-emerald-600/20 px-2 py-1 text-[9px] font-bold text-emerald-300 hover:bg-emerald-600/30 disabled:opacity-35"
                  >
                    <PackagePlus className="h-3 w-3" />{t('lab.add')}
                  </button>
                </div>
                <p className="mt-1 line-clamp-2 text-[10px] leading-relaxed text-slate-400">{crateItem.description}</p>
              </article>
            ))}
          </div>
        </div>
        <div className="border-t border-slate-800 p-2 text-[9px] text-slate-500"><ChevronRight className="mr-1 inline h-3 w-3" />{t('lab.publicOnly')}</div>
      </aside>
    </main>
  );
};
