import React from 'react';
import { AlertTriangle, Sparkles, HeartHandshake, Github, Bug, ArrowRight, ShieldAlert, Cpu } from 'lucide-react';
import { soundEffects } from './AudioEffects';

interface CommunityNoticeProps {
  lang: 'en' | 'si';
}

export const CommunityNotice: React.FC<CommunityNoticeProps> = ({ lang }) => {
  const isSi = lang === 'si';

  return (
    <section id="community-notice" className="relative py-24 px-4 sm:px-6 lg:px-8 bg-[#06080e] border-t border-amber-500/30 overflow-hidden">
      {/* Ambient background glows */}
      <div className="absolute top-0 left-1/4 w-96 h-96 bg-amber-500/5 rounded-full blur-3xl pointer-events-none" />
      <div className="absolute bottom-0 right-1/4 w-96 h-96 bg-cyan-500/5 rounded-full blur-3xl pointer-events-none" />

      <div className="max-w-7xl mx-auto relative z-10">
        {/* Prominent Header Badge */}
        <div className="text-center max-w-3xl mx-auto mb-14 space-y-4">
          <div className="inline-flex items-center gap-2 px-4 py-1.5 rounded-full border border-amber-500/40 bg-amber-500/10 backdrop-blur-md">
            <AlertTriangle className="w-4 h-4 text-amber-400 animate-pulse" />
            <span className="font-mono text-xs font-bold tracking-widest uppercase text-amber-300">
              {isSi ? '[විවෘත අධ්‍යාපනික ප්‍රයත්නය සහ ආරක්ෂණ නිවේදනය]' : '[OPEN LEARNING INITIATIVE & COMMUNITY ADVISORY]'}
            </span>
          </div>

          <h2 className="text-3xl sm:text-5xl font-extrabold font-display text-white tracking-tight">
            {isSi ? (
              <>
                ඉගෙනුම, විනිවිදභාවය සහ <span className="text-amber-400 glow-amber">[එක්ව දියුණු කරමු]</span>
              </>
            ) : (
              <>
                TRANSPARENCY, LEARNING & <span className="text-amber-400 glow-amber">[COMMUNITY CALL]</span>
              </>
            )}
          </h2>

          <p className="text-slate-300 text-sm sm:text-base font-light leading-relaxed">
            {isSi
              ? 'මෙම ව්‍යාපෘතිය තනි පුද්ගල ඉගෙනුම් අරමුණු වෙනුවෙන් AI තාක්ෂණය (Gemini & ChatGPT) පූර්ණ ලෙස භාවිතයෙන් ගොඩනැගූවක් වන අතර, මෙහි අඩුපාඩු තිබිය හැකි බැවින් වැඩිදියුණු කිරීමට කැමති ඕනෑම අයෙකුට විවෘතව ආරාධනා කරමු.'
              : 'Aegis was forged as an experimental learning journey using cutting-edge AI assistance (Gemini & ChatGPT). We believe in radical honesty: here is our background, our security caveats, and our warm invitation to collaborate.'}
          </p>
        </div>

        {/* 3 Core Highlighted Pillar Cards */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          {/* Card 1: Educational & AI Provenance */}
          <div className="p-7 rounded-2xl bg-[#090d16] border border-cyan-500/30 hover:border-cyan-400/60 transition-all flex flex-col justify-between group crosshair relative">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <span className="font-mono text-[10px] text-cyan-400 bg-cyan-500/10 px-2.5 py-1 rounded border border-cyan-500/20 font-semibold tracking-wider">
                  01 // ORIGIN & PROVENANCE
                </span>
                <div className="p-2.5 rounded-lg bg-cyan-500/10 text-cyan-400 group-hover:rotate-12 transition-transform">
                  <Sparkles className="w-5 h-5" />
                </div>
              </div>

              <h3 className="text-xl font-bold font-display text-white group-hover:text-cyan-300 transition-colors">
                {isSi ? 'අධ්‍යාපනික ප්‍රයත්නය සහ AI භාවිතය' : 'Educational Learning & AI Provenance'}
              </h3>

              <p className="text-xs sm:text-sm text-slate-300 font-light leading-relaxed">
                {isSi ? (
                  <>
                    මෙම ව්‍යාපෘතිය මා විසින් ආරම්භ කරන ලද්දේ පහළ මට්ටමේ පද්ධති ක්‍රමලේඛනය (Systems Programming), Zero-Trust ආරක්ෂණ ක්‍රමවේද සහ Post-Quantum ගුප්තකේතනය ප්‍රායෝගිකව හැදෑරීම සඳහා ය.
                    <br /><br />
                    සමස්ත පද්ධතියම <strong>Google Gemini</strong> සහ <strong>OpenAI ChatGPT</strong> වැනි AI මාදිලිවල උපකාරයෙන් සැලසුම් කර සහ සංවර්ධනය කරන ලද්දකි.
                  </>
                ) : (
                  <>
                    This project was conceived as an independent educational experiment to master systems-level Rust programming, Win32/Linux kernel boundaries, and post-quantum cryptographic primitives.
                    <br /><br />
                    The entire codebase was architected, written, and verified with extensive assistance from <strong>Google Gemini</strong> and <strong>OpenAI ChatGPT</strong>.
                  </>
                )}
              </p>
            </div>

            <div className="mt-6 pt-4 border-t border-white/5 font-mono text-[11px] text-cyan-400/80 flex items-center gap-2">
              <Cpu className="w-3.5 h-3.5" />
              <span>{isSi ? '100% AI සහයෝගී ක්‍රමලේඛනය' : 'Human-Guided AI Systems Engineering'}</span>
            </div>
          </div>

          {/* Card 2: Security Notice & Honest Apology */}
          <div className="p-7 rounded-2xl bg-[#0e0c0a] border-2 border-amber-500/50 hover:border-amber-400 transition-all flex flex-col justify-between group crosshair relative shadow-xl shadow-amber-500/5">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <span className="font-mono text-[10px] text-amber-400 bg-amber-500/10 px-2.5 py-1 rounded border border-amber-500/30 font-bold tracking-wider">
                  02 // CRITICAL CAVEAT
                </span>
                <div className="p-2.5 rounded-lg bg-amber-500/20 text-amber-400 group-hover:scale-110 transition-transform">
                  <ShieldAlert className="w-5 h-5 text-amber-400 animate-pulse" />
                </div>
              </div>

              <h3 className="text-xl font-bold font-display text-white group-hover:text-amber-300 transition-colors">
                {isSi ? '100% ආරක්ෂණ සහතිකය නොමැති බව' : 'Security Notice & Humble Apology'}
              </h3>

              <div className="text-xs sm:text-sm text-slate-300 font-light leading-relaxed space-y-2">
                <p>
                  {isSi ? (
                    <>
                      <strong className="text-amber-300">කරුණාවෙන් සලකන්න:</strong> මෙහි NIST FIPS 203/204, DPAPI, සහ ZK Range Proofs ක්‍රියාත්මක කර තිබුණ ද, මෙය <strong>තවමත් 100% ක් සම්පූර්ණ ආරක්ෂිත හෝ වෘත්තීය විගණනයකට ලක්වූවක් නොවේ</strong>.
                    </>
                  ) : (
                    <>
                      <strong className="text-amber-300">IMPORTANT NOTICE:</strong> While Aegis implements cutting-edge specifications (FIPS 203/204, DPAPI, Kernel Sandboxing, ZK Proofs), <strong>it has NOT undergone independent third-party audits or real-world banking battle-testing</strong>.
                    </>
                  )}
                </p>
                <p className="text-slate-400 text-xs">
                  {isSi ? (
                    <>
                      මෙහි තාක්ෂණික අඩුපාඩු, ගැටළු හෝ දෝෂ (bugs) තිබිය හැකි බැවින් සිදුවන අපහසුතාවයන්ට අපගේ අවංක කනගාටුව ප්‍රකාශ කරමු. ස්වාධීන ආරක්ෂණ පරීක්ෂාවකින් තොරව සැබෑ මූල්‍ය හෝ රහස්‍ය මෙහෙයුම් සඳහා භාවිත නොකරන්න.
                    </>
                  ) : (
                    <>
                      We sincerely apologize for any shortcomings, edge-case bugs, or inconveniences you may experience. Do not deploy in mission-critical sovereign environments without formal independent code review.
                    </>
                  )}
                </p>
              </div>
            </div>

            <div className="mt-6 pt-4 border-t border-amber-500/20 font-mono text-[11px] text-amber-300/90 flex items-center gap-2">
              <AlertTriangle className="w-3.5 h-3.5 text-amber-400" />
              <span>{isSi ? 'පර්යේෂණාත්මක මට්ටමේ පවතී' : 'Academic & Experimental Tier'}</span>
            </div>
          </div>

          {/* Card 3: Open Call for Contributors */}
          <div className="p-7 rounded-2xl bg-[#09110d] border border-emerald-500/30 hover:border-emerald-400/60 transition-all flex flex-col justify-between group crosshair relative">
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <span className="font-mono text-[10px] text-emerald-400 bg-emerald-500/10 px-2.5 py-1 rounded border border-emerald-500/20 font-semibold tracking-wider">
                  03 // OPEN COMMUNITY
                </span>
                <div className="p-2.5 rounded-lg bg-emerald-500/10 text-emerald-400 group-hover:rotate-12 transition-transform">
                  <HeartHandshake className="w-5 h-5" />
                </div>
              </div>

              <h3 className="text-xl font-bold font-display text-white group-hover:text-emerald-300 transition-colors">
                {isSi ? 'වැඩිදියුණු කිරීමට ඔබත් එක්වන්න!' : 'Join Us & Help Us Improve'}
              </h3>

              <p className="text-xs sm:text-sm text-slate-300 font-light leading-relaxed">
                {isSi ? (
                  <>
                    මෙහි අඩුපාඩු සහ දෝෂ හඳුනාගෙන, ඒවා නිරාකරණය කිරීමට සහ පද්ධතිය තවදුරටත් දියුණු කිරීමට කැමති ඕනෑම සංවර්ධකයෙකුට, ආරක්ෂණ පර්යේෂකයෙකුට හෝ ශිෂ්‍යයෙකුට <strong>අප සමඟ එකතු වන ලෙස ආදරයෙන් ආරාධනා කරමු!</strong>
                    <br /><br />
                    ඔබගේ Pull Requests, Bug Reports සහ වටිනා අදහස් මහත් සේ අගය කරනු ලැබේ.
                  </>
                ) : (
                  <>
                    Aegis is 100% open-source under the MIT license. We warmly welcome security researchers, systems engineers, Rust programmers, and students to review code, patch flaws, and build with us!
                    <br /><br />
                    Whether identifying an edge case, hardening memory safety, or refining the frontend, your contributions are deeply appreciated.
                  </>
                )}
              </p>
            </div>

            <div className="mt-6 pt-4 border-t border-white/5 space-y-2">
              <a
                href="https://github.com/charitha-only/aegis-desktop/blob/main/CONTRIBUTING.md"
                target="_blank"
                rel="noopener noreferrer"
                onClick={() => soundEffects.click()}
                className="w-full py-2 px-3 rounded-lg bg-emerald-500/20 hover:bg-emerald-500/30 text-emerald-300 border border-emerald-500/40 font-mono text-xs font-bold flex items-center justify-center gap-2 transition-colors"
              >
                <Github className="w-3.5 h-3.5" />
                <span>{isSi ? 'දායකත්ව මාර්ගෝපදේශය (CONTRIBUTING)' : 'VIEW CONTRIBUTING GUIDE'}</span>
                <ArrowRight className="w-3.5 h-3.5" />
              </a>

              <a
                href="https://github.com/charitha-only/aegis-desktop/issues"
                target="_blank"
                rel="noopener noreferrer"
                onClick={() => soundEffects.click()}
                className="w-full py-1.5 px-3 rounded-lg bg-white/5 hover:bg-white/10 text-slate-300 font-mono text-[11px] flex items-center justify-center gap-2 transition-colors"
              >
                <Bug className="w-3.5 h-3.5 text-amber-400" />
                <span>{isSi ? 'දෝෂයක් වාර්තා කරන්න (Report Issue)' : 'REPORT AN ISSUE OR BUG'}</span>
              </a>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};
