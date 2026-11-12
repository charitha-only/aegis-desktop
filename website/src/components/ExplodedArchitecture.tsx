import React, { useState } from 'react';
import { Layers, ShieldCheck, Cpu, Database, EyeOff, Key, Code2, Sparkles, Binary, Wifi, ShieldAlert } from 'lucide-react';
import { soundEffects } from './AudioEffects';

interface ArchitectureLayer {
  id: string;
  number: string;
  title: string;
  badge: string;
  category: string;
  description: string;
  specs: string[];
  code: string;
  icon: React.ComponentType<{ className?: string }>;
  accentColor: string;
  schematicTag: string;
}

interface ExplodedArchitectureProps {
  lang?: 'en' | 'si';
}

export const ExplodedArchitecture: React.FC<ExplodedArchitectureProps> = ({ lang = 'en' }) => {
  const [selectedLayer, setSelectedLayer] = useState<number>(0);
  const [isExploded, setIsExploded] = useState<boolean>(true);
  const isSi = lang === 'si';

  const layers: ArchitectureLayer[] = [
    {
      id: 'layer-dwm',
      number: '01',
      title: isSi ? 'DWM තිර ග්‍රහණ වැළැක්වීමේ ආවරණය' : 'PROTECTIVE DWM ANTI-CAPTURE LID',
      badge: 'WIN32 DWM / OBS BLACKOUT',
      category: isSi ? 'දෘඩාංග මට්ටම' : 'HARDWARE INTERFACE LAYER',
      schematicTag: 'WDA_EXCLUDEFROMCAPTURE // 0x00000011',
      description: isSi
        ? 'Windows Desktop Window Manager (DWM) සංයුක්ත ස්ථරය සමඟ සෘජුව සම්බන්ධ වී OBS Studio, Zoom හෝ malware මඟින් screen capture කිරීම කළු තිරයකින් වළක්වයි.'
        : 'Interfaces directly with the Windows Desktop Window Manager compositor to enforce hardware-level frame blackouts against OBS Studio, Zoom, and remote screen scrapers.',
      specs: [
        'SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)',
        'Direct DWM frame black-out against OBS & TeamViewer',
        'Windows Hello & Touch ID Biometric FIDO2 Challenge-Response',
        'Fail-closed display containment on window initialization',
      ],
      code: `// Hardware-level Win32 DWM display masking
let affinity = WDA_EXCLUDEFROMCAPTURE; // 0x00000011
let result = SetWindowDisplayAffinity(hwnd, affinity);
if result == 0 {
    return Err(SecurityError::DisplayAffinityFailed);
}`,
      icon: EyeOff,
      accentColor: 'from-amber-500 to-yellow-600',
    },
    {
      id: 'layer-rasp',
      number: '02',
      title: isSi ? 'RING-0 RASP ස්වයං-ආරක්ෂණ ඒකකය' : 'RING-0 RASP & ANTI-TAMPER DIE',
      badge: 'ANTI-DEBUG / MEMORY HOOKS',
      category: isSi ? 'මූලික ධාවන ස්ථරය' : 'KERNEL RUNTIME GUARD',
      schematicTag: 'SYS_HOOK_SCAN // FAIL-CLOSED CONTAINMENT',
      description: isSi
        ? 'IsDebuggerPresent, CheckRemoteDebuggerPresent සහ VirtualProtect මතක කොකු ක්ෂණිකව පරිලෝකනය කර ක්‍රියාවලිය tamper වුවහොත් fail-closed ලෙස අගුලු දමයි.'
        : 'Continuous kernel-level scanning for attached debuggers, hardware breakpoints, and VirtualProtect opcode modifications, enforcing instant fail-closed containment.',
      specs: [
        'Win32 IsDebuggerPresent & CheckRemoteDebuggerPresent probe',
        'VirtualProtect / NtQueryInformationProcess hook detector',
        'Hardware breakpoint (DR0-DR3) register verification',
        'Instantaneous fail-closed memory zeroization',
      ],
      code: `// Real-time RASP memory hook scan
let peb_debugger = IsDebuggerPresent() != 0;
let mut remote_debugger: BOOL = 0;
CheckRemoteDebuggerPresent(GetCurrentProcess(), &mut remote_debugger);
if peb_debugger || remote_debugger != 0 {
    self.containment_active.store(true, Ordering::SeqCst);
    return Err(SecurityError::ProcessTampered);
}`,
      icon: ShieldAlert,
      accentColor: 'from-red-500 to-amber-600',
    },
    {
      id: 'layer-kernel',
      number: '03',
      title: isSi ? 'CHACHA20-POLY1305 ගුප්තකේතන දත්ත සමුදාය' : 'CHACHA20-POLY1305 AEAD DATABASE & CORE',
      badge: 'AEAD CIPHER / ACID ENGINE',
      category: isSi ? 'දත්ත සමුදාය සහ කර්නලය' : 'CORE CRYPTOGRAPHIC ENGINE',
      schematicTag: 'CHACHA20_POLY1305 // ATOMIC_ACID_COMMIT',
      description: isSi
        ? 'සත්‍යාපිත ChaCha20-Poly1305 AEAD ගුප්තකේතනය සහ ACID ගනුදෙනු සහිත අභ්‍යන්තර දත්ත සමුදාය. ශුන්‍ය-alloc මතක ආරක්ෂණය සහ ස්ලයිඩින්-වින්ඩෝ anti-DoS සීමාකාරක.'
        : 'High-concurrency authenticated database with ChaCha20-Poly1305 AEAD, ACID snapshot transactions, constant-time comparisons, and automatic memory zeroization.',
      specs: [
        'ChaCha20-Poly1305 AEAD 256-bit encrypted table storage',
        'ACID transactions (begin, commit, rollback) with snapshot isolation',
        'Constant-time crypto comparisons (timing-attack immune)',
        'Ring-buffer O(1) sliding window anti-DoS rate limiter',
      ],
      code: `// ACID Commit to ChaCha20-Poly1305 authenticated storage
let cipher = ChaCha20Poly1305::new(&key);
let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref())?;
std::fs::write(&self.db_path, &ciphertext)?;
self.transaction_snapshot.lock().unwrap().take();`,
      icon: Database,
      accentColor: 'from-cyan-500 to-blue-600',
    },
    {
      id: 'layer-wasm',
      number: '04',
      title: isSi ? 'WASM උපදෙස් ඉන්ධන වැලිපිල්ල' : 'WASM INSTRUCTION-METERED SANDBOX',
      badge: 'WASMTIME / DETERMINISTIC FUEL',
      category: isSi ? 'ප්ලගීන පරිසරය' : 'UNTRUSTED EXTENSION ENGINE',
      schematicTag: 'METERED_FUEL_BOUND // 5M_FUEL_CAP',
      description: isSi
        ? 'තෙවන පාර්ශ්වීය ප්ලගීන සඳහා දැඩි ඉන්ධන කෝටා (fuel limits) සහ 64MB උපරිම මතක සීමාවන් සහිතව සකස් කළ ආරක්ෂිත ධාවන පරිසරය.'
        : 'Strictly bounded execution environment for third-party plugins with deterministic fuel counters, instruction limits, and fail-closed trap containment.',
      specs: [
        'Deterministic instruction fuel metering (anti-infinite loop)',
        'Memory allocation ceiling (max 64 MB heap cap)',
        'Declared capability manifests (crypto:sha256, storage:read)',
        'Host trap containment preventing application crashes',
      ],
      code: `// Instruction fuel metering & trap boundary
store.set_fuel(5_000_000)?;
let result = instance.call(&mut store, "execute", &[]);
match result {
    Err(trap) if trap.is_out_of_fuel() => Err(PluginError::QuotaExceeded),
    Ok(data) => Ok(data),
}`,
      icon: Cpu,
      accentColor: 'from-emerald-500 to-teal-600',
    },
    {
      id: 'layer-pinning',
      number: '05',
      title: isSi ? 'mTLS සහතික PINNING සහ P2P MESH' : 'mTLS SPKI PINNING & P2P NOISE MESH',
      badge: 'SPKI PINNING / NOISE PROTOCOL',
      category: isSi ? 'ජාල ආරක්ෂණය' : 'ZERO-TRUST NETWORK SENTINEL',
      schematicTag: 'SHA256_SPKI_HASH // P2P_DIFF_SYNC',
      description: isSi
        ? 'Burp Suite හෝ rogue CA මඟින් කෙරෙන MITM proxy මැදිහත්වීම් වළක්වන SPKI hash pinning සහ විමධ්‍යගත P2P Noise differential sync.'
        : 'Defeats proxy interception (Burp Suite, Charles) via cryptographic SPKI hash pinning and enables peer-to-peer differential vault sync via Noise protocol.',
      specs: [
        'Cryptographic SPKI public key hash pinning',
        'MITM proxy interception detection & immediate denial',
        'Mutual TLS (mTLS) client certificate enforcement',
        'Ephemeral Noise handshake with symmetric session derivation',
      ],
      code: `// SPKI public key hash verification against MITM
let observed_pin = format!("pin-sha256=\"{}\"", hex::encode(digest));
if !endpoint.expected_spki_pins.contains(&observed_pin) {
    return Err(SecurityError::MitmInterceptionBlocked);
}`,
      icon: Wifi,
      accentColor: 'from-purple-500 to-indigo-600',
    },
    {
      id: 'layer-fido2',
      number: '06',
      title: isSi ? 'භෞතික FIDO2 / YUBIKEY සහ FOUR-EYES' : 'FIDO2 HARDWARE KEY & FOUR-EYES MATRIX',
      badge: 'CTAP2 / BASEL SOD / PCI-DSS',
      category: isSi ? 'මූල්‍ය පාලනය' : 'PHYSICAL HARDWARE & GOVERNANCE',
      schematicTag: 'USER_PRESENCE_TOUCH // SOD_IMMUTABLE_CHAIN',
      description: isSi
        ? 'භෞතික ස්පර්ශය අනිවාර්ය කරන YubiKey / FIDO2 දෘඩාංග අත්සන් සහ තනි පුද්ගලයෙකුට අනුමත කළ නොහැකි Maker-Checker Four-Eyes සම්මුතිය.'
        : 'Enforces mandatory physical hardware presence (YubiKey CTAP2) and corporate banking Segregation of Duties (SOD) with chained SHA-256 ledger integrity.',
      specs: [
        'CTAP2 user touch presence enforcement for key release',
        'Ed25519 hardware assertion generation & verification',
        'Strict Segregation of Duties (checker_id != maker_id)',
        'Immutable chained SHA-256 audit ledger blocks',
      ],
      code: `// Dual-control maker-checker + hardware touch enforcement
if tx.maker_id.eq_ignore_ascii_case(checker_id) {
    return Err(BankingError::SelfApprovalForbidden);
}
let block_hash = compute_sha256_chain(&prev_hash, &tx);
ledger.append(AuditBlock::new(block_hash, tx));`,
      icon: Key,
      accentColor: 'from-amber-400 to-orange-600',
    },
  ];

  return (
    <section id="architecture" className="relative py-28 px-4 sm:px-6 lg:px-8 border-t border-white/10 bg-[#050608]">
      {/* Tactical Background Grid Lines */}
      <div className="absolute inset-0 pointer-events-none opacity-20 bg-[linear-gradient(to_right,#80808012_1px,transparent_1px),linear-gradient(to_bottom,#80808012_1px,transparent_1px)] bg-[size:32px_32px]" />

      <div className="relative z-10 max-w-7xl mx-auto">
        {/* Section Header (Directly matching Image 4: DESIGNED AT THE [TRANSISTOR]) */}
        <div className="flex flex-col md:flex-row md:items-end justify-between mb-16 gap-6">
          <div className="space-y-3">
            <div className="inline-flex items-center gap-2 font-mono text-xs text-amber-400 uppercase tracking-widest">
              <Layers className="w-4 h-4" />
              <span>[SCHEMATIC SPECIFICATION] // 6-LAYER ZERO-TRUST STACK</span>
            </div>
            <h2 className="text-3xl sm:text-5xl font-extrabold font-display text-white tracking-tight">
              {isSi ? 'නිර්මාණය වූයේ' : 'ENGINEERED AT THE'}{' '}
              <span className="text-transparent bg-clip-text bg-gradient-to-r from-amber-400 via-amber-200 to-amber-500 glow-amber">
                {isSi ? '[BARE METAL මට්ටමෙනි.]' : '[BARE METAL].'}
              </span>
            </h2>
            <p className="text-slate-400 max-w-2xl text-sm sm:text-base font-light">
              {isSi
                ? 'මතක සීමාවන් සහ දෘඩාංග හුදකලාව මත ගොඩනැඟූ ශුන්‍ය විශ්වාස කර්නලය. සෑම register එකක්ම තහවුරුයි, සෑම උපදෙසක්ම මනින ලදයි, කිසිවක් අනාරක්ෂිත නැත.'
                : 'A zero-trust kernel designed around memory bounds and hardware enclaves. Every register verified, every instruction metered, every hook fail-closed.'}
            </p>
          </div>

          <div className="flex items-center gap-3">
            <button
              onClick={() => {
                soundEffects.click();
                setIsExploded(!isExploded);
              }}
              className="px-4 py-2.5 rounded-lg bg-[#0d111a] border border-amber-500/30 hover:border-amber-400 text-amber-300 font-mono text-xs tracking-wider flex items-center gap-2 transition-all shadow-lg"
            >
              <Sparkles className="w-3.5 h-3.5" />
              <span>{isExploded ? (isSi ? 'ස්ථර එකතු කරන්න' : 'COLLAPSE 3D STACK') : (isSi ? '3D ස්ථර විහිදුවන්න' : 'EXPLODE 3D STACK')}</span>
            </button>
          </div>
        </div>

        {/* Isometric Visual Schematic Hero Card (Directly matching Image 4) */}
        <div className="mb-14 rounded-2xl bg-[#090c14] border border-amber-500/30 p-6 sm:p-8 relative overflow-hidden shadow-2xl crosshair">
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-center">
            {/* Exploded 3D Isometric Hardware Diagram (sample-chip.jpg + Leader Lines) */}
            <div className="lg:col-span-6 relative aspect-square sm:aspect-[4/3] rounded-xl overflow-hidden bg-black/90 border border-white/10 flex items-center justify-center p-4">
              <img
                src="./assets/sample-chip.jpg"
                alt="Exploded Isometric Micro-Architecture Chip Stack"
                className="max-h-full max-w-full object-contain filter contrast-125 brightness-105 transition-transform duration-700 hover:scale-105"
              />
              <div className="absolute top-3 left-3 font-mono text-[9px] text-amber-400 bg-black/80 px-2 py-1 rounded border border-amber-500/30">
                BLUEPRINT // DIE_STACK_0x7E
              </div>
              <div className="absolute bottom-3 right-3 font-mono text-[9px] text-slate-400 bg-black/80 px-2 py-1 rounded border border-white/10">
                [IN PRODUCTION v1.2.0]
              </div>
            </div>

            {/* Architecture Exploded Callout Annotations */}
            <div className="lg:col-span-6 space-y-4">
              <div className="font-mono text-xs text-amber-400 uppercase tracking-widest flex items-center gap-2">
                <Binary className="w-4 h-4 text-amber-400" />
                <span>MICRO-LAYER PIN MAPPINGS & ENCLAVE BUS</span>
              </div>
              <p className="font-mono text-xs text-slate-300 leading-relaxed">
                Click any layer below to inspect its internal Win32 compositor hooks, Wasm instruction fuel counters, and cryptographic hardware assertions.
              </p>

              <div className="space-y-2 font-mono text-[11px]">
                {layers.map((l, idx) => (
                  <div
                    key={l.id}
                    onClick={() => {
                      soundEffects.click();
                      setSelectedLayer(idx);
                    }}
                    className={`p-2.5 rounded-lg border cursor-pointer transition-all flex items-center justify-between ${
                      selectedLayer === idx
                        ? 'bg-amber-500/10 border-amber-400 text-amber-300 font-bold'
                        : 'bg-white/5 border-white/5 text-slate-400 hover:border-white/20'
                    }`}
                  >
                    <span>L0{idx + 1} // {l.title}</span>
                    <span className="text-[10px] text-slate-500">{l.schematicTag}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* 3D Interactive Stack & Details Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-10 items-start">
          {/* Left Column: 3D Exploded Layer Stack Cards */}
          <div className="lg:col-span-6 space-y-4 perspective-container">
            {layers.map((layer, index) => {
              const isSelected = selectedLayer === index;
              const LayerIcon = layer.icon;

              return (
                <div
                  key={layer.id}
                  onClick={() => {
                    soundEffects.click();
                    setSelectedLayer(index);
                  }}
                  onMouseEnter={() => soundEffects.hover()}
                  style={{
                    transform: isExploded
                      ? `translateY(${index * 4}px) translateZ(${isSelected ? 30 : 0}px)`
                      : 'none',
                  }}
                  className={`cursor-pointer isometric-layer rounded-xl border p-5 transition-all duration-300 relative crosshair ${
                    isSelected
                      ? 'bg-[#0f1420] border-amber-400 shadow-xl shadow-amber-500/10 border-glow-amber'
                      : 'bg-[#0a0d15]/90 border-white/10 hover:border-white/30 hover:bg-[#0c101a]'
                  }`}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex items-center gap-3">
                      <div className={`p-2.5 rounded-lg bg-gradient-to-br ${layer.accentColor} text-black font-bold shadow-md`}>
                        <LayerIcon className="w-5 h-5 text-black" />
                      </div>
                      <div>
                        <div className="flex items-center gap-2">
                          <span className="font-mono text-xs text-amber-400 font-bold">L{layer.number}</span>
                          <span className="text-white/20">•</span>
                          <span className="font-mono text-[10px] text-slate-400 tracking-wider uppercase">{layer.category}</span>
                        </div>
                        <h3 className="font-display font-bold text-base text-white mt-0.5 tracking-tight">
                          {layer.title}
                        </h3>
                      </div>
                    </div>

                    <span className="font-mono text-[11px] text-slate-400 bg-white/5 px-2.5 py-1 rounded border border-white/10">
                      {layer.badge}
                    </span>
                  </div>

                  <p className="text-xs text-slate-400 mt-3 font-light leading-relaxed">
                    {layer.description}
                  </p>
                </div>
              );
            })}
          </div>

          {/* Right Column: Layer Micro-Inspection & Code Terminal */}
          <div className="lg:col-span-6 sticky top-28">
            <div className="rounded-xl bg-[#080b12] border border-amber-500/30 overflow-hidden shadow-2xl relative crosshair">
              {/* Terminal Title Bar */}
              <div className="flex items-center justify-between px-5 py-3.5 bg-[#0d121c] border-b border-white/10">
                <div className="flex items-center gap-2">
                  <div className="flex gap-1.5">
                    <div className="w-2.5 h-2.5 rounded-full bg-red-500/70" />
                    <div className="w-2.5 h-2.5 rounded-full bg-amber-500/70" />
                    <div className="w-2.5 h-2.5 rounded-full bg-emerald-500/70" />
                  </div>
                  <span className="font-mono text-xs text-slate-300 ml-2 font-semibold">
                    INSPECT // LAYER_{layers[selectedLayer].number}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <Code2 className="w-3.5 h-3.5 text-amber-400" />
                  <span className="font-mono text-[11px] text-amber-400">RUST / IPC ENGINE</span>
                </div>
              </div>

              {/* Technical Specifications Checklist */}
              <div className="p-5 border-b border-white/10 bg-[#0a0e18]/80 font-mono text-xs space-y-2">
                <div className="text-[10px] text-slate-500 uppercase tracking-widest mb-3">
                  // VERIFIED CAPABILITY CONTRACTS
                </div>
                {layers[selectedLayer].specs.map((spec, sIdx) => (
                  <div key={sIdx} className="flex items-start gap-2.5 text-slate-300">
                    <span className="text-amber-400 font-bold select-none">&gt;</span>
                    <span className="font-light">{spec}</span>
                  </div>
                ))}
              </div>

              {/* Code Snippet Box */}
              <div className="p-5 bg-black/90 overflow-x-auto">
                <pre className="font-mono text-xs text-amber-200/90 leading-relaxed">
                  <code>{layers[selectedLayer].code}</code>
                </pre>
              </div>

              {/* Footer Audit Signature */}
              <div className="px-5 py-3 bg-[#0a0d16] border-t border-white/10 flex items-center justify-between font-mono text-[10px] text-slate-500">
                <span>SCHEMATIC: NIST SP 800-218</span>
                <span className="text-emerald-400">HARDWARE BOUND</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};
