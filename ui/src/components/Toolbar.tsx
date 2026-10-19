import React, { useState } from 'react';
import {
  Play,
  CheckCircle,
  FolderDown,
  Trash2,
  BookOpen,
  Cpu,
  GraduationCap,
} from 'lucide-react';
import { SeedTutorButton } from './SeedTutorButton';
import { useI18n } from '../i18n';
import { GraphSpec } from '../types';
import packageInfo from '../../package.json';

interface ToolbarProps {
  graph: GraphSpec;
  onValidate: () => void;
  onExecute: () => void;
  onExportDisk: () => void;
  onClear: () => void;
  onToggleLessons: () => void;
  onLoadExample: (exampleName: 'condition' | 'text' | 'http') => void;
  onFocusNode?: (nodeId: string) => void;
  apiConnected: boolean;
  isValidating: boolean;
  isExecuting: boolean;
  isExporting: boolean;
  lessonsOpen: boolean;
}

export const Toolbar: React.FC<ToolbarProps> = ({
  graph,
  onValidate,
  onExecute,
  onExportDisk,
  onClear,
  onToggleLessons,
  onLoadExample,
  onFocusNode,
  apiConnected,
  isValidating,
  isExecuting,
  isExporting,
  lessonsOpen,
}) => {
  const { locale, setLocale, t } = useI18n();
  const [showExamples, setShowExamples] = useState(false);

  return (
    <header className="h-14 bg-slate-900 border-b border-slate-800 flex items-center justify-between px-4 select-none z-10">
      {/* Brand */}
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2">
          <div className="p-1.5 rounded-lg bg-orange-600/20 border border-orange-500/40 text-orange-400">
            <Cpu className="w-5 h-5" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <span className="font-bold text-sm tracking-wide text-slate-100">RUST LAB</span>
              <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-orange-500/20 text-orange-400 font-semibold border border-orange-500/30">
                v{packageInfo.version}
              </span>
            </div>
            <span className="text-[10px] text-slate-400 block -mt-0.5 font-mono">
              {t('toolbar.tagline')}
            </span>
          </div>
        </div>

        {/* API Status Badge */}
        <div className="flex items-center gap-1.5 ml-4 px-2 py-1 rounded bg-slate-950 border border-slate-800 text-[11px] font-mono">
          <span
            className={`w-2 h-2 rounded-full ${
              apiConnected ? 'bg-emerald-400 shadow-emerald-400/50 shadow-sm' : 'bg-rose-500'
            }`}
          />
          <span className="text-slate-400">API:</span>
          <span className={apiConnected ? 'text-emerald-400 font-semibold' : 'text-rose-400'}>
            {apiConnected ? t('toolbar.connected') : t('toolbar.disconnected')}
          </span>
        </div>
      </div>

      {/* Action Buttons */}
      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={onToggleLessons}
          className={`flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-xs font-semibold transition-colors ${
            lessonsOpen
              ? 'border-orange-500/50 bg-orange-500/20 text-orange-300'
              : 'border-slate-700 bg-slate-800 text-slate-200 hover:bg-slate-700'
          }`}
        >
          <GraduationCap className="h-3.5 w-3.5" />
          <span>{t('toolbar.lessons')}</span>
        </button>

        {/* Load Example Dropdown */}
        <div className="relative">
          <button
            onClick={() => setShowExamples(!showExamples)}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 transition-colors"
          >
            <BookOpen className="w-3.5 h-3.5 text-yellow-400" />
            <span>{t('toolbar.loadExample')}</span>
          </button>

          {showExamples && (
            <div className="absolute left-0 mt-1.5 w-56 bg-slate-900 border border-slate-800 rounded-lg shadow-2xl p-1 z-50">
              <button
                onClick={() => {
                  onLoadExample('condition');
                  setShowExamples(false);
                }}
                className="w-full text-left px-2.5 py-1.5 rounded text-xs hover:bg-slate-800 text-slate-200 flex flex-col"
              >
                <span className="font-semibold text-purple-400">{t('toolbar.example.condition.title')}</span>
                <span className="text-[10px] text-slate-400">{t('toolbar.example.condition.description')}</span>
              </button>
              <button
                onClick={() => {
                  onLoadExample('text');
                  setShowExamples(false);
                }}
                className="w-full text-left px-2.5 py-1.5 rounded text-xs hover:bg-slate-800 text-slate-200 flex flex-col mt-0.5"
              >
                <span className="font-semibold text-blue-400">{t('toolbar.example.text.title')}</span>
                <span className="text-[10px] text-slate-400">{t('toolbar.example.text.description')}</span>
              </button>
              <button
                onClick={() => {
                  onLoadExample('http');
                  setShowExamples(false);
                }}
                className="w-full text-left px-2.5 py-1.5 rounded text-xs hover:bg-slate-800 text-slate-200 flex flex-col mt-0.5"
              >
                <span className="font-semibold text-cyan-400">{t('toolbar.example.http.title')}</span>
                <span className="text-[10px] text-slate-400">{t('toolbar.example.http.description')}</span>
              </button>
            </div>
          )}
        </div>

        <SeedTutorButton graph={graph} onFocusNode={onFocusNode} />

        {/* Validate Graph */}
        <button
          onClick={onValidate}
          disabled={isValidating}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 transition-colors disabled:opacity-50"
        >
          <CheckCircle className={`w-3.5 h-3.5 text-cyan-400 ${isValidating ? 'animate-spin' : ''}`} />
          <span>{t('toolbar.validate')}</span>
        </button>

        {/* Run Execution */}
        <button
          onClick={onExecute}
          disabled={isExecuting}
          className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-semibold bg-emerald-600 hover:bg-emerald-500 text-white shadow-lg shadow-emerald-600/20 transition-all disabled:opacity-50"
        >
          <Play className={`w-3.5 h-3.5 fill-current ${isExecuting ? 'animate-pulse' : ''}`} />
          <span>{isExecuting ? t('toolbar.running') : t('toolbar.run')}</span>
        </button>

        {/* Export Cargo */}
        <button
          onClick={onExportDisk}
          disabled={isExporting}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold bg-orange-600 hover:bg-orange-500 text-white shadow-lg shadow-orange-600/20 transition-all disabled:opacity-50"
          title={t('toolbar.exportTitle')}
        >
          <FolderDown className="w-3.5 h-3.5" />
          <span>{t('toolbar.export')}</span>
        </button>

        <div className="h-5 w-px bg-slate-800 mx-1" />

        {/* Clear */}
        <button
          onClick={onClear}
          className="p-1.5 rounded-lg text-slate-400 hover:text-rose-400 hover:bg-rose-500/10 border border-transparent hover:border-rose-500/20 transition-colors"
          title={t('toolbar.clearTitle')}
        >
          <Trash2 className="w-4 h-4" />
        </button>

        <div className="flex items-center rounded-lg border border-slate-700 bg-slate-800 p-0.5 text-[10px] font-bold">
          {(['en', 'pl'] as const).map((language) => (
            <button
              key={language}
              type="button"
              onClick={() => setLocale(language)}
              title={t(language === 'en' ? 'language.english' : 'language.polish')}
              className={`rounded px-1.5 py-1 transition-colors ${
                locale === language ? 'bg-cyan-600 text-white' : 'text-slate-400 hover:text-slate-100'
              }`}
            >
              {language.toUpperCase()}
            </button>
          ))}
        </div>
      </div>
    </header>
  );
};
