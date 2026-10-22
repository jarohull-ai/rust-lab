import React, { memo } from 'react';
import { useI18n } from '../i18n';
import { Handle, Position } from '@xyflow/react';
import { NodeSpec, PIN_COLORS, PinType } from '../types';
import {
  Globe,
  FileText,
  FileCode,
  Terminal,
  Braces,
  Sparkles,
  GitBranch,
  Timer,
  ArrowRightLeft,
  Trash2,
  CheckCircle2,
  SkipForward,
} from 'lucide-react';

interface CustomNodeProps {
  id: string;
  data: {
    kind: string;
    spec?: NodeSpec;
    config: Record<string, any>;
    status?: 'idle' | 'executed' | 'skipped' | 'error';
    outputValue?: any;
    onConfigChange: (nodeId: string, key: string, value: any) => void;
    onDelete: (nodeId: string) => void;
  };
}

const KIND_ICONS: Record<string, React.ReactNode> = {
  http_get: <Globe className="w-4 h-4 text-cyan-400" />,
  fs_read: <FileText className="w-4 h-4 text-orange-400" />,
  fs_write: <FileCode className="w-4 h-4 text-orange-400" />,
  log_output: <Terminal className="w-4 h-4 text-emerald-400" />,
  json_parse: <Braces className="w-4 h-4 text-amber-400" />,
  text_clean: <Sparkles className="w-4 h-4 text-blue-400" />,
  condition: <GitBranch className="w-4 h-4 text-purple-400" />,
  delay: <Timer className="w-4 h-4 text-yellow-400" />,
  string_to_path: <ArrowRightLeft className="w-4 h-4 text-rose-400" />,
  json_to_string: <ArrowRightLeft className="w-4 h-4 text-rose-400" />,
  json_to_bool: <ArrowRightLeft className="w-4 h-4 text-rose-400" />,
};

export const CustomNode = memo(({ id, data }: CustomNodeProps) => {
  const { t } = useI18n();
  const { kind, spec, config, status, outputValue, onConfigChange, onDelete } = data;

  const getBorderColor = () => {
    if (status === 'executed') return 'border-emerald-500 shadow-emerald-500/20';
    if (status === 'skipped') return 'border-slate-600 opacity-60';
    if (status === 'error') return 'border-rose-500 shadow-rose-500/20';
    return 'border-slate-700/80 hover:border-slate-600';
  };

  return (
    <div
      role="group"
      aria-label={t('node.ariaLabel', { id, kind })}
      className={`min-w-[240px] max-w-[320px] rounded-xl bg-slate-900/95 backdrop-blur-md border text-slate-200 shadow-xl transition-all duration-200 ${getBorderColor()}`}
    >
      {/* Node Header */}
      <div className="flex items-center justify-between px-3.5 py-2.5 border-b border-slate-800 bg-slate-950/40 rounded-t-xl">
        <div className="flex items-center gap-2">
          <div className="p-1 rounded-md bg-slate-800/80">
            {KIND_ICONS[kind] || <Terminal className="w-4 h-4 text-slate-400" />}
          </div>
          <div>
            <div className="text-xs font-semibold tracking-wide text-slate-100 flex items-center gap-1.5">
              <span>{id}</span>
              {status === 'executed' && (
                <span title={t('node.executed')}>
                  <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
                </span>
              )}
              {status === 'skipped' && (
                <span title={t('node.skipped')}>
                  <SkipForward className="w-3.5 h-3.5 text-slate-400" />
                </span>
              )}
            </div>
            <div className="text-[10px] text-slate-400 font-mono">{kind}</div>
          </div>
        </div>

        <button
          type="button"
          onClick={() => onDelete(id)}
          className="p-1 rounded text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 transition-colors"
          title={t('node.delete')}
        >
          <Trash2 className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Inputs and Outputs Handles */}
      <div className="p-3 space-y-3">
        {/* Input Pins */}
        {spec && spec.inputs.length > 0 && (
          <div className="space-y-1.5">
            <span className="text-[10px] uppercase font-bold tracking-wider text-slate-500">{t('node.inputs')}</span>
            {spec.inputs.map((pin) => {
              const color = PIN_COLORS[pin.pin_type as PinType] || PIN_COLORS.any;
              return (
                <div key={pin.name} className="relative flex items-center h-6 pl-2">
                  <Handle
                    type="target"
                    position={Position.Left}
                    id={pin.name}
                    aria-label={t('pin.input', { name: pin.name, type: pin.pin_type })}
                    style={{ backgroundColor: color.hex, left: -18 }}
                  />
                  <span
                    className={`inline-block w-2 h-2 rounded-full mr-2 ${color.border} border`}
                    style={{ backgroundColor: color.hex }}
                  />
                  <span className="text-xs font-mono text-slate-300">{pin.name}</span>
                  <span className={`ml-auto text-[10px] font-mono px-1 rounded ${color.text} ${color.bg}`}>
                    {pin.pin_type}
                  </span>
                </div>
              );
            })}
          </div>
        )}

        {/* Node Config Fields */}
        <div className="space-y-2 pt-1 border-t border-slate-800/60">
          {kind === 'http_get' && (
            <div>
              <label className="text-[10px] uppercase text-slate-400 font-semibold">{t('node.urlConfig')}</label>
              <input
                type="text"
                value={config.url || ''}
                onChange={(e) => onConfigChange(id, 'url', e.target.value)}
                placeholder="https://httpbin.org/json"
                className="w-full text-xs font-mono px-2 py-1 rounded bg-slate-950 border border-slate-800 focus:border-cyan-500 outline-none text-slate-100"
              />
            </div>
          )}

          {kind === 'fs_read' && (
            <div>
              <label className="text-[10px] uppercase text-slate-400 font-semibold">{t('node.filePath')}</label>
              <input
                type="text"
                value={config.path || config.file_path || ''}
                onChange={(e) => {
                  onConfigChange(id, 'path', e.target.value);
                  onConfigChange(id, 'file_path', e.target.value);
                }}
                placeholder="data/input.txt"
                className="w-full text-xs font-mono px-2 py-1 rounded bg-slate-950 border border-slate-800 focus:border-orange-500 outline-none text-slate-100"
              />
            </div>
          )}

          {kind === 'fs_write' && (
            <div className="space-y-1">
              <label className="text-[10px] uppercase text-slate-400 font-semibold">{t('node.path')}</label>
              <input
                type="text"
                value={config.path || config.file_path || ''}
                onChange={(e) => {
                  onConfigChange(id, 'path', e.target.value);
                  onConfigChange(id, 'file_path', e.target.value);
                }}
                placeholder="output.txt"
                className="w-full text-xs font-mono px-2 py-1 rounded bg-slate-950 border border-slate-800 focus:border-orange-500 outline-none text-slate-100"
              />
            </div>
          )}

          {kind === 'log_output' && (
            <div>
              <label className="text-[10px] uppercase text-slate-400 font-semibold">{t('node.staticText')}</label>
              <input
                type="text"
                value={config.text || ''}
                onChange={(e) => onConfigChange(id, 'text', e.target.value)}
                placeholder={t('node.messagePlaceholder')}
                className="w-full text-xs px-2 py-1 rounded bg-slate-950 border border-slate-800 focus:border-emerald-500 outline-none text-slate-100"
              />
            </div>
          )}

          {kind === 'condition' && (
            <div className="space-y-1.5">
              <div className="flex gap-2">
                <div className="flex-1">
                  <label className="text-[10px] uppercase text-emerald-400 font-semibold">{t('node.trueNode')}</label>
                  <input
                    type="text"
                    value={config.true_branch || ''}
                    onChange={(e) => onConfigChange(id, 'true_branch', e.target.value)}
                    placeholder="e.g. node_a"
                    className="w-full text-xs font-mono px-2 py-1 rounded bg-slate-950 border border-slate-800 focus:border-purple-500 outline-none"
                  />
                </div>
                <div className="flex-1">
                  <label className="text-[10px] uppercase text-rose-400 font-semibold">{t('node.falseNode')}</label>
                  <input
                    type="text"
                    value={config.false_branch || ''}
                    onChange={(e) => onConfigChange(id, 'false_branch', e.target.value)}
                    placeholder="e.g. node_b"
                    className="w-full text-xs font-mono px-2 py-1 rounded bg-slate-950 border border-slate-800 focus:border-purple-500 outline-none"
                  />
                </div>
              </div>
            </div>
          )}

          {kind === 'json_parse' && (
            <div className="space-y-1.5">
              <div>
                <label className="text-[10px] uppercase text-slate-400 font-semibold">
                  {t('node.jsonText')}
                </label>
                <input
                  type="text"
                  value={config.json_str || ''}
                  onChange={(e) => onConfigChange(id, 'json_str', e.target.value)}
                  placeholder='{"enabled": true}'
                  className="w-full text-xs font-mono px-2 py-1 rounded bg-slate-950 border border-slate-800 focus:border-amber-500 outline-none text-slate-100"
                />
              </div>
              <div>
                <label className="text-[10px] uppercase text-slate-400 font-semibold">
                  {t('node.jsonPath')}
                </label>
                <input
                  type="text"
                  value={config.path || ''}
                  onChange={(e) => onConfigChange(id, 'path', e.target.value)}
                  placeholder="data.items.0.name"
                  className="w-full text-xs font-mono px-2 py-1 rounded bg-slate-950 border border-slate-800 focus:border-amber-500 outline-none text-slate-100"
                />
              </div>
            </div>
          )}

          {kind === 'delay' && (
            <div>
              <label className="text-[10px] uppercase text-slate-400 font-semibold">{t('node.duration')}</label>
              <input
                type="text"
                value={config.duration || '200ms'}
                onChange={(e) => onConfigChange(id, 'duration', e.target.value)}
                placeholder="e.g. 500ms, 1s"
                className="w-full text-xs font-mono px-2 py-1 rounded bg-slate-950 border border-slate-800 focus:border-yellow-500 outline-none"
              />
            </div>
          )}

          {kind === 'text_clean' && (
            <div className="space-y-1 text-[11px] text-slate-300">
              <div className="pb-1">
                <label className="text-[10px] uppercase text-slate-400 font-semibold">
                  {t('node.staticText')}
                </label>
                <input
                  type="text"
                  value={config.text || ''}
                  onChange={(e) => onConfigChange(id, 'text', e.target.value)}
                  placeholder={t('node.messagePlaceholder')}
                  className="w-full text-xs px-2 py-1 rounded bg-slate-950 border border-slate-800 focus:border-blue-500 outline-none text-slate-100"
                />
              </div>
              <div className="flex items-center justify-between">
                <span>{t('node.trim')}:</span>
                <input
                  type="checkbox"
                  checked={config.trim !== false}
                  onChange={(e) => onConfigChange(id, 'trim', e.target.checked)}
                />
              </div>
              <div className="flex items-center justify-between">
                <span>{t('node.lowercase')}:</span>
                <input
                  type="checkbox"
                  checked={!!config.to_lower}
                  onChange={(e) => onConfigChange(id, 'to_lower', e.target.checked)}
                />
              </div>
              <div className="flex items-center justify-between">
                <span>{t('node.uppercase')}:</span>
                <input
                  type="checkbox"
                  checked={!!config.to_upper}
                  onChange={(e) => onConfigChange(id, 'to_upper', e.target.checked)}
                />
              </div>
            </div>
          )}
        </div>

        {/* Output Pins */}
        {spec && spec.outputs.length > 0 && (
          <div className="space-y-1.5 pt-2 border-t border-slate-800/60">
            <span className="text-[10px] uppercase font-bold tracking-wider text-slate-500">{t('node.outputs')}</span>
            {spec.outputs.map((pin) => {
              const color = PIN_COLORS[pin.pin_type as PinType] || PIN_COLORS.any;
              return (
                <div key={pin.name} className="relative flex items-center justify-end h-6 pr-2">
                  <span className={`mr-auto text-[10px] font-mono px-1 rounded ${color.text} ${color.bg}`}>
                    {pin.pin_type}
                  </span>
                  <span className="text-xs font-mono text-slate-300 mr-2">{pin.name}</span>
                  <span
                    className={`inline-block w-2 h-2 rounded-full ${color.border} border`}
                    style={{ backgroundColor: color.hex }}
                  />
                  <Handle
                    type="source"
                    position={Position.Right}
                    id={pin.name}
                    aria-label={t('pin.output', { name: pin.name, type: pin.pin_type })}
                    style={{ backgroundColor: color.hex, right: -18 }}
                  />
                </div>
              );
            })}
          </div>
        )}

        {/* Live Output Preview if executed */}
        {outputValue && (
          <div className="pt-2 border-t border-slate-800/60">
            <span className="text-[10px] font-mono text-emerald-400 font-semibold block mb-1">{t('node.runtimeOutput')}:</span>
            <pre className="text-[10px] font-mono bg-slate-950 p-1.5 rounded border border-slate-800 text-slate-300 max-h-20 overflow-y-auto whitespace-pre-wrap">
              {JSON.stringify(outputValue, null, 2)}
            </pre>
          </div>
        )}
      </div>
    </div>
  );
});

CustomNode.displayName = 'CustomNode';
