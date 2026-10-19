import React, { useState } from 'react';
import CodeMirror from '@uiw/react-codemirror';
import { rust } from '@codemirror/lang-rust';
import { ExportResponse } from '../types';
import { Code2, Copy, Check, FileCode, Package, RefreshCw } from 'lucide-react';

interface CodeViewProps {
  exportData: ExportResponse | null;
  loading: boolean;
  onRefresh: () => void;
}

export const CodeView: React.FC<CodeViewProps> = ({ exportData, loading, onRefresh }) => {
  const [activeTab, setActiveTab] = useState<'main' | 'nodes' | 'cargo'>('main');
  const [copied, setCopied] = useState(false);

  const getCurrentCode = () => {
    if (!exportData) return '// Construct a valid graph on the canvas to see generated Rust code.';
    switch (activeTab) {
      case 'main':
        return exportData.main_rs;
      case 'nodes':
        return exportData.nodes_rs;
      case 'cargo':
        return exportData.cargo_toml;
    }
  };

  const copyCode = async () => {
    const code = getCurrentCode();
    await navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="w-[480px] bg-slate-900/95 border-l border-slate-800 flex flex-col h-full overflow-hidden select-none">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-slate-800 bg-slate-950/60">
        <div className="flex items-center gap-2">
          <Code2 className="w-4 h-4 text-cyan-400" />
          <span className="font-semibold text-xs tracking-wide text-slate-100">Live Rust Code View</span>
        </div>

        <div className="flex items-center gap-1.5">
          <button
            onClick={onRefresh}
            disabled={loading}
            className="p-1 rounded text-slate-400 hover:text-slate-100 hover:bg-slate-800 transition-colors disabled:opacity-40"
            title="Refresh Code"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
          </button>

          <button
            onClick={copyCode}
            className="flex items-center gap-1 text-[11px] px-2 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 transition-colors"
          >
            {copied ? (
              <>
                <Check className="w-3 h-3 text-emerald-400" />
                <span className="text-emerald-400">Copied</span>
              </>
            ) : (
              <>
                <Copy className="w-3 h-3" />
                <span>Copy</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-slate-800 bg-slate-950/40 px-2 pt-1 gap-1">
        <button
          onClick={() => setActiveTab('main')}
          className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-mono rounded-t-md transition-colors ${
            activeTab === 'main'
              ? 'bg-slate-900 text-cyan-400 border-t-2 border-cyan-400'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-900/50'
          }`}
        >
          <FileCode className="w-3.5 h-3.5" />
          <span>src/main.rs</span>
        </button>

        <button
          onClick={() => setActiveTab('nodes')}
          className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-mono rounded-t-md transition-colors ${
            activeTab === 'nodes'
              ? 'bg-slate-900 text-orange-400 border-t-2 border-orange-400'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-900/50'
          }`}
        >
          <FileCode className="w-3.5 h-3.5" />
          <span>src/nodes.rs</span>
        </button>

        <button
          onClick={() => setActiveTab('cargo')}
          className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-mono rounded-t-md transition-colors ${
            activeTab === 'cargo'
              ? 'bg-slate-900 text-emerald-400 border-t-2 border-emerald-400'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-900/50'
          }`}
        >
          <Package className="w-3.5 h-3.5" />
          <span>Cargo.toml</span>
        </button>
      </div>

      {/* Code Editor */}
      <div className="flex-1 overflow-auto text-xs font-mono bg-slate-950">
        <CodeMirror
          value={getCurrentCode()}
          height="100%"
          theme="dark"
          extensions={[rust()]}
          editable={false}
          basicSetup={{
            lineNumbers: true,
            foldGutter: true,
            highlightActiveLine: false,
          }}
          className="h-full"
        />
      </div>

      <div className="p-2 border-t border-slate-800 text-[10px] text-slate-500 font-mono bg-slate-950/80 flex items-center justify-between">
        <span>⚡ Real-time code compilation synced from canvas</span>
        <span className="text-emerald-500">Standalone Project</span>
      </div>
    </div>
  );
};
