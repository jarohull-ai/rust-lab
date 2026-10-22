import React, { useEffect, useMemo, useState } from 'react';
import {
  ArrowRight,
  Check,
  Circle,
  GraduationCap,
  Play,
  RotateCcw,
  Trophy,
  X,
} from 'lucide-react';
import { useI18n } from '../i18n';
import { evaluateLesson, LESSON_IDS, LessonId, nextLesson } from '../lessons';
import { ExecutionReport, GraphSpec } from '../types';

interface GuidedLessonsProps {
  activeLesson: LessonId | null;
  graph: GraphSpec;
  executionReport: ExecutionReport | null;
  onStart: (lesson: LessonId) => void;
  onReset: () => void;
  onExit: () => void;
  onClose: () => void;
}

const COMPLETED_KEY = 'rust-lab.lessons.completed';

function loadCompleted(): LessonId[] {
  try {
    const parsed = JSON.parse(window.localStorage.getItem(COMPLETED_KEY) ?? '[]');
    return Array.isArray(parsed)
      ? parsed.filter((value): value is LessonId => LESSON_IDS.includes(value as LessonId))
      : [];
  } catch {
    return [];
  }
}

export const GuidedLessons: React.FC<GuidedLessonsProps> = ({
  activeLesson,
  graph,
  executionReport,
  onStart,
  onReset,
  onExit,
  onClose,
}) => {
  const { t } = useI18n();
  const [completed, setCompleted] = useState<LessonId[]>(loadCompleted);
  const checks = useMemo(
    () => (activeLesson ? evaluateLesson(activeLesson, graph, executionReport) : []),
    [activeLesson, executionReport, graph]
  );
  const solved = checks.length > 0 && checks.every(Boolean);

  useEffect(() => {
    if (!activeLesson || !solved || completed.includes(activeLesson)) return;
    const next = [...completed, activeLesson];
    setCompleted(next);
    window.localStorage.setItem(COMPLETED_KEY, JSON.stringify(next));
  }, [activeLesson, completed, solved]);

  const followingLesson = activeLesson ? nextLesson(activeLesson) : null;
  const passedChecks = checks.filter(Boolean).length;
  const progress = checks.length === 0 ? 0 : Math.round((passedChecks / checks.length) * 100);

  return (
    <section className="absolute left-4 top-4 z-30 w-80 max-h-[calc(100%-2rem)] overflow-hidden rounded-xl border border-slate-700 bg-slate-900/98 text-slate-200 shadow-2xl backdrop-blur-xl">
      <div className="flex items-center justify-between border-b border-slate-800 bg-slate-950/80 px-3.5 py-2.5">
        <div className="flex items-center gap-2">
          <GraduationCap className="h-4 w-4 text-orange-400" />
          <span className="text-xs font-semibold tracking-wide">{t('lessons.title')}</span>
          <span className="rounded-full bg-slate-800 px-1.5 py-0.5 text-[9px] font-mono text-slate-400">
            {completed.length}/{LESSON_IDS.length}
          </span>
        </div>
        <button
          type="button"
          onClick={onClose}
          title={t('lessons.close')}
          className="rounded p-1 text-slate-400 transition-colors hover:bg-slate-800 hover:text-slate-100"
        >
          <X className="h-3.5 w-3.5" />
        </button>
      </div>

      <div className="max-h-[calc(100vh-8rem)] overflow-y-auto p-3">
        {!activeLesson ? (
          <div className="space-y-2.5">
            <p className="text-[11px] leading-relaxed text-slate-400">
              {t('lessons.intro')}
            </p>
            {LESSON_IDS.map((lesson, index) => {
              const isComplete = completed.includes(lesson);
              return (
                <button
                  key={lesson}
                  type="button"
                  onClick={() => onStart(lesson)}
                  className="group w-full rounded-lg border border-slate-800 bg-slate-950/60 p-2.5 text-left transition-colors hover:border-orange-500/50 hover:bg-slate-800/60"
                >
                  <div className="flex items-start gap-2.5">
                    <span className={`flex h-6 w-6 shrink-0 items-center justify-center rounded-full text-[10px] font-bold ${
                      isComplete
                        ? 'bg-emerald-500/20 text-emerald-300'
                        : 'bg-orange-500/15 text-orange-300'
                    }`}>
                      {isComplete ? <Check className="h-3.5 w-3.5" /> : index + 1}
                    </span>
                    <span className="min-w-0 flex-1">
                      <span className="block text-xs font-semibold text-slate-100">
                        {t(`lesson.${lesson}.title`)}
                      </span>
                      <span className="mt-0.5 block text-[10px] leading-relaxed text-slate-400">
                        {t(`lesson.${lesson}.summary`)}
                      </span>
                    </span>
                    <ArrowRight className="mt-1 h-3.5 w-3.5 text-slate-600 transition-colors group-hover:text-orange-400" />
                  </div>
                </button>
              );
            })}
          </div>
        ) : (
          <div className="space-y-3">
            <div>
              <div className="flex items-center justify-between gap-2">
                <span className="text-[10px] font-bold uppercase tracking-wider text-orange-400">
                  {t('lessons.active')}
                </span>
                <span className="text-[10px] font-mono text-slate-500">
                  {passedChecks}/{checks.length}
                </span>
              </div>
              <h2 className="mt-1 text-sm font-semibold text-slate-100">
                {t(`lesson.${activeLesson}.title`)}
              </h2>
              <p className="mt-1 text-[11px] leading-relaxed text-slate-400">
                {t(`lesson.${activeLesson}.goal`)}
              </p>
              <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-slate-800">
                <div
                  className={`h-full rounded-full transition-all duration-300 ${
                    solved ? 'bg-emerald-500' : 'bg-orange-500'
                  }`}
                  style={{ width: `${progress}%` }}
                />
              </div>
            </div>

            <div className="space-y-1.5">
              {checks.map((passed, index) => (
                <div
                  key={`${activeLesson}-${index}`}
                  className={`flex items-start gap-2 rounded-md border px-2 py-1.5 text-[10px] leading-relaxed ${
                    passed
                      ? 'border-emerald-900/70 bg-emerald-950/25 text-emerald-200'
                      : 'border-slate-800 bg-slate-950/50 text-slate-400'
                  }`}
                >
                  {passed ? (
                    <Check className="mt-0.5 h-3 w-3 shrink-0 text-emerald-400" />
                  ) : (
                    <Circle className="mt-0.5 h-3 w-3 shrink-0 text-slate-600" />
                  )}
                  <span>{t(`lesson.${activeLesson}.criterion.${index}`)}</span>
                </div>
              ))}
            </div>

            {solved ? (
              <div className="rounded-lg border border-emerald-700/60 bg-emerald-950/30 p-3 text-center">
                <Trophy className="mx-auto h-6 w-6 text-yellow-400" />
                <div className="mt-1 text-xs font-semibold text-emerald-300">
                  {t('lessons.completed')}
                </div>
                <div className="mt-0.5 text-[10px] text-slate-400">
                  {t('lessons.completedDescription')}
                </div>
                {followingLesson ? (
                  <button
                    type="button"
                    onClick={() => onStart(followingLesson)}
                    className="mt-2 inline-flex items-center gap-1.5 rounded-md bg-emerald-600 px-3 py-1.5 text-[10px] font-semibold text-white transition-colors hover:bg-emerald-500"
                  >
                    {t('lessons.next')}
                    <ArrowRight className="h-3 w-3" />
                  </button>
                ) : (
                  <button
                    type="button"
                    onClick={onExit}
                    className="mt-2 inline-flex items-center gap-1.5 rounded-md bg-yellow-500 px-3 py-1.5 text-[10px] font-semibold text-slate-950 transition-colors hover:bg-yellow-400"
                  >
                    {t('lessons.finish')}
                    <Trophy className="h-3 w-3" />
                  </button>
                )}
              </div>
            ) : (
              <div className="rounded-lg border border-cyan-900/60 bg-cyan-950/20 p-2 text-[10px] leading-relaxed text-cyan-200/80">
                {t(`lesson.${activeLesson}.hint`)}
              </div>
            )}

            <div className="flex items-center justify-between border-t border-slate-800 pt-2">
              <button
                type="button"
                onClick={onExit}
                className="text-[10px] text-slate-500 transition-colors hover:text-slate-200"
              >
                {t('lessons.allLessons')}
              </button>
              <button
                type="button"
                onClick={onReset}
                className="inline-flex items-center gap-1 text-[10px] text-slate-400 transition-colors hover:text-orange-300"
              >
                <RotateCcw className="h-3 w-3" />
                {t('lessons.reset')}
              </button>
            </div>

            {!solved && (
              <div className="flex items-center gap-1.5 text-[9px] text-slate-600">
                <Play className="h-3 w-3" />
                {t('lessons.runReminder')}
              </div>
            )}
          </div>
        )}
      </div>
    </section>
  );
};
