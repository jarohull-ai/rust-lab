import React from 'react';
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
  Plus,
  Layers,
} from 'lucide-react';

interface SidebarProps {
  nodeSpecs: NodeSpec[];
  onAddNode: (kind: string) => void;
}

const CATEGORY_MAP: Record<string, string> = {
  http_get: 'Network & I/O',
  fs_read: 'Network & I/O',
  fs_write: 'Network & I/O',
  log_output: 'Network & I/O',
  json_parse: 'Transformation',
  text_clean: 'Transformation',
  condition: 'Control Flow',
  delay: 'Control Flow',
  string_to_path: 'Type Converters',
  json_to_string: 'Type Converters',
  json_to_bool: 'Type Converters',
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

export const Sidebar: React.FC<SidebarProps> = ({ nodeSpecs, onAddNode }) => {
  const onDragStart = (event: React.DragEvent, nodeType: string) => {
    event.dataTransfer.setData('application/reactflow', nodeType);
    event.dataTransfer.effectAllowed = 'move';
  };

  const categories = ['Network & I/O', 'Transformation', 'Control Flow', 'Type Converters'];

  return (
    <aside className="w-72 bg-slate-900/90 border-r border-slate-800 flex flex-col h-full overflow-hidden select-none">
      {/* Header */}
      <div className="p-4 border-b border-slate-800 flex items-center gap-2">
        <Layers className="w-4 h-4 text-orange-400" />
        <span className="font-semibold text-sm tracking-wide text-slate-100">Node Palette</span>
        <span className="ml-auto text-xs font-mono text-slate-400 bg-slate-800 px-1.5 py-0.5 rounded">
          {nodeSpecs.length} nodes
        </span>
      </div>

      {/* Palette Categories */}
      <div className="flex-1 overflow-y-auto p-3 space-y-4">
        {categories.map((cat) => {
          const nodesInCat = nodeSpecs.filter((s) => (CATEGORY_MAP[s.name] || 'Other') === cat);
          if (nodesInCat.length === 0) return null;

          return (
            <div key={cat} className="space-y-2">
              <span className="text-[11px] font-bold uppercase tracking-wider text-slate-400 pl-1">
                {cat}
              </span>
              <div className="space-y-1.5">
                {nodesInCat.map((spec) => (
                  <div
                    key={spec.name}
                    draggable
                    onDragStart={(e) => onDragStart(e, spec.name)}
                    onClick={() => onAddNode(spec.name)}
                    className="group flex items-start gap-2.5 p-2.5 rounded-lg bg-slate-950/60 border border-slate-800/80 hover:border-slate-700 hover:bg-slate-800/40 cursor-grab active:cursor-grabbing transition-all duration-150"
                  >
                    <div className="p-1.5 rounded-md bg-slate-900 border border-slate-800 mt-0.5 group-hover:scale-105 transition-transform">
                      {KIND_ICONS[spec.name] || <Terminal className="w-3.5 h-3.5 text-slate-400" />}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center justify-between">
                        <span className="font-mono text-xs font-semibold text-slate-200 group-hover:text-cyan-400 transition-colors">
                          {spec.name}
                        </span>
                        <button
                          type="button"
                          className="opacity-0 group-hover:opacity-100 p-0.5 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-100 transition-opacity"
                          title="Add to Canvas"
                        >
                          <Plus className="w-3 h-3" />
                        </button>
                      </div>
                      <p className="text-[11px] text-slate-400 line-clamp-1 mt-0.5">
                        {spec.description}
                      </p>

                      {/* Pins preview pills */}
                      <div className="flex flex-wrap gap-1 mt-1.5">
                        {spec.inputs.map((pin) => {
                          const c = PIN_COLORS[pin.pin_type as PinType] || PIN_COLORS.any;
                          return (
                            <span
                              key={`in-${pin.name}`}
                              className={`text-[9px] font-mono px-1 rounded ${c.bg} ${c.text}`}
                              title={`Input ${pin.name} (${pin.pin_type})`}
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
                              title={`Output ${pin.name} (${pin.pin_type})`}
                            >
                              {pin.name}→
                            </span>
                          );
                        })}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </div>

      <div className="p-3 border-t border-slate-800 text-[11px] text-slate-500 bg-slate-950/40">
        💡 Drag node to canvas or click to add.
      </div>
    </aside>
  );
};
