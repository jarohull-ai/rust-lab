import React, { useState } from 'react';
import { useI18n } from '../i18n';
import CodeMirror from '@uiw/react-codemirror';
import { rust } from '@codemirror/lang-rust';
import { CompileResponse, ExportResponse } from '../types';
import { Code2, Copy, Check, FileCode, Package, RefreshCw, Hammer, XCircle } from 'lucide-react';

interface CodeViewProps {
  exportData: ExportResponse | null;
  compileReport: CompileResponse | null;
  loading: boolean;
  compiling: boolean;
  onRefresh: () => void;
  onCompile: () => void;
}

export const CodeView: React.FC<CodeViewProps> = ({
  exportData,
  compileReport,
  loading,
  compiling,
  onRefresh,
  onCompile,
}) => {
  const { t } = useI18n();
  const [activeTab, setActiveTab] = useState<'main' | 'nodes' | 'cargo'>('main');
  const [copied, setCopied] = useState(false);

  const getCurrentCode = () => {
    if (!exportData) return t('code.empty');
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
          <span className="font-semibold text-xs tracking-wide text-slate-100">{t('code.title')}</span>
        </div>

        <div className="flex items-center gap-1.5">
          <button
            type="button"
            onClick={onCompile}
            disabled={compiling || !exportData}
            className="flex items-center gap-1 rounded bg-cyan-600/15 px-2 py-1 text-[10px] font-semibold text-cyan-300 transition-colors hover:bg-cyan-600/25 disabled:opacity-40"
            title={t('code.compileTitle')}
          >
            <Hammer className={`h-3 w-3 ${compiling ? 'animate-pulse' : ''}`} />
            {compiling ? t('code.checking') : t('code.check')}
          </button>
          <button
            onClick={onRefresh}
            disabled={loading}
            className="p-1 rounded text-slate-400 hover:text-slate-100 hover:bg-slate-800 transition-colors disabled:opacity-40"
            title={t('code.refresh')}
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
                <span className="text-emerald-400">{t('code.copied')}</span>
              </>
            ) : (
              <>
                <Copy className="w-3 h-3" />
                <span>{t('code.copy')}</span>
              </>
            )}
          </button>
        </div>
      </div>

      {compileReport && (
        <div
          className={`border-b px-3 py-2 text-[10px] ${
            compileReport.success
              ? 'border-emerald-800 bg-emerald-950/40 text-emerald-300'
              : 'border-rose-800 bg-rose-950/40 text-rose-200'
          }`}
          role="status"
        >
          <div className="flex items-center gap-1.5 font-semibold">
            {compileReport.success ? (
              <Check className="h-3.5 w-3.5" />
            ) : (
              <XCircle className="h-3.5 w-3.5" />
            )}
            {t(compileReport.success ? 'code.compileSuccess' : 'code.compileFailed')}
          </div>
          {!compileReport.success && (compileReport.stderr || compileReport.stdout) && (
            <pre className="mt-1.5 max-h-28 overflow-auto whitespace-pre-wrap rounded bg-slate-950/70 p-2 font-mono text-[9px] leading-relaxed text-slate-300">
              {compileReport.stderr || compileReport.stdout}
            </pre>
          )}
        </div>
      )}

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
        <span>{t('code.footer')}</span>
        <span className="text-emerald-500">{t('code.standalone')}</span>
      </div>
    </div>
  );
};
