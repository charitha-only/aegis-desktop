import React, { useState } from 'react';
import { EyeOff, Video, AlertTriangle, CheckCircle2, ShieldAlert, MonitorPlay, RefreshCw } from 'lucide-react';
import { soundEffects } from './AudioEffects';

export const AntiCaptureDemo: React.FC = () => {
  const [isRecordingSimulated, setIsRecordingSimulated] = useState<boolean>(false);
  const [protectionEnabled, setProtectionEnabled] = useState<boolean>(true);

  const toggleRecordingSimulation = () => {
    soundEffects.shield();
    setIsRecordingSimulated(!isRecordingSimulated);
  };

  const toggleProtection = () => {
    soundEffects.click();
    setProtectionEnabled(!protectionEnabled);
  };

  return (
    <section id="anti-capture" className="relative py-28 px-4 sm:px-6 lg:px-8 bg-[#07090f] border-t border-white/10">
      <div className="max-w-7xl mx-auto">
        {/* Section Header */}
        <div className="max-w-3xl mb-16 space-y-3">
          <div className="inline-flex items-center gap-2 font-mono text-xs text-amber-400 uppercase tracking-widest">
            <EyeOff className="w-4 h-4" />
            <span>[HARDWARE DWM DISPLAY MASKING]</span>
          </div>
          <h2 className="text-3xl sm:text-5xl font-extrabold font-display text-white tracking-tight">
            ANTI-SCREEN CAPTURE <span className="text-amber-400 glow-amber">[SHIELD]</span>
          </h2>
          <p className="text-slate-400 text-sm sm:text-base font-light">
            Malware, Discord screen shares, OBS Studio, and remote desktop tools constantly scrape confidential corporate balances and user credentials. Aegis talks to the Desktop Window Manager (DWM) directly using Win32 <code className="text-amber-400">SetWindowDisplayAffinity</code> to blank the stream.
          </p>
        </div>

        {/* Interactive Dual-View Simulator */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-center">
          {/* Controls Panel */}
          <div className="lg:col-span-4 space-y-6">
            <div className="p-6 rounded-xl bg-[#0b0f19] border border-amber-500/30 space-y-6 crosshair">
              <h3 className="font-mono text-sm uppercase tracking-wider text-slate-200 font-bold flex items-center gap-2">
                <Video className="w-4 h-4 text-amber-400" />
                <span>OBS / Capture Simulator Controls</span>
              </h3>

              <div className="space-y-4">
                <div>
                  <button
                    onClick={toggleRecordingSimulation}
                    className={`w-full py-3.5 px-4 rounded-lg font-mono text-xs tracking-wider font-bold transition-all flex items-center justify-center gap-2 shadow-lg ${
                      isRecordingSimulated
                        ? 'bg-red-600 hover:bg-red-500 text-white shadow-red-600/30 animate-pulse'
                        : 'bg-[#151c2c] hover:bg-[#1c263c] text-amber-300 border border-amber-500/30'
                    }`}
                  >
                    <MonitorPlay className="w-4 h-4" />
                    <span>{isRecordingSimulated ? 'STOP OBS RECORDING SIMULATION' : 'SIMULATE OBS SCREEN RECORDING'}</span>
                  </button>
                  <p className="text-[11px] font-mono text-slate-400 mt-2">
                    {isRecordingSimulated ? 'OBS is capturing the screen right now!' : 'Click to simulate a screen-recorder capture hook.'}
                  </p>
                </div>

                <div className="pt-4 border-t border-white/10">
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-mono text-xs font-semibold text-slate-200">Aegis Win32 DWM Shield</div>
                      <div className="font-mono text-[11px] text-slate-400">SetWindowDisplayAffinity</div>
                    </div>
                    <button
                      onClick={toggleProtection}
                      className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                        protectionEnabled ? 'bg-amber-500' : 'bg-slate-700'
                      }`}
                    >
                      <span
                        className={`inline-block h-4 w-4 transform rounded-full bg-black transition-transform ${
                          protectionEnabled ? 'translate-x-6' : 'translate-x-1'
                        }`}
                      />
                    </button>
                  </div>
                </div>
              </div>

              {/* Status Report Box */}
              <div className="p-4 rounded-lg bg-[#05070c] border border-white/10 font-mono text-xs space-y-2">
                <div className="flex justify-between">
                  <span className="text-slate-400">DWM Affinity:</span>
                  <span className={protectionEnabled ? 'text-emerald-400' : 'text-slate-500'}>
                    {protectionEnabled ? '0x00000011 (EXCLUDE)' : '0x00000000 (NONE)'}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-400">Capture Stream:</span>
                  <span className={isRecordingSimulated ? (protectionEnabled ? 'text-amber-400 font-bold' : 'text-red-400 font-bold') : 'text-slate-400'}>
                    {isRecordingSimulated ? (protectionEnabled ? 'MASKED (BLACK FRAME)' : 'VULNERABLE (LEAKING)') : 'IDLE'}
                  </span>
                </div>
              </div>
            </div>
          </div>

          {/* Right Preview Windows */}
          <div className="lg:col-span-8 grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* 1. What the Real User Sees on their Display */}
            <div className="rounded-xl bg-[#0a0d15] border border-white/10 overflow-hidden shadow-2xl relative crosshair">
              <div className="px-4 py-2.5 bg-[#0e1320] border-b border-white/10 flex items-center justify-between font-mono text-xs text-slate-300">
                <div className="flex items-center gap-2">
                  <div className="w-2 h-2 rounded-full bg-emerald-400" />
                  <span>Physical Monitor (User View)</span>
                </div>
                <span className="text-[10px] text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded">
                  CLEAR 60 FPS
                </span>
              </div>

              {/* Fake Banking App UI */}
              <div className="p-5 space-y-4">
                <div className="flex justify-between items-center border-b border-white/5 pb-3">
                  <div>
                    <div className="font-mono text-[10px] text-slate-400 uppercase">Corporate Treasury</div>
                    <div className="text-xl font-bold font-mono text-amber-300">$2,480,500.00 USD</div>
                  </div>
                  <span className="px-2 py-1 rounded bg-amber-500/10 border border-amber-500/30 text-amber-400 font-mono text-[10px]">
                    SWIFT: AEGIS-NYC-01
                  </span>
                </div>

                <div className="space-y-2">
                  <div className="font-mono text-[11px] text-slate-400 uppercase">Recent Wire Outflow:</div>
                  <div className="p-2.5 rounded bg-white/[0.02] border border-white/5 font-mono text-xs flex justify-between">
                    <span className="text-slate-300">Acct #8841-2910</span>
                    <span className="text-red-400 font-semibold">-$50,000.00 USD</span>
                  </div>
                  <div className="p-2.5 rounded bg-white/[0.02] border border-white/5 font-mono text-xs flex justify-between">
                    <span className="text-slate-300">Acct #3319-9942</span>
                    <span className="text-emerald-400 font-semibold">+$250,000.00 USD</span>
                  </div>
                </div>

                <div className="p-3 rounded bg-amber-500/5 border border-amber-500/20 font-mono text-[11px] text-amber-300/80 flex items-center gap-2">
                  <CheckCircle2 className="w-3.5 h-3.5 text-amber-400 flex-shrink-0" />
                  <span>User sees crisp, fluid UI normally.</span>
                </div>
              </div>
            </div>

            {/* 2. What OBS / Screen Recorder Sees */}
            <div className="rounded-xl bg-[#0a0d15] border border-white/10 overflow-hidden shadow-2xl relative crosshair">
              <div className="px-4 py-2.5 bg-[#0e1320] border-b border-white/10 flex items-center justify-between font-mono text-xs text-slate-300">
                <div className="flex items-center gap-2">
                  <div className={`w-2 h-2 rounded-full ${isRecordingSimulated ? 'bg-red-500 animate-ping' : 'bg-slate-500'}`} />
                  <span>OBS Capture Stream View</span>
                </div>
                <span className={`text-[10px] px-2 py-0.5 rounded font-mono ${
                  isRecordingSimulated
                    ? (protectionEnabled ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-red-500/10 text-red-400 border border-red-500/20')
                    : 'bg-white/5 text-slate-400'
                }`}>
                  {isRecordingSimulated ? (protectionEnabled ? 'BLACKED OUT' : 'DATA EXPOSED') : 'NO STREAM'}
                </span>
              </div>

              {/* Dynamic OBS Stream Result */}
              {isRecordingSimulated && protectionEnabled ? (
                /* Hardware Blanked Black Frame */
                <div className="h-[238px] bg-black flex flex-col items-center justify-center p-6 text-center space-y-3 relative overflow-hidden scanlines">
                  <div className="p-3 rounded-full bg-white/5 border border-white/10 text-amber-400">
                    <ShieldAlert className="w-8 h-8 text-amber-400 animate-pulse" />
                  </div>
                  <div>
                    <div className="font-mono text-xs font-bold text-amber-400 tracking-wider">
                      WDA_EXCLUDEFROMCAPTURE ACTIVE
                    </div>
                    <div className="font-mono text-[11px] text-slate-500 mt-1 max-w-[220px]">
                      DWM excluded window surface from video encoding pipeline.
                    </div>
                  </div>
                </div>
              ) : isRecordingSimulated && !protectionEnabled ? (
                /* Unprotected Leak */
                <div className="p-5 space-y-4 bg-red-950/20 border-2 border-red-500/40">
                  <div className="p-2 bg-red-500/20 rounded font-mono text-xs text-red-300 flex items-center gap-2">
                    <AlertTriangle className="w-4 h-4 text-red-400 flex-shrink-0" />
                    <span>CRITICAL: Unprotected app leaking data to OBS!</span>
                  </div>
                  <div>
                    <div className="font-mono text-[10px] text-red-400">EXPOSED BALANCE</div>
                    <div className="text-xl font-bold font-mono text-red-300">$2,480,500.00 USD</div>
                  </div>
                </div>
              ) : (
                /* Idle Preview */
                <div className="h-[238px] flex flex-col items-center justify-center p-6 text-center text-slate-500 font-mono text-xs space-y-2">
                  <RefreshCw className="w-6 h-6 animate-spin text-slate-600" style={{ animationDuration: '8s' }} />
                  <div>OBS Stream Inactive</div>
                  <div className="text-[10px] text-slate-600">Toggle simulation button to test capture masking.</div>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

