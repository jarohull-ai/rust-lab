import React from 'react';
import { ArrowRight, Braces, Lightbulb, Workflow } from 'lucide-react';
import { useI18n } from '../i18n';
import { LanguageTarget } from '../types';
import packageInfo from '../../package.json';

interface WelcomeScreenProps {
  onStartLesson: () => void;
  onExplore: () => void;
  target?: LanguageTarget;
  onTargetChange?: (target: LanguageTarget) => void;
}

const languages: {
  id: LanguageTarget;
  label: string;
  badge: string;
  icon: string;
  border: string;
  bg: string;
  buttonBg: string;
  iconBg: string;
  text: string;
}[] = [
  {
    id: 'rust',
    label: 'Rust',
    badge: 'RUST LAB',
    icon: '/rust-logo.svg',
    border: 'border-orange-500/30',
    bg: 'bg-orange-500/15',
    buttonBg: 'bg-orange-600 hover:bg-orange-500 shadow-orange-600/20',
    iconBg: 'bg-orange-400 border-orange-300/50 shadow-orange-500/20',
    text: 'text-orange-300',
  },
  {
    id: 'go',
    label: 'Go',
    badge: 'GO LAB',
    icon: '/go-logo.svg',
    border: 'border-cyan-500/30',
    bg: 'bg-cyan-500/15',
    buttonBg: 'bg-cyan-600 hover:bg-cyan-500 shadow-cyan-600/20',
    iconBg: 'bg-cyan-500/20 border-cyan-400/50 shadow-cyan-500/20',
    text: 'text-cyan-300',
  },
  {
    id: 'python',
    label: 'Python',
    badge: 'PYTHON LAB',
    icon: '/python-logo.svg',
    border: 'border-amber-500/30',
    bg: 'bg-amber-500/15',
    buttonBg: 'bg-amber-600 hover:bg-amber-500 shadow-amber-600/20',
    iconBg: 'bg-amber-500/20 border-amber-400/50 shadow-amber-500/20',
    text: 'text-amber-300',
  },
  {
    id: 'cpp',
    label: 'C++',
    badge: 'C++ LAB',
    icon: '/cpp-logo.svg',
    border: 'border-blue-500/30',
    bg: 'bg-blue-500/15',
    buttonBg: 'bg-blue-600 hover:bg-blue-500 shadow-blue-600/20',
    iconBg: 'bg-blue-500/20 border-blue-400/50 shadow-blue-500/20',
    text: 'text-blue-300',
  },
];

export const WelcomeScreen: React.FC<WelcomeScreenProps> = ({
  onStartLesson,
  onExplore,
  target = 'rust',
  onTargetChange,
}) => {
  const { locale, setLocale, t } = useI18n();

  const currentLang = languages.find((l) => l.id === target) || languages[0];

  const features = [
    { key: 'graph', icon: Workflow, color: 'text-cyan-400 bg-cyan-500/10 border-cyan-500/20' },
    {
      key: target,
      icon: Braces,
      color:
        target === 'go'
          ? 'text-cyan-400 bg-cyan-500/10 border-cyan-500/20'
          : target === 'python'
          ? 'text-yellow-400 bg-yellow-500/10 border-yellow-500/20'
          : target === 'cpp'
          ? 'text-blue-400 bg-blue-500/10 border-blue-500/20'
          : 'text-orange-400 bg-orange-500/10 border-orange-500/20',
    },
    { key: 'tutor', icon: Lightbulb, color: 'text-yellow-400 bg-yellow-500/10 border-yellow-500/20' },
  ] as const;

  return (
    <div
      className="fixed inset-0 z-[100] flex items-center justify-center bg-slate-950/90 p-4 backdrop-blur-md"
      role="dialog"
      aria-modal="true"
      aria-labelledby="welcome-title"
    >
      <div className="w-full max-w-2xl overflow-hidden rounded-2xl border border-slate-700 bg-slate-900 shadow-2xl shadow-black/50">
        <div className="flex items-center justify-between border-b border-slate-800 bg-slate-950/60 px-5 py-3">
          <div className="flex items-center gap-2 text-xs font-semibold tracking-wide text-slate-200">
            <span className={`rounded-md border ${currentLang.border} ${currentLang.bg} px-2 py-1 ${currentLang.text}`}>
              {currentLang.badge}
            </span>
            <span className="font-mono text-[10px] text-slate-500">v{packageInfo.version}</span>
          </div>

          <div className="flex items-center gap-3">
            {/* Target Language Switcher inside Welcome */}
            {onTargetChange && (
              <div className="flex items-center rounded-lg border border-slate-700 bg-slate-800 p-0.5 text-[10px] font-bold">
                {languages.map((lang) => (
                  <button
                    key={lang.id}
                    type="button"
                    onClick={() => onTargetChange(lang.id)}
                    className={`rounded px-2 py-1 transition-colors ${
                      target === lang.id
                        ? `${lang.bg} ${lang.text} border ${lang.border}`
                        : 'text-slate-400 hover:text-slate-100'
                    }`}
                  >
                    {lang.label}
                  </button>
                ))}
              </div>
            )}

            {/* i18n switcher */}
            <div className="flex items-center rounded-lg border border-slate-700 bg-slate-800 p-0.5 text-[10px] font-bold">
              {(['en', 'pl'] as const).map((language) => (
                <button
                  key={language}
                  type="button"
                  onClick={() => setLocale(language)}
                  title={t(language === 'en' ? 'language.english' : 'language.polish')}
                  className={`rounded px-2 py-1 transition-colors ${
                    locale === language
                      ? 'bg-cyan-600 text-white'
                      : 'text-slate-400 hover:text-slate-100'
                  }`}
                >
                  {language.toUpperCase()}
                </button>
              ))}
            </div>
          </div>
        </div>

        <div className="p-6 sm:p-8">
          <div className="mx-auto max-w-lg text-center">
            <div className={`mx-auto flex h-16 w-16 items-center justify-center rounded-2xl border ${currentLang.iconBg} shadow-lg`}>
              <img src={currentLang.icon} alt={currentLang.label} className="h-12 w-12 object-contain" />
            </div>
            <h1 id="welcome-title" className="mt-4 text-2xl font-bold tracking-tight text-white">
              {t('welcome.title')}
            </h1>
            <p className="mt-2 text-sm leading-relaxed text-slate-400">
              {t('welcome.subtitle')}
            </p>
          </div>

          <div className="mt-6 grid gap-3 sm:grid-cols-3">
            {features.map(({ key, icon: Icon, color }) => (
              <div key={key} className="rounded-xl border border-slate-800 bg-slate-950/50 p-3.5">
                <div className={`inline-flex rounded-lg border p-2 ${color}`}>
                  <Icon className="h-4 w-4" />
                </div>
                <h2 className="mt-2.5 text-xs font-semibold text-slate-100">
                  {t(`welcome.feature.${key}.title`)}
                </h2>
                <p className="mt-1 text-[10px] leading-relaxed text-slate-500">
                  {t(`welcome.feature.${key}.description`)}
                </p>
              </div>
            ))}
          </div>

          <div className="mt-7 flex flex-col-reverse items-stretch justify-center gap-2 sm:flex-row">
            <button
              type="button"
              onClick={onExplore}
              className="rounded-lg border border-slate-700 bg-slate-800 px-4 py-2.5 text-xs font-semibold text-slate-200 transition-colors hover:bg-slate-700"
            >
              {t('welcome.explore')}
            </button>
            <button
              type="button"
              onClick={onStartLesson}
              className={`inline-flex items-center justify-center gap-2 rounded-lg ${currentLang.buttonBg} px-4 py-2.5 text-xs font-semibold text-white shadow-lg transition-colors`}
            >
              {t('welcome.startLesson')}
              <ArrowRight className="h-3.5 w-3.5" />
            </button>
          </div>

          <p className="mt-4 text-center text-[9px] text-slate-600">
            {t('welcome.localOnly')}
          </p>
        </div>
      </div>
    </div>
  );
};
