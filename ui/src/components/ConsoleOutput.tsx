import React from 'react';
import { ExecutionReport, ValidateResponse } from '../types';
import { Terminal, CheckCircle2, AlertCircle, SkipForward, XCircle, ChevronDown, ChevronUp } from 'lucide-react';

interface ConsoleOutputProps {
  logs: string[];
  executionReport: ExecutionReport | null;
  validationReport: ValidateResponse | null;
  isOpen: boolean;
  onToggle: () => void;
  onClear: () => void;
}

export const ConsoleOutput: React.FC<ConsoleOutputProps> = ({
  logs,
  executionReport,
  validationReport,
  isOpen,
  onToggle,
  onClear,
}) => {
  return (
    <div
      className={`border-t border-slate-800 bg-slate-950/95 transition-all duration-200 flex flex-col ${
        isOpen ? 'h-52' : 'h-9'
      }`}
    >
      {/* Console Bar */}
      <div className="flex items-center justify-between px-4 py-1.5 border-b border-slate-800 bg-slate-900/60 select-none">
        <button
          onClick={onToggle}
          className="flex items-center gap-2 text-xs font-mono font-semibold text-slate-300 hover:text-slate-100"
        >
          <Terminal className="w-3.5 h-3.5 text-emerald-400" />
          <span>Execution Console & Diagnostics</span>
          {validationReport && (
            <span
              className={`text-[10px] px-1.5 py-0.2 rounded font-mono ${
                validationReport.valid
                  ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
                  : 'bg-rose-500/20 text-rose-400 border border-rose-500/30'
              }`}
            >
              {validationReport.valid ? 'Valid Graph' : 'Invalid Graph'}
            </span>
          )}
          {isOpen ? <ChevronDown className="w-3.5 h-3.5" /> : <ChevronUp className="w-3.5 h-3.5" />}
        </button>

        {isOpen && (
          <button
            onClick={onClear}
            className="text-[11px] text-slate-400 hover:text-slate-200 px-2 py-0.5 rounded hover:bg-slate-800 transition-colors"
          >
            Clear Console
          </button>
        )}
      </div>

      {/* Content */}
      {isOpen && (
        <div className="flex-1 overflow-y-auto p-3 font-mono text-xs space-y-2 select-text">
          {/* Validation errors */}
          {validationReport && !validationReport.valid && (
            <div className="p-2.5 rounded bg-rose-950/30 border border-rose-800/50 text-rose-300 space-y-1">
              <div className="flex items-center gap-1.5 font-semibold text-rose-400">
                <XCircle className="w-4 h-4 text-rose-400" />
                <span>Validation Errors:</span>
              </div>
              {validationReport.errors.map((err, idx) => (
                <div key={idx} className="pl-5 text-[11px]">
                  • {err}
                </div>
              ))}
            </div>
          )}

          {/* Validation success */}
          {validationReport && validationReport.valid && (
            <div className="p-2 rounded bg-emerald-950/20 border border-emerald-800/40 text-emerald-300 flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
              <span>
                Graph is valid! Execution topological order: [{validationReport.execution_order.join(' → ')}]
              </span>
            </div>
          )}

          {/* Execution Report */}
          {executionReport && (
            <div className="p-2.5 rounded bg-slate-900 border border-slate-800 space-y-2">
              <div className="flex items-center gap-4 text-slate-200">
                <span className="flex items-center gap-1.5 text-emerald-400 font-semibold">
                  <CheckCircle2 className="w-3.5 h-3.5" /> Executed ({executionReport.executed.length}):{' '}
                  <span className="font-normal text-slate-300">{executionReport.executed.join(', ') || 'none'}</span>
                </span>
                {executionReport.skipped.length > 0 && (
                  <span className="flex items-center gap-1.5 text-slate-400 font-semibold">
                    <SkipForward className="w-3.5 h-3.5" /> Skipped ({executionReport.skipped.length}):{' '}
                    <span className="font-normal text-slate-400">{executionReport.skipped.join(', ')}</span>
                  </span>
                )}
              </div>

              {/* Node outputs dictionary */}
              <div className="space-y-1 mt-1">
                <span className="text-[11px] text-slate-400 uppercase font-semibold">Node Outputs:</span>
                <div className="grid grid-cols-2 gap-2">
                  {Object.entries(executionReport.outputs).map(([nodeId, out]) => (
                    <div key={nodeId} className="p-2 rounded bg-slate-950 border border-slate-800">
                      <span className="text-cyan-400 font-semibold">{nodeId}:</span>
                      <pre className="text-[10px] text-slate-300 mt-1 overflow-x-auto">
                        {JSON.stringify(out, null, 2)}
                      </pre>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* General logs */}
          {logs.map((log, index) => (
            <div key={index} className="text-slate-300 leading-relaxed">
              {log}
            </div>
          ))}

          {logs.length === 0 && !executionReport && !validationReport && (
            <div className="text-slate-600 italic">No execution logs yet. Click "Run Execution" to test your graph.</div>
          )}
        </div>
      )}
    </div>
  );
};
