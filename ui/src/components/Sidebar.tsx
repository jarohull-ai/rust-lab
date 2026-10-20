import React from 'react';
import { useI18n } from '../i18n';
import { LanguageTarget, NodeSpec, PIN_COLORS, PinType } from '../types';
import { getNodeTargetMeta } from '../targetInfo';
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
  Plus,
  Layers,
} from 'lucide-react';

interface SidebarProps {
  nodeSpecs: NodeSpec[];
  onAddNode: (kind: string) => void;
  target?: LanguageTarget;
}

const CATEGORY_MAP: Record<string, string> = {
  http_get: 'network',
  fs_read: 'network',
  fs_write: 'network',
  log_output: 'network',
  json_parse: 'transformation',
  text_clean: 'transformation',
  condition: 'control',
  delay: 'control',
  string_to_path: 'converters',
  json_to_string: 'converters',
  json_to_bool: 'converters',
};

const KIND_ICONS: Record<string, React.ReactNode> = {
  http_get: <Globe className="w-3.5 h-3.5 text-cyan-400" />,
  fs_read: <FileText className="w-3.5 h-3.5 text-orange-400" />,
  fs_write: <FileCode className="w-3.5 h-3.5 text-orange-400" />,
  log_output: <Terminal className="w-3.5 h-3.5 text-emerald-400" />,
  json_parse: <Braces className="w-3.5 h-3.5 text-amber-400" />,
  text_clean: <Sparkles className="w-3.5 h-3.5 text-blue-400" />,
  condition: <GitBranch className="w-3.5 h-3.5 text-purple-400" />,
  delay: <Timer className="w-3.5 h-3.5 text-yellow-400" />,
  string_to_path: <ArrowRightLeft className="w-3.5 h-3.5 text-rose-400" />,
  json_to_string: <ArrowRightLeft className="w-3.5 h-3.5 text-rose-400" />,
  json_to_bool: <ArrowRightLeft className="w-3.5 h-3.5 text-rose-400" />,
};

export const Sidebar: React.FC<SidebarProps> = ({ nodeSpecs, onAddNode, target = 'rust' }) => {
  const { t } = useI18n();
  const onDragStart = (event: React.DragEvent, nodeType: string) => {
    event.dataTransfer.setData('application/reactflow', nodeType);
    event.dataTransfer.effectAllowed = 'move';
  };

  const categories = ['network', 'transformation', 'control', 'converters'];

  const themeMeta = {
    rust: {
      iconColor: 'text-orange-400',
      badge: 'border-orange-500/30 bg-orange-500/10 text-orange-300',
      hoverBorder: 'hover:border-orange-500/40',
      hoverText: 'group-hover:text-orange-400',
    },
    go: {
      iconColor: 'text-cyan-400',
      badge: 'border-cyan-500/30 bg-cyan-500/10 text-cyan-300',
      hoverBorder: 'hover:border-cyan-500/40',
      hoverText: 'group-hover:text-cyan-400',
    },
    python: {
      iconColor: 'text-amber-400',
      badge: 'border-amber-500/30 bg-amber-500/10 text-amber-300',
      hoverBorder: 'hover:border-amber-500/40',
      hoverText: 'group-hover:text-amber-400',
    },
    cpp: {
      iconColor: 'text-blue-400',
      badge: 'border-blue-500/30 bg-blue-500/10 text-blue-300',
      hoverBorder: 'hover:border-blue-500/40',
      hoverText: 'group-hover:text-blue-400',
    },
  }[target] || {
    iconColor: 'text-orange-400',
    badge: 'border-orange-500/30 bg-orange-500/10 text-orange-300',
    hoverBorder: 'hover:border-orange-500/40',
    hoverText: 'group-hover:text-orange-400',
  };

  return (
    <aside className="w-72 bg-slate-900/90 border-r border-slate-800 flex flex-col h-full overflow-hidden select-none">
      {/* Header */}
      <div className="p-4 border-b border-slate-800 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Layers className={`w-4 h-4 ${themeMeta.iconColor}`} />
          <span className="font-semibold text-sm tracking-wide text-slate-100">{t('sidebar.title')}</span>
          <span className={`text-[9px] font-mono font-bold px-1.5 py-0.5 rounded uppercase border ${themeMeta.badge}`}>
            {target}
          </span>
        </div>
        <span className="text-xs font-mono text-slate-400 bg-slate-800 px-1.5 py-0.5 rounded">
          {t('sidebar.count', { count: nodeSpecs.length })}
        </span>
      </div>

      {/* Palette Categories */}
      <div className="flex-1 overflow-y-auto p-3 space-y-4">
        {categories.map((cat) => {
          const nodesInCat = nodeSpecs.filter((s) => CATEGORY_MAP[s.name] === cat);
          if (nodesInCat.length === 0) return null;

          return (
            <div key={cat} className="space-y-2">
              <span className="text-[11px] font-bold uppercase tracking-wider text-slate-400 pl-1">
                {t(`category.${cat}`)}
              </span>
              <div className="space-y-1.5">
                {nodesInCat.map((spec) => {
                  const nodeMeta = getNodeTargetMeta(spec.name, target);
                  return (
                    <button
                      key={spec.name}
                      type="button"
                      draggable
                      onDragStart={(e) => onDragStart(e, spec.name)}
                      onClick={() => onAddNode(spec.name)}
                      aria-label={t('sidebar.addNode', { name: spec.name })}
                      className={`group flex w-full items-start gap-2.5 p-2.5 text-left rounded-lg bg-slate-950/60 border border-slate-800/80 ${themeMeta.hoverBorder} hover:bg-slate-800/40 cursor-grab active:cursor-grabbing transition-all duration-150`}
                    >
                      <div className="p-1.5 rounded-md bg-slate-900 border border-slate-800 mt-0.5 group-hover:scale-105 transition-transform">
                        {KIND_ICONS[spec.name] || <Terminal className="w-3.5 h-3.5 text-slate-400" />}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center justify-between gap-1">
                          <span className={`font-mono text-xs font-semibold text-slate-200 ${themeMeta.hoverText} transition-colors truncate`}>
                            {spec.name}
                          </span>
                          <span className={`text-[8.5px] font-mono px-1 py-0.2 rounded border shrink-0 ${themeMeta.badge}`} title={nodeMeta.signature}>
                            {nodeMeta.badge}
                          </span>
                          <span
                            className="opacity-0 group-hover:opacity-100 p-0.5 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-100 transition-opacity ml-auto"
                            title={t('sidebar.add')}
                          >
                            <Plus className="w-3 h-3" />
                          </span>
                        </div>
                        <p className="text-[11px] text-slate-400 line-clamp-1 mt-0.5">
                          {t(`node.${spec.name}.description`)}
                        </p>

                        {/* Pins preview pills */}
                        <div className="flex flex-wrap gap-1 mt-1.5">
                          {spec.inputs.map((pin) => {
                            const c = PIN_COLORS[pin.pin_type as PinType] || PIN_COLORS.any;
                            return (
                              <span
                                key={`in-${pin.name}`}
                                className={`text-[9px] font-mono px-1 rounded ${c.bg} ${c.text}`}
                                title={t('pin.input', { name: pin.name, type: pin.pin_type })}
                              >
                                →{pin.name}
                              </span>
                            );
                          })}
                          {spec.outputs.map((pin) => {
                            const c = PIN_COLORS[pin.pin_type as PinType] || PIN_COLORS.any;
                            return (
                              <span
                                key={`out-${pin.name}`}
                                className={`text-[9px] font-mono px-1 rounded ${c.bg} ${c.text}`}
                                title={t('pin.output', { name: pin.name, type: pin.pin_type })}
                              >
                                {pin.name}→
                              </span>
                            );
                          })}
                        </div>
                      </div>
                    </button>
                  );
                })}
              </div>
            </div>
          );
        })}
      </div>

      <div className="p-3 border-t border-slate-800 text-[11px] text-slate-500 bg-slate-950/40">
        💡 {t('sidebar.drag')}
      </div>
    </aside>
  );
};
