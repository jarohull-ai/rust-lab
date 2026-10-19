import React, { useState, useEffect, useCallback } from 'react';
import { Lightbulb, AlertTriangle, XCircle, Info, X, Sparkles, ArrowRight } from 'lucide-react';
import { GraphSpec } from '../types';

export interface CopilotHint {
  severity: 'info' | 'warning' | 'error';
  message: string;
  node_id?: string;
  suggestion?: string;
}

interface CopilotButtonProps {
  graph: GraphSpec;
  onFocusNode?: (nodeId: string) => void;
}

export const CopilotButton: React.FC<CopilotButtonProps> = ({ graph, onFocusNode }) => {
  const [hints, setHints] = useState<CopilotHint[]>([]);
  const [isOpen, setIsOpen] = useState<boolean>(false);
  const [isLoading, setIsLoading] = useState<boolean>(false);

  const fetchHints = useCallback(async () => {
    setIsLoading(true);
    try {
      const res = await fetch('/api/copilot/hints', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(graph),
      });
      if (res.ok) {
        const data = await res.json();
        setHints(data.hints || []);
      }
    } catch (e) {
      console.error('Failed to fetch Copilot hints:', e);
    } finally {
      setIsLoading(false);
    }
  }, [graph]);

  // Reactive updates on graph change
  useEffect(() => {
    const timer = setTimeout(() => {
      fetchHints();
    }, 300);
    return () => clearTimeout(timer);
  }, [fetchHints]);

  const errorCount = hints.filter((h) => h.severity === 'error').length;
  const warningCount = hints.filter((h) => h.severity === 'warning').length;
  const infoCount = hints.filter((h) => h.severity === 'info').length;

  const getBadgeColor = () => {
    if (errorCount > 0) return 'bg-rose-500 text-white animate-pulse';
    if (warningCount > 0) return 'bg-amber-500 text-slate-950 font-bold';
    if (infoCount > 0) return 'bg-cyan-500 text-slate-950 font-bold';
    return 'bg-slate-700 text-slate-300';
  };

  return (
    <div className="relative">
      {/* Lightbulb Trigger Button */}
      <button
        onClick={() => {
          setIsOpen(!isOpen);
          if (!isOpen) fetchHints();
        }}
        className={`relative flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold border transition-all duration-200 ${
          isOpen
            ? 'bg-yellow-500/20 text-yellow-300 border-yellow-500/50 shadow-lg shadow-yellow-500/10'
            : 'bg-slate-800 hover:bg-slate-700 text-slate-200 border-slate-700'
        }`}
        title="Copilot AI (SEED Contextual Hints)"
      >
        <Lightbulb
          className={`w-4 h-4 transition-transform ${
            hints.length > 0 ? 'text-yellow-400 drop-shadow-[0_0_6px_rgba(250,204,21,0.6)]' : 'text-slate-400'
          } ${isLoading ? 'animate-spin' : ''}`}
        />
        <span>Copilot</span>

        {hints.length > 0 && (
          <span
            className={`ml-1 text-[10px] px-1.5 py-0.2 rounded-full font-mono flex items-center justify-center min-w-[18px] ${getBadgeColor()}`}
          >
            {hints.length}
          </span>
        )}
      </button>

      {/* Popover Window */}
      {isOpen && (
        <div className="absolute right-0 mt-2 w-96 max-h-[500px] bg-slate-900/98 backdrop-blur-xl border border-slate-700 rounded-xl shadow-2xl flex flex-col z-50 overflow-hidden text-slate-200">
          {/* Header */}
          <div className="flex items-center justify-between px-3.5 py-2.5 border-b border-slate-800 bg-slate-950/80">
            <div className="flex items-center gap-2">
              <Sparkles className="w-4 h-4 text-yellow-400" />
              <span className="font-semibold text-xs tracking-wide text-slate-100">
                Copilot AI (SEED Engine)
              </span>
            </div>
            <button
              onClick={() => setIsOpen(false)}
              className="p-1 rounded text-slate-400 hover:text-slate-100 hover:bg-slate-800 transition-colors"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </div>

          {/* Hints List */}
          <div className="flex-1 overflow-y-auto p-3 space-y-2.5">
            {hints.length === 0 ? (
              <div className="p-4 text-center text-xs text-slate-400 space-y-1">
                <span className="text-xl block">🎉</span>
                <span className="font-semibold text-emerald-400 block">Graf jest bezbłędny!</span>
                <span className="text-[11px] text-slate-500">
                  Brak brakujących wejść, cykli ani niezgodności typów. Możesz uruchomić graf lub wyeksportować kod.
                </span>
              </div>
            ) : (
              hints.map((hint, idx) => {
                const isError = hint.severity === 'error';
                const isWarning = hint.severity === 'warning';

                return (
                  <div
                    key={idx}
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
                          {hint.message}
                        </div>

                        {hint.suggestion && (
                          <div className="mt-1.5 text-[11px] text-slate-400 bg-slate-950/70 p-1.5 rounded border border-slate-800 flex items-start gap-1.5">
                            <ArrowRight className="w-3 h-3 text-yellow-400 shrink-0 mt-0.5" />
                            <span>{hint.suggestion}</span>
                          </div>
                        )}

                        {hint.node_id && onFocusNode && (
                          <button
                            onClick={() => onFocusNode(hint.node_id!)}
                            className="mt-1.5 text-[10px] font-mono px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition-colors"
                          >
                            Pokaż węzeł: {hint.node_id}
                          </button>
                        )}
                      </div>
                    </div>
                  </div>
                );
              })
            )}
          </div>

          {/* Footer */}
          <div className="px-3 py-1.5 border-t border-slate-800/80 bg-slate-950/60 text-[10px] text-slate-500 font-mono flex items-center justify-between">
            <span>⚡ Reaktywne reguły w czasie rzeczywistym</span>
            <span>Zero LLM cost</span>
          </div>
        </div>
      )}
    </div>
  );
};
