import React, { useState, useEffect } from 'react';
import { useI18n } from '../i18n';
import CodeMirror from '@uiw/react-codemirror';
import { rust } from '@codemirror/lang-rust';
import { go } from '@codemirror/lang-go';
import { python } from '@codemirror/lang-python';
import { cpp } from '@codemirror/lang-cpp';
import { CompileResponse, ExportResponse, LanguageTarget } from '../types';
import { Code2, Copy, Check, FileCode, Package, RefreshCw, Hammer, XCircle } from 'lucide-react';

interface CodeViewProps {
  exportData: ExportResponse | null;
  compileReport: CompileResponse | null;
  loading: boolean;
  compiling: boolean;
  onRefresh: () => void;
  onCompile: () => void;
  target?: LanguageTarget;
}

export const CodeView: React.FC<CodeViewProps> = ({
  exportData,
  compileReport,
  loading,
  compiling,
  onRefresh,
  onCompile,
  target = 'rust',
}) => {
  const { t, locale } = useI18n();
  const [copied, setCopied] = useState(false);

  const currentLang = exportData?.target || target;
  const langName = currentLang === 'go' ? 'Go' : currentLang === 'python' ? 'Python' : currentLang === 'cpp' ? 'C++' : 'Rust';

  const files = exportData?.files && exportData.files.length > 0
    ? exportData.files
    : [
        { name: 'main.rs', path: 'src/main.rs', content: exportData?.main_rs || '', language: 'rust' },
        { name: 'nodes.rs', path: 'src/nodes.rs', content: exportData?.nodes_rs || '', language: 'rust' },
        { name: 'Cargo.toml', path: 'Cargo.toml', content: exportData?.cargo_toml || '', language: 'toml' },
      ];

  const [activePath, setActivePath] = useState<string>(files[0]?.path || 'src/main.rs');

  useEffect(() => {
    if (files.length > 0 && !files.some((f) => f.path === activePath)) {
      setActivePath(files[0].path);
    }
  }, [files, activePath]);

  const activeFile = files.find((f) => f.path === activePath) || files[0];

  const getCurrentCode = () => {
    if (!exportData) return t('code.empty');
    return activeFile?.content || '';
  };

  const getLanguageExtension = (lang: string) => {
    switch (lang.toLowerCase()) {
      case 'go':
        return [go()];
      case 'python':
      case 'py':
        return [python()];
      case 'cpp':
      case 'c++':
        return [cpp()];
      case 'rust':
      case 'rs':
      default:
        return [rust()];
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
          <span className="font-semibold text-xs tracking-wide text-slate-100">
            {locale === 'pl' ? `Podgląd kodu ${langName}` : `Live ${langName} Code View`}
          </span>
          <span className="rounded bg-slate-800 px-1.5 py-0.5 text-[9px] font-mono uppercase text-slate-300 border border-slate-700">
            {currentLang}
          </span>
        </div>

        <div className="flex items-center gap-1.5">
          {(currentLang === 'rust') && (
            <button
              type="button"
              onClick={onCompile}
              disabled={compiling || !exportData}
              className="flex items-center gap-1 rounded bg-cyan-600/15 px-2 py-1 text-[10px] font-semibold text-cyan-300 transition-colors hover:bg-cyan-600/25 disabled:opacity-40"
              title={t('code.compileTitle')}
            >
              <Hammer className={`h-3 w-3 ${compiling ? 'animate-pulse' : ''}`} />
              {compiling ? t('code.checking') : (locale === 'pl' ? `Sprawdź ${langName}` : `Check ${langName}`)}
            </button>
          )}

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
              : 'border-rose-800 bg-rose-950/40 text-rose-300'
          }`}
        >
          <div className="flex items-center gap-1.5 font-semibold">
            {compileReport.success ? (
              <>
                <Check className="h-3.5 w-3.5" />
                <span>{t('code.compileSuccess')}</span>
              </>
            ) : (
              <>
                <XCircle className="h-3.5 w-3.5" />
                <span>{t('code.compileFailed')}</span>
              </>
            )}
          </div>
          <pre className="mt-1 max-h-24 overflow-auto whitespace-pre-wrap font-mono text-[9px] text-slate-300">
            {compileReport.stderr || compileReport.stdout || t('code.compileNoOutput')}
          </pre>
        </div>
      )}

      {/* Dynamic Tabs */}
      <div className="flex border-b border-slate-800 bg-slate-950/40 px-2 pt-1 gap-1 overflow-x-auto">
        {files.map((file) => {
          const isActive = file.path === (activeFile?.path || activePath);
          const isManifest = file.name === 'Cargo.toml' || file.name === 'go.mod' || file.name === 'requirements.txt' || file.name === 'CMakeLists.txt';
          return (
            <button
              key={file.path}
              onClick={() => setActivePath(file.path)}
              className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-mono rounded-t-md transition-colors whitespace-nowrap ${
                isActive
                  ? 'bg-slate-900 text-cyan-400 border-t-2 border-cyan-400'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-slate-900/50'
              }`}
            >
              {isManifest ? <Package className="w-3.5 h-3.5" /> : <FileCode className="w-3.5 h-3.5" />}
              <span>{file.path}</span>
            </button>
          );
        })}
      </div>

      {/* Code Editor */}
      <div className="flex-1 overflow-auto text-xs font-mono bg-slate-950">
        <CodeMirror
          value={getCurrentCode()}
          height="100%"
          theme="dark"
          extensions={getLanguageExtension(activeFile?.language || target)}
          editable={false}
          basicSetup={{
            lineNumbers: true,
            foldGutter: true,
            highlightActiveLine: false,
          }}
        />
      </div>
    </div>
  );
};
