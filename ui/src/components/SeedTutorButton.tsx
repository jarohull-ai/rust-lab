import React, { useCallback, useEffect, useRef, useState } from 'react';
import {
  AlertTriangle,
  ArrowRight,
  CheckCircle2,
  Info,
  Lightbulb,
  RotateCcw,
  Sparkles,
  X,
  XCircle,
} from 'lucide-react';
import { useI18n } from '../i18n';
import { GraphSpec } from '../types';
import { apiFetch } from '../api';

export interface TutorHint {
  id: string;
  code: string;
  severity: 'info' | 'warning' | 'error';
  concept: string;
  level: 'nudge' | 'explain' | 'solution';
  attempts: number;
  params?: Record<string, string>;
  node_id?: string;
  suggestion?: string;
}

interface TutorProgress {
  attempts: Record<string, number>;
  active: Record<string, string>;
  learned: string[];
  revision?: string;
}

interface TutorResponse {
  hints: TutorHint[];
  progress: TutorProgress;
  learned_now: string[];
}

const PROGRESS_KEY = 'rust-lab.seed-tutor.progress';

const emptyProgress = (): TutorProgress => ({ attempts: {}, active: {}, learned: [] });

const loadProgress = (): TutorProgress => {
  try {
    const value = window.localStorage.getItem(PROGRESS_KEY);
    if (!value) return emptyProgress();
    const parsed = JSON.parse(value) as Partial<TutorProgress>;
    return {
      attempts: parsed.attempts ?? {},
      active: parsed.active ?? {},
      learned: Array.isArray(parsed.learned) ? parsed.learned : [],
      revision: parsed.revision,
    };
  } catch {
    return emptyProgress();
  }
};

interface SeedTutorButtonProps {
  graph: GraphSpec;
  onFocusNode?: (nodeId: string) => void;
}

export const SeedTutorButton: React.FC<SeedTutorButtonProps> = ({ graph, onFocusNode }) => {
  const { t } = useI18n();
  const [hints, setHints] = useState<TutorHint[]>([]);
  const [progress, setProgress] = useState<TutorProgress>(loadProgress);
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const progressRef = useRef(progress);
  const requestIdRef = useRef(0);

  const fetchHints = useCallback(async () => {
    const requestId = ++requestIdRef.current;
    setIsLoading(true);
    try {
      const response = await apiFetch('/api/tutor/hints', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          graph,
          progress: progressRef.current,
          revision: JSON.stringify(graph),
        }),
      });
      if (response.ok) {
        const data = (await response.json()) as TutorResponse;
        if (requestId !== requestIdRef.current) return;
        setHints(data.hints || []);
        progressRef.current = data.progress ?? emptyProgress();
        setProgress(progressRef.current);
        window.localStorage.setItem(PROGRESS_KEY, JSON.stringify(progressRef.current));
      }
    } catch (error) {
      console.error('Failed to fetch SEED Tutor hints:', error);
    } finally {
      if (requestId === requestIdRef.current) setIsLoading(false);
    }
  }, [graph]);

  const resetProgress = useCallback(() => {
    const next = emptyProgress();
    progressRef.current = next;
    setProgress(next);
    window.localStorage.removeItem(PROGRESS_KEY);
    void fetchHints();
  }, [fetchHints]);

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
            <div className="flex items-center gap-1">
              <button
                onClick={resetProgress}
                title={t('tutor.reset')}
                className="p-1 rounded text-slate-400 hover:text-slate-100 hover:bg-slate-800 transition-colors"
              >
                <RotateCcw className="w-3.5 h-3.5" />
              </button>
              <button
                onClick={() => setIsOpen(false)}
                title={t('tutor.close')}
                className="p-1 rounded text-slate-400 hover:text-slate-100 hover:bg-slate-800 transition-colors"
              >
                <X className="w-3.5 h-3.5" />
              </button>
            </div>
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
                const showConcept = hint.level !== 'nudge';
                const showSolution = hint.level === 'solution';
                return (
                  <div
                    key={`${hint.id}-${index}`}
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
                        <span className="inline-flex mb-1 text-[9px] uppercase tracking-wider font-bold text-slate-400">
                          {t(`tutor.level.${hint.level}`, { attempt: hint.attempts })}
                        </span>
                        <div className="font-medium text-slate-100 leading-snug">
                          {t(`hint.message.${hint.code}`, params)}
                        </div>
                        {showConcept && (
                          <div className="mt-1.5 text-[10px] text-slate-400 space-y-1">
                            <div>{t('tutor.concept', { concept: t(`hint.concept.${hint.concept}`) })}</div>
                            <div className="text-slate-500 leading-relaxed">
                              {t(`hint.explanation.${hint.concept}`)}
                            </div>
                          </div>
                        )}
                        {showSolution && hint.suggestion && (
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

            {progress.learned.length > 0 && (
              <div className="p-2.5 rounded-lg border border-emerald-800/60 bg-emerald-950/20">
                <div className="flex items-center gap-1.5 text-[11px] font-semibold text-emerald-300">
                  <CheckCircle2 className="w-3.5 h-3.5" />
                  {t('tutor.learned.title')}
                </div>
                <div className="mt-1.5 flex flex-wrap gap-1">
                  {progress.learned.map((concept) => (
                    <span
                      key={concept}
                      className="text-[9px] px-1.5 py-0.5 rounded-full bg-emerald-900/50 text-emerald-200 border border-emerald-800/60"
                    >
                      {t(`hint.concept.${concept}`)}
                    </span>
                  ))}
                </div>
              </div>
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
