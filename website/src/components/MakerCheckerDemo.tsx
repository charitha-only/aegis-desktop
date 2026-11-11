import React, { useState } from 'react';
import { Lock, Fingerprint, CheckCircle2, XCircle, ArrowRight, ShieldCheck, Link2 } from 'lucide-react';
import { soundEffects } from './AudioEffects';

interface LedgerBlock {
  index: number;
  prevHash: string;
  blockHash: string;
  txId: string;
  maker: string;
  checker: string;
  amount: string;
  status: 'APPROVED' | 'REJECTED';
  stepUpToken: string;
}

export const MakerCheckerDemo: React.FC = () => {
  const [makerId, setMakerId] = useState<string>('alice.treasury');
  const [checkerId, setCheckerId] = useState<string>('bob.auditor');
  const [amount, setAmount] = useState<number>(25000);
  const [hasBiometricToken, setHasBiometricToken] = useState<boolean>(false);
  const [statusMessage, setStatusMessage] = useState<{ text: string; isError: boolean } | null>(null);

  const [ledger, setLedger] = useState<LedgerBlock[]>([
    {
      index: 0,
      prevHash: '0000000000000000000000000000000000000000000000000000000000000000',
      blockHash: '7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069',
      txId: 'TX-GENESIS',
      maker: 'SYSTEM',
      checker: 'SYSTEM',
      amount: '$0.00',
      status: 'APPROVED',
      stepUpToken: 'ROOT',
    },
    {
      index: 1,
      prevHash: '7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069',
      blockHash: 'a4b89c3112d001e859cf9a918a25c11030e4bb3310065b9e0f491c1208945cf2',
      txId: 'TX-7491-01',
      maker: 'clara.ops',
      checker: 'david.compliance',
      amount: '$150,000.00',
      status: 'APPROVED',
      stepUpToken: 'FIDO2-BIO-VERIFIED',
    },
  ]);

  const handleSimulateBiometric = () => {
    soundEffects.success();
    setHasBiometricToken(true);
    setStatusMessage({
      text: 'Windows Hello / Touch ID Biometric Presence Confirmed: Ephemeral StepUpAuthToken Issued (60s TTL)',
      isError: false,
    });
  };

  const handleApprove = () => {
    // 1. Segregation of Duties Check
    if (makerId.trim().toLowerCase() === checkerId.trim().toLowerCase()) {
      soundEffects.shield();
      setStatusMessage({
        text: 'SECURITY VIOLATION: Segregation of Duties failed! Maker cannot self-approve (checker_id == maker_id).',
        isError: true,
      });
      return;
    }

    // 2. High Value Biometric Requirement Check
    if (amount >= 10000 && !hasBiometricToken) {
      soundEffects.shield();
      setStatusMessage({
        text: 'STEP-UP REQUIRED: Transaction >= $10,000.00 USD mandates biometric presence token before approval.',
        isError: true,
      });
      return;
    }

    soundEffects.success();
    const prevBlock = ledger[ledger.length - 1];
    const newIndex = prevBlock.index + 1;
    const randomHex = Array.from({ length: 64 }, () => Math.floor(Math.random() * 16).toString(16)).join('');

    const newBlock: LedgerBlock = {
      index: newIndex,
      prevHash: prevBlock.blockHash,
      blockHash: randomHex,
      txId: `TX-89${newIndex}2-${Math.floor(Math.random() * 90 + 10)}`,
      maker: makerId,
      checker: checkerId,
      amount: `$${amount.toLocaleString()}.00`,
      status: 'APPROVED',
      stepUpToken: hasBiometricToken ? 'FIDO2-CONSUMED' : 'STANDARD',
    };

    setLedger([...ledger, newBlock]);
    setHasBiometricToken(false);
    setStatusMessage({
      text: `TRANSACTION COMMITTED: Block #${newIndex} cryptographically linked to SHA-256 chain. Token consumed.`,
      isError: false,
    });
  };

  return (
    <section id="maker-checker" className="relative py-28 px-4 sm:px-6 lg:px-8 bg-[#050608] border-t border-white/10">
      <div className="max-w-7xl mx-auto">
        {/* Section Header */}
        <div className="max-w-3xl mb-16 space-y-3">
          <div className="inline-flex items-center gap-2 font-mono text-xs text-amber-400 uppercase tracking-widest">
            <Lock className="w-4 h-4" />
            <span>[PCI-DSS REQ 10 // BASEL GOVERNANCE]</span>
          </div>
          <h2 className="text-3xl sm:text-5xl font-extrabold font-display text-white tracking-tight">
            MAKER-CHECKER <span className="text-amber-400 glow-amber">[FOUR-EYES]</span> ENGINE
          </h2>
          <p className="text-slate-400 text-sm sm:text-base font-light">
            In corporate treasury and commercial banking, single-actor execution is forbidden. Aegis enforces strict dual-authorization: every financial transaction must be drafted by an authenticated Maker and independently approved by a separate Checker with biometric step-up guarantees.
          </p>
        </div>

        {/* Dual Column Simulator & Ledger View */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-10">
          {/* Left Column: Interactive Proposal Form */}
          <div className="lg:col-span-5 space-y-6">
            <div className="p-6 rounded-xl bg-[#0a0d16] border border-amber-500/30 space-y-5 crosshair">
              <div className="flex items-center justify-between pb-3 border-b border-white/10">
                <span className="font-mono text-xs font-bold text-slate-200">INTERACTIVE DUAL-CONTROL WORKFLOW</span>
                <span className="font-mono text-[10px] text-amber-400">ED25519 THRESHOLD</span>
              </div>

              {/* Form Fields */}
              <div className="space-y-4">
                <div>
                  <label className="block font-mono text-xs text-slate-400 mb-1">
                    Maker Identity (Drafts Proposal):
                  </label>
                  <input
                    type="text"
                    value={makerId}
                    onChange={(e) => setMakerId(e.target.value)}
                    className="w-full bg-[#05070d] border border-white/10 rounded px-3 py-2 font-mono text-xs text-amber-300 focus:outline-none focus:border-amber-400"
                  />
                </div>

                <div>
                  <label className="block font-mono text-xs text-slate-400 mb-1">
                    Checker Identity (Reviews & Approves):
                  </label>
                  <input
                    type="text"
                    value={checkerId}
                    onChange={(e) => setCheckerId(e.target.value)}
                    className="w-full bg-[#05070d] border border-white/10 rounded px-3 py-2 font-mono text-xs text-cyan-300 focus:outline-none focus:border-cyan-400"
                  />
                  <span className="text-[10px] font-mono text-slate-500">
                    Try setting Maker & Checker to the same name to test Segregation of Duties denial!
                  </span>
                </div>

                <div>
                  <label className="block font-mono text-xs text-slate-400 mb-1">
                    Transaction Amount (USD):
                  </label>
                  <input
                    type="number"
                    value={amount}
                    onChange={(e) => setAmount(Number(e.target.value))}
                    step="5000"
                    className="w-full bg-[#05070d] border border-white/10 rounded px-3 py-2 font-mono text-xs text-white focus:outline-none focus:border-amber-400"
                  />
                  {amount >= 10000 && (
                    <div className="mt-1 font-mono text-[11px] text-amber-400 flex items-center gap-1.5">
                      <span>⚠️ High-value policy active (&gt;= $10,000 USD): Biometrics required.</span>
                    </div>
                  )}
                </div>

                {/* Biometric Button */}
                {amount >= 10000 && (
                  <button
                    onClick={handleSimulateBiometric}
                    className={`w-full py-2.5 px-4 rounded font-mono text-xs flex items-center justify-center gap-2 transition-all ${
                      hasBiometricToken
                        ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40'
                        : 'bg-amber-500/10 hover:bg-amber-500/20 text-amber-300 border border-amber-500/30'
                    }`}
                  >
                    <Fingerprint className="w-4 h-4 text-amber-400" />
                    <span>{hasBiometricToken ? '✓ BIOMETRIC TOKEN VALID (60s)' : 'TRIGGER WINDOWS HELLO BIOMETRICS'}</span>
                  </button>
                )}

                {/* Approve Button */}
                <button
                  onClick={handleApprove}
                  className="w-full py-3 bg-gradient-to-r from-amber-500 to-amber-600 hover:from-amber-400 hover:to-amber-500 text-black font-mono text-xs font-bold tracking-wider rounded transition-all shadow-lg shadow-amber-500/20 flex items-center justify-center gap-2"
                >
                  <ShieldCheck className="w-4 h-4" />
                  <span>EXECUTE FOUR-EYES APPROVAL</span>
                </button>
              </div>

              {/* Status Message */}
              {statusMessage && (
                <div
                  className={`p-3 rounded font-mono text-xs flex items-start gap-2 ${
                    statusMessage.isError
                      ? 'bg-red-500/10 border border-red-500/30 text-red-300'
                      : 'bg-emerald-500/10 border border-emerald-500/30 text-emerald-300'
                  }`}
                >
                  {statusMessage.isError ? (
                    <XCircle className="w-4 h-4 text-red-400 flex-shrink-0 mt-0.5" />
                  ) : (
                    <CheckCircle2 className="w-4 h-4 text-emerald-400 flex-shrink-0 mt-0.5" />
                  )}
                  <span>{statusMessage.text}</span>
                </div>
              )}
            </div>
          </div>

          {/* Right Column: Live SHA-256 Chained Cryptographic Ledger */}
          <div className="lg:col-span-7 space-y-4">
            <div className="flex items-center justify-between font-mono text-xs text-slate-300 px-1">
              <div className="flex items-center gap-2">
                <Link2 className="w-4 h-4 text-amber-400" />
                <span className="font-bold">IMMUTABLE CRYPTOGRAPHIC AUDIT LEDGER</span>
              </div>
              <span className="text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded text-[10px]">
                CONTINUITY: VERIFIED
              </span>
            </div>

            <div className="space-y-3 max-h-[500px] overflow-y-auto pr-2">
              {ledger.map((block) => (
                <div
                  key={block.index}
                  className="p-4 rounded-xl bg-[#090c14] border border-white/10 hover:border-amber-500/30 font-mono text-xs transition-all relative crosshair"
                >
                  <div className="flex items-center justify-between border-b border-white/5 pb-2 mb-2">
                    <div className="flex items-center gap-2">
                      <span className="text-amber-400 font-bold">BLOCK #{block.index}</span>
                      <span className="text-white/20">|</span>
                      <span className="text-slate-300">{block.txId}</span>
                    </div>
                    <span className="text-emerald-400 font-bold">{block.amount}</span>
                  </div>

                  <div className="grid grid-cols-2 gap-2 text-[11px] mb-2 text-slate-400">
                    <div>
                      Maker: <span className="text-amber-300">{block.maker}</span>
                    </div>
                    <div>
                      Checker: <span className="text-cyan-300">{block.checker}</span>
                    </div>
                  </div>

                  <div className="space-y-1 text-[10px] text-slate-500 bg-[#04060b] p-2 rounded">
                    <div className="truncate">PrevHash: {block.prevHash}</div>
                    <div className="truncate text-amber-400/80 font-bold">BlockHash: {block.blockHash}</div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};
