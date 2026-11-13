import React, { useState } from 'react';
import { Download, Terminal, Check, Copy, ExternalLink, ShieldCheck, Box, Package } from 'lucide-react';
import { soundEffects } from './AudioEffects';

export const DownloadsSection: React.FC = () => {
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);

  const releaseTag = 'v1.5.0';
  const releaseBaseUrl = `https://github.com/charitha-only/aegis-desktop/releases/download/${releaseTag}`;

  const downloadAssets = [
    {
      title: 'Windows x64 Standalone Zip',
      filename: `AegisDesktopApplication-${releaseTag}-windows-x64.zip`,
      size: '2.70 MB',
      os: 'Windows 10 / 11 (x64)',
      badge: 'Authenticode Signed',
      url: `${releaseBaseUrl}/AegisDesktopApplication-${releaseTag}-windows-x64.zip`,
      icon: Box,
    },
    {
      title: 'Windows Inno Setup Installer',
      filename: `AegisDesktopApplication-Setup-${releaseTag}.exe`,
      size: '3.20 MB',
      os: 'Windows 10 / 11 (x64)',
      badge: 'Silent VBS + Start Menu',
      url: `https://github.com/charitha-only/aegis-desktop/releases/tag/${releaseTag}`,
      icon: Package,
    },
    {
      title: 'Linux Debian Package (.deb)',
      filename: 'AegisDesktopApplication.deb',
      size: '1.70 MB',
      os: 'Ubuntu 22.04+ / Debian 12+',
      badge: 'Wayland / wl-copy Driver',
      url: `https://github.com/charitha-only/aegis-desktop/releases/tag/${releaseTag}`,
      icon: Package,
    },
    {
      title: 'macOS Universal Package (.pkg)',
      filename: 'AegisDesktopApplication.pkg',
      size: '1.80 MB',
      os: 'macOS 12+ (Apple Silicon & Intel)',
      badge: 'Universal Binary',
      url: `https://github.com/charitha-only/aegis-desktop/releases/tag/${releaseTag}`,
      icon: Package,
    },
  ];

  const quickstartCommands = [
    { label: 'Clone Repository', cmd: 'git clone https://github.com/charitha-only/aegis-desktop.git' },
    { label: 'Install Dependencies', cmd: 'cd aegis-desktop && npm install && npm --prefix src-ui install' },
    { label: 'Run Local Dev (Core + UI)', cmd: 'npm run dev' },
    { label: 'Run Full Test Suite', cmd: 'npm test' },
  ];

  const handleCopy = (text: string, index: number) => {
    soundEffects.click();
    navigator.clipboard.writeText(text);
    setCopiedIndex(index);
    setTimeout(() => setCopiedIndex(null), 2500);
  };

  return (
    <section id="downloads" className="relative py-28 px-4 sm:px-6 lg:px-8 bg-[#050608] border-t border-white/10">
      <div className="max-w-7xl mx-auto">
        {/* Section Header */}
        <div className="max-w-3xl mb-16 space-y-3">
          <div className="inline-flex items-center gap-2 font-mono text-xs text-amber-400 uppercase tracking-widest">
            <Download className="w-4 h-4" />
            <span>[DISTRIBUTION PACKAGES // v1.5.0]</span>
          </div>
          <h2 className="text-3xl sm:text-5xl font-extrabold font-display text-white tracking-tight">
            DOWNLOAD & <span className="text-amber-400 glow-amber">[DEPLOY]</span>
          </h2>
          <p className="text-slate-400 text-sm sm:text-base font-light">
            Verified release binaries signed with Authenticode and DigiCert RFC 3161 timestamps. Includes full SPDX 2.3 Software Bill of Materials (SBOM) tracking 442 dependencies.
          </p>
        </div>

        {/* Download Asset Cards Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-16">
          {downloadAssets.map((asset, idx) => {
            const Icon = asset.icon;
            return (
              <div
                key={idx}
                onMouseEnter={() => soundEffects.hover()}
                className="p-5 rounded-xl bg-[#0a0d16] border border-white/10 hover:border-amber-500/40 transition-all flex flex-col justify-between group crosshair"
              >
                <div>
                  <div className="flex items-center justify-between mb-3">
                    <div className="p-2.5 rounded-lg bg-amber-500/10 text-amber-400 border border-amber-500/20 group-hover:scale-110 transition-transform">
                      <Icon className="w-5 h-5" />
                    </div>
                    <span className="font-mono text-[10px] text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded border border-emerald-500/20">
                      {asset.badge}
                    </span>
                  </div>

                  <h3 className="font-mono text-sm font-bold text-white mb-1 group-hover:text-amber-300 transition-colors">
                    {asset.title}
                  </h3>
                  <div className="font-mono text-[11px] text-slate-400 mb-1">{asset.os}</div>
                  <div className="font-mono text-[10px] text-slate-500 mb-4">{asset.size}</div>
                </div>

                <a
                  href={asset.url}
                  onClick={() => soundEffects.click()}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="w-full py-2.5 px-3 rounded-lg bg-[#141a29] hover:bg-amber-500 hover:text-black font-mono text-xs font-semibold text-amber-300 border border-amber-500/30 flex items-center justify-center gap-1.5 transition-all text-center"
                >
                  <Download className="w-3.5 h-3.5" />
                  <span>Download Binary</span>
                </a>
              </div>
            );
          })}
        </div>

        {/* Quickstart Command Center */}
        <div className="p-6 rounded-xl bg-[#090c14] border border-amber-500/20 space-y-4 crosshair">
          <div className="flex items-center justify-between pb-3 border-b border-white/10">
            <div className="flex items-center gap-2">
              <Terminal className="w-4 h-4 text-amber-400" />
              <span className="font-mono text-xs font-bold text-slate-200">DEVELOPER QUICKSTART CLI COMMANDS</span>
            </div>
            <a
              href="https://github.com/charitha-only/aegis-desktop"
              target="_blank"
              rel="noopener noreferrer"
              className="font-mono text-[11px] text-amber-400 hover:text-amber-300 flex items-center gap-1"
            >
              <span>GitHub Repository</span>
              <ExternalLink className="w-3 h-3" />
            </a>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {quickstartCommands.map((item, idx) => (
              <div
                key={idx}
                className="p-3 rounded-lg bg-[#05070d] border border-white/5 flex items-center justify-between font-mono text-xs group"
              >
                <div className="truncate pr-2">
                  <div className="text-[10px] text-slate-500 mb-0.5">{item.label}</div>
                  <div className="text-slate-300 truncate">
                    <span className="text-amber-500 mr-2">$</span>
                    {item.cmd}
                  </div>
                </div>
                <button
                  onClick={() => handleCopy(item.cmd, idx)}
                  className="p-1.5 rounded hover:bg-white/10 text-slate-400 hover:text-white transition-colors flex-shrink-0"
                  title="Copy command"
                >
                  {copiedIndex === idx ? (
                    <Check className="w-4 h-4 text-emerald-400" />
                  ) : (
                    <Copy className="w-4 h-4" />
                  )}
                </button>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
};

