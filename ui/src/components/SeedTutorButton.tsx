import React, { useCallback, useEffect, useState } from 'react';
import { AlertTriangle, ArrowRight, Info, Lightbulb, Sparkles, X, XCircle } from 'lucide-react';
import { useI18n } from '../i18n';
import { GraphSpec } from '../types';

export interface TutorHint {
  code: string;
  severity: 'info' | 'warning' | 'error';
  concept: string;
  params?: Record<string, string>;
  node_id?: string;
  suggestion?: string;
}

interface SeedTutorButtonProps {
  graph: GraphSpec;
  onFocusNode?: (nodeId: string) => void;
}

export const SeedTutorButton: React.FC<SeedTutorButtonProps> = ({ graph, onFocusNode }) => {
  const { t } = useI18n();
  const [hints, setHints] = useState<TutorHint[]>([]);
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  const fetchHints = useCallback(async () => {
    setIsLoading(true);
    try {
      const response = await fetch('/api/tutor/hints', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(graph),
      });
      if (response.ok) {
        const data = await response.json();
        setHints(data.hints || []);
      }
    } catch (error) {
      console.error('Failed to fetch SEED Tutor hints:', error);
    } finally {
      setIsLoading(false);
    }
  }, [graph]);

  useEffect(() => {
    const timer = setTimeout(fetchHints, 300);
    return () => clearTimeout(timer);
  }, [fetchHints]);

  const errorCount = hints.filter((hint) => hint.severity === 'error').length;
  const warningCount = hints.filter((hint) => hint.severity === 'warning').length;
  const infoCount = hints.filter((hint) => hint.severity === 'info').length;
  const badgeColor = errorCount > 0
    ? 'bg-rose-500 text-white animate-pulse'
    : warningCount > 0
      ? 'bg-amber-500 text-slate-950 font-bold'
      : infoCount > 0
        ? 'bg-cyan-500 text-slate-950 font-bold'
        : 'bg-slate-700 text-slate-300';

  return (
    <div className="relative">
      <button
        onClick={() => {
          setIsOpen(!isOpen);
          if (!isOpen) void fetchHints();
        }}
        className={`relative flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold border transition-all duration-200 ${
          isOpen
            ? 'bg-yellow-500/20 text-yellow-300 border-yellow-500/50 shadow-lg shadow-yellow-500/10'
            : 'bg-slate-800 hover:bg-slate-700 text-slate-200 border-slate-700'
        }`}
        title={t('tutor.tooltip')}
      >
        <Lightbulb
          className={`w-4 h-4 transition-transform ${
            hints.length > 0 ? 'text-yellow-400 drop-shadow-[0_0_6px_rgba(250,204,21,0.6)]' : 'text-slate-400'
          } ${isLoading ? 'animate-spin' : ''}`}
        />
        <span>{t('tutor.button')}</span>
        {hints.length > 0 && (
          <span className={`ml-1 text-[10px] px-1.5 py-0.2 rounded-full font-mono flex items-center justify-center min-w-[18px] ${badgeColor}`}>
            {hints.length}
          </span>
        )}
      </button>

      {isOpen && (
        <div className="absolute right-0 mt-2 w-96 max-h-[500px] bg-slate-900/98 backdrop-blur-xl border border-slate-700 rounded-xl shadow-2xl flex flex-col z-50 overflow-hidden text-slate-200">
          <div className="flex items-center justify-between px-3.5 py-2.5 border-b border-slate-800 bg-slate-950/80">
            <div className="flex items-center gap-2">
              <Sparkles className="w-4 h-4 text-yellow-400" />
              <span className="font-semibold text-xs tracking-wide text-slate-100">{t('tutor.title')}</span>
            </div>
            <button
              onClick={() => setIsOpen(false)}
              title={t('tutor.close')}
              className="p-1 rounded text-slate-400 hover:text-slate-100 hover:bg-slate-800 transition-colors"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </div>

          <div className="flex-1 overflow-y-auto p-3 space-y-2.5">
            {hints.length === 0 ? (
              <div className="p-4 text-center text-xs text-slate-400 space-y-1">
                <span className="text-xl block">🎉</span>
                <span className="font-semibold text-emerald-400 block">{t('tutor.allGood.title')}</span>
                <span className="text-[11px] text-slate-500">{t('tutor.allGood.description')}</span>
              </div>
            ) : (
              hints.map((hint, index) => {
                const isError = hint.severity === 'error';
                const isWarning = hint.severity === 'warning';
                const params = hint.params ?? {};
                return (
                  <div
                    key={`${hint.code}-${hint.node_id ?? 'graph'}-${index}`}
                    className={`p-2.5 rounded-lg border text-xs transition-all ${
                      isError
                        ? 'bg-rose-950/25 border-rose-800/60 text-rose-200'
                        : isWarning
                          ? 'bg-amber-950/25 border-amber-800/60 text-amber-200'
                          : 'bg-cyan-950/25 border-cyan-800/60 text-cyan-200'
                    }`}
                  >
                    <div className="flex items-start gap-2">
                      <div className="mt-0.5">
                        {isError && <XCircle className="w-4 h-4 text-rose-400 shrink-0" />}
                        {isWarning && <AlertTriangle className="w-4 h-4 text-amber-400 shrink-0" />}
                        {!isError && !isWarning && <Info className="w-4 h-4 text-cyan-400 shrink-0" />}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="font-medium text-slate-100 leading-snug">
                          {t(`hint.message.${hint.code}`, params)}
                        </div>
                        <div className="mt-1 text-[10px] text-slate-500">
                          {t('tutor.concept', { concept: t(`hint.concept.${hint.concept}`) })}
                        </div>
                        {hint.suggestion && (
                          <div className="mt-1.5 text-[11px] text-slate-400 bg-slate-950/70 p-1.5 rounded border border-slate-800 flex items-start gap-1.5">
                            <ArrowRight className="w-3 h-3 text-yellow-400 shrink-0 mt-0.5" />
                            <span>{t(`hint.suggestion.${hint.suggestion}`, params)}</span>
                          </div>
                        )}
                        {hint.node_id && onFocusNode && (
                          <button
                            onClick={() => onFocusNode(hint.node_id!)}
                            className="mt-1.5 text-[10px] font-mono px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition-colors"
                          >
                            {t('tutor.focusNode', { node: hint.node_id })}
                          </button>
                        )}
                      </div>
                    </div>
                  </div>
                );
              })
            )}
          </div>

          <div className="px-3 py-1.5 border-t border-slate-800/80 bg-slate-950/60 text-[10px] text-slate-500 font-mono flex items-center justify-between">
            <span>{t('tutor.footer')}</span>
            <span>{t('tutor.offline')}</span>
          </div>
        </div>
      )}
    </div>
  );
};
