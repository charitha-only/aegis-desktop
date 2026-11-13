export type Language = 'en' | 'si';

export interface Translations {
  appTitle: string;
  appSubtitle: string;
  systemReady: string;
  tabs: {
    security: string;
    banking: string;
    rasp: string;
    network: string;
    fido2: string;
    p2p: string;
    vault: string;
    filesystem: string;
    updater: string;
    plugins: string;
    wasm: string;
    diagnostics: string;
    logs: string;
    db: string;
    tpm: string;
    rbac: string;
    traffic: string;
    compliance: string;
    delta: string;
    pq: string;
    piv: string;
    tray: string;
    pqDsa: string;
    nativeIpc: string;
    memGuard: string;
    kernelSandbox: string;
    zkProof: string;
  };
  pqDsa: {
    title: string;
    subtitle: string;
    genKeypair: string;
    sign: string;
    verify: string;
    specMetadata: string;
    signatureStatus: string;
  };
  nativeIpc: {
    title: string;
    subtitle: string;
    runBenchmark: string;
    status: string;
    endpoint: string;
    latency: string;
    throughput: string;
  };
  memGuard: {
    title: string;
    subtitle: string;
    protectSecret: string;
    toggleAntiDump: string;
    dpapiStatus: string;
    lockedPages: string;
  };
  kernelSandbox: {
    title: string;
    subtitle: string;
    enforce: string;
    backend: string;
    isolationTier: string;
    childProcessBlocked: string;
  };
  zkProof: {
    title: string;
    subtitle: string;
    generateProof: string;
    verifyProof: string;
    statement: string;
    soundness: string;
  };
  delta: {
    title: string;
    subtitle: string;
    createPatch: string;
    verifyPatch: string;
    applyPatch: string;
    bytesSaved: string;
    ratio: string;
  };
  pq: {
    title: string;
    subtitle: string;
    genKeypair: string;
    encaps: string;
    decaps: string;
    specMetadata: string;
    sharedSecret: string;
  };
  piv: {
    title: string;
    subtitle: string;
    verifyPin: string;
    logout: string;
    signSlot: string;
    certExport: string;
    retries: string;
    lockoutStatus: string;
  };
  tray: {
    title: string;
    subtitle: string;
    quickKillswitch: string;
    lockWorkstation: string;
    statusBadge: string;
    menuActions: string;
  };
  db: {
    title: string;
    subtitle: string;
    queryAccounts: string;
    insertRecord: string;
    startTx: string;
    commitTx: string;
    rollbackTx: string;
    stats: string;
  };
  tpm: {
    title: string;
    subtitle: string;
    pcrList: string;
    sealSecret: string;
    unsealSecret: string;
    tamperSim: string;
    tamperDesc: string;
  };
  rbac: {
    title: string;
    subtitle: string;
    activeRole: string;
    switchRole: string;
    checkPerm: string;
    matrix: string;
    auditLog: string;
  };
  traffic: {
    title: string;
    subtitle: string;
    killswitchActive: string;
    killswitchInactive: string;
    toggleKillswitch: string;
    simulateTraffic: string;
    clearHistory: string;
    inspected: string;
    blocked: string;
  };
  compliance: {
    title: string;
    subtitle: string;
    generateCert: string;
    verifyCert: string;
    viewPrintable: string;
    asvsScore: string;
  };
  banking: {
    title: string;
    screenTitle: string;
    screenDesc: string;
    screenProtected: string;
    screenUnprotected: string;
    enableProtection: string;
    disableProtection: string;
    makerCheckerTitle: string;
    makerDesc: string;
    proposeTx: string;
    approveTx: string;
    rejectTx: string;
    ledgerTitle: string;
    verifyLedger: string;
    stepUpTitle: string;
    stepUpDesc: string;
  };
  rasp: {
    title: string;
    subtitle: string;
    integrityTier: string;
    debuggerStatus: string;
    memoryHooks: string;
    failClosed: string;
    runScan: string;
    resetContainment: string;
    clean: string;
    detected: string;
  };
  network: {
    title: string;
    subtitle: string;
    pinStatus: string;
    mitmTrap: string;
    evaluate: string;
    enrollHost: string;
    cleanMitm: string;
  };
  fido2: {
    title: string;
    subtitle: string;
    devicesFound: string;
    registerKey: string;
    signAssertion: string;
    touchRequired: string;
    touchConfirmed: string;
  };
  p2p: {
    title: string;
    subtitle: string;
    initSession: string;
    completeHandshake: string;
    encryptVault: string;
    decryptVault: string;
    macVerified: string;
  };
  common: {
    status: string;
    actions: string;
    verified: string;
    denied: string;
    refresh: string;
    loading: string;
    copy: string;
    success: string;
    error: string;
  };
}

export const translations: Record<Language, Translations> = {
  en: {
    appTitle: 'AEGIS DESKTOP',
    appSubtitle: 'ZERO-TRUST CYBERNETIC FRAMEWORK',
    systemReady: 'All Enclaves Operational',
    tabs: {
      security: 'Security Posture',
      banking: 'Financial Defense',
      rasp: 'RASP & Anti-Tamper',
      network: 'mTLS & Pinning',
      fido2: 'FIDO2 Hardware Key',
      p2p: 'P2P Encrypted Sync',
      vault: 'Argon2 Vault',
      filesystem: 'Sandboxed FS',
      updater: 'Auto-Updater',
      plugins: 'Sandboxed Plugins',
      wasm: 'Wasm Fuel Sandbox',
      diagnostics: 'Diagnostics',
      logs: 'Audit Logs',
      db: 'Encrypted DB',
      tpm: 'TPM PCR Enclave',
      rbac: 'Enterprise RBAC',
      traffic: 'Traffic Monitor',
      compliance: 'Security Audit',
      delta: 'Differential Delta Updater',
      pq: 'Post-Quantum KEM',
      piv: 'Smart Card & PIV',
      tray: 'Tray & Killswitch',
      pqDsa: 'Post-Quantum Signatures',
      nativeIpc: 'Sub-ms Native IPC',
      memGuard: 'In-Memory RAM Guard',
      kernelSandbox: 'OS Kernel Sandbox',
      zkProof: 'ZK Compliance Proofs',
    },
    delta: {
      title: 'Differential Binary Delta Auto-Updater (Quantum-Safe Patching)',
      subtitle: 'Byte-level binary diffing, SHA-256 target validation, and zero-downtime rollback journal.',
      createPatch: 'Generate Binary Delta Patch',
      verifyPatch: 'Verify Delta Patch Integrity',
      applyPatch: 'Atomically Apply Delta Update',
      bytesSaved: 'Bandwidth Saved',
      ratio: 'Compression Efficiency',
    },
    pq: {
      title: 'Post-Quantum Hybrid Cryptography (NIST FIPS 203)',
      subtitle: 'Dual-envelope KEM combining classical X25519 with lattice-based ML-KEM-768 for quantum Shor immunity.',
      genKeypair: 'Generate Hybrid Keypair (1216B PK / 2432B SK)',
      encaps: 'Encapsulate Quantum-Safe Ciphertext (1120B)',
      decaps: 'Decapsulate & Verify Dual Shared Secret',
      specMetadata: 'View NIST FIPS 203 Audit Metadata',
      sharedSecret: '256-bit Hybrid Quantum Shared Secret',
    },
    piv: {
      title: 'Smart Card & YubiKey PIV Slot Management (NIST SP 800-73-4)',
      subtitle: 'Hardware-grade PKCS#11 slots (9A Auth, 9C Digital Signature, 9D Key Mgmt, 9E Card Auth) with 3-attempt lockout defense.',
      verifyPin: 'Authenticate Smart Card User PIN',
      logout: 'Revoke Session Unlock',
      signSlot: 'Execute Hardware-Bound Digital Signature',
      certExport: 'Export Slot X.509 Certificate Metadata',
      retries: 'PIN Retries Remaining',
      lockoutStatus: 'Hardware Lockout Defense',
    },
    tray: {
      title: 'Native OS System Tray Daemon & Quick-Killswitch Controls',
      subtitle: 'Always-on background defensive monitor, real-time security badge, instant panic air-gap, and Maker-Checker alerts.',
      quickKillswitch: 'Emergency Air-Gap Killswitch',
      lockWorkstation: 'Lock Workstation Immediately',
      statusBadge: 'Security Enclave Badge',
      menuActions: 'Select Tray Action',
    },
    pqDsa: {
      title: 'Post-Quantum Digital Signatures (NIST FIPS 204 ML-DSA-65 / Dilithium)',
      subtitle: 'Lattice-based digital signatures combined with Ed25519 for dual-defense quantum forgery immunity.',
      genKeypair: 'Generate Hybrid Keypair (1984B PK / 4064B SK)',
      sign: 'Sign Payload with Hybrid Secret Key',
      verify: 'Verify Dual Classical + Quantum Signature',
      specMetadata: 'View NIST FIPS 204 Specifications',
      signatureStatus: 'Dual-Envelope Signature Verified',
    },
    nativeIpc: {
      title: 'Sub-Millisecond Native OS IPC Transport (Named Pipes & Sockets)',
      subtitle: 'Zero-copy length-delimited binary framing bypassing loopback TCP network overhead with kernel ACLs.',
      runBenchmark: 'Run Latency & Throughput Benchmark',
      status: 'Transport Daemon Health',
      endpoint: 'OS Kernel Endpoint',
      latency: 'Average Roundtrip Latency',
      throughput: 'Message Throughput Rate',
    },
    memGuard: {
      title: 'Kernel In-Memory Protection & RAM Anti-Dump Sentinel',
      subtitle: 'Transient DPAPI memory encryption and VirtualLock page pinning preventing Mimikatz and pagefile dumps.',
      protectSecret: 'Encrypt Secret in Volatile RAM',
      toggleAntiDump: 'Toggle Anti-Dump Sentinel',
      dpapiStatus: 'DPAPI RAM Protection State',
      lockedPages: 'Physical Memory Pages Pinned',
    },
    kernelSandbox: {
      title: 'OS Kernel Sandboxing (Windows Job Objects & Landlock LSM)',
      subtitle: 'Kernel-level process boundary enforcement restricting rogue child execution, desktop handles, and memory caps.',
      enforce: 'Enforce Kernel Policy',
      backend: 'Active Kernel Sandbox Backend',
      isolationTier: 'OS Confinement Tier',
      childProcessBlocked: 'Child Process Spawning Blocked',
    },
    zkProof: {
      title: 'Privacy-Preserving Zero-Knowledge (ZK) Compliance Proofs',
      subtitle: 'Dual-base Schnorr & Bulletproof range proofs verifying values without disclosing sensitive balances or amounts.',
      generateProof: 'Generate ZK Range Proof',
      verifyProof: 'Verify Range Statement',
      statement: 'Certified Mathematical Statement',
      soundness: 'Computational Soundness',
    },
    db: {
      title: 'Encrypted Embedded Database Engine',
      subtitle: 'ChaCha20-Poly1305 authenticated encryption, ACID transactions, and tamper-resistant storage.',
      queryAccounts: 'Query Accounts Table',
      insertRecord: 'Insert Record',
      startTx: 'Begin Transaction',
      commitTx: 'Commit Transaction',
      rollbackTx: 'Rollback Transaction',
      stats: 'Database Health & Cipher Specs',
    },
    tpm: {
      title: 'Hardware TPM 2.0 PCR-Sealed Cryptographic Enclave',
      subtitle: 'Hardware Root of Trust with cryptographic sealing to Platform Configuration Registers (0, 2, 7, 11).',
      pcrList: 'Measured Hardware PCR Registers',
      sealSecret: 'Seal Secret to Current PCR Composite',
      unsealSecret: 'Unseal & Decrypt Secret',
      tamperSim: 'Simulate Bootloader/Firmware Tamper',
      tamperDesc: 'Simulates unauthorized rootkit or modified kernel, causing fail-closed enclave denial.',
    },
    rbac: {
      title: 'Enterprise Granular RBAC Engine',
      subtitle: 'Five corporate tiers with Basel III & PCI-DSS Segregation of Duties and dual-control enforcement.',
      activeRole: 'Active Corporate Identity',
      switchRole: 'Assume Enterprise Role',
      checkPerm: 'Evaluate Capability Permission',
      matrix: 'Enterprise Role Permission Matrix',
      auditLog: 'Privileged Access Audit Trail',
    },
    traffic: {
      title: 'Outbound Network Firewall & Real-Time Flow Monitor',
      subtitle: 'Zero-trust packet inspection, latency measurement, threat scoring, and emergency air-gap killswitch.',
      killswitchActive: 'AIR-GAP KILLSWITCH ACTIVE (100% BLOCKED)',
      killswitchInactive: 'OUTBOUND TRAFFIC FILTERING ACTIVE',
      toggleKillswitch: 'Toggle Air-Gap Panic Killswitch',
      simulateTraffic: 'Inject Test Outbound Packet',
      clearHistory: 'Clear Packet History',
      inspected: 'Total Flows Inspected',
      blocked: 'Threats Neutralized & Blocked',
    },
    compliance: {
      title: 'Automated Security Audit & Compliance Certificate Generator',
      subtitle: 'Real-time telemetry aggregator evaluating OWASP ASVS 5.0 (143/143 controls) with Ed25519 digital signature.',
      generateCert: 'Generate Attested Certificate',
      verifyCert: 'Verify Cryptographic Signature',
      viewPrintable: 'View Printable Certificate',
      asvsScore: 'Verification Score',
    },
    banking: {
      title: 'Enterprise Banking & Financial Suite',
      screenTitle: 'Win32 DWM Anti-Screen Capture',
      screenDesc: 'Hardware-enforced video blackout preventing OBS, Zoom, and malware frame grabbing.',
      screenProtected: 'DWM PROTECTION ACTIVE (WDA_EXCLUDEFROMCAPTURE)',
      screenUnprotected: 'SCREEN CAPTURE ALLOWED',
      enableProtection: 'Activate Anti-Capture Shield',
      disableProtection: 'Deactivate Protection',
      makerCheckerTitle: 'Maker-Checker Four-Eyes Consensus',
      makerDesc: 'Segregation of duties enforcing dual-signature cryptographic sign-off for corporate transactions.',
      proposeTx: 'Propose Transaction (Maker)',
      approveTx: 'Approve Transaction (Checker)',
      rejectTx: 'Reject Transaction',
      ledgerTitle: 'Cryptographic Chained Ledger',
      verifyLedger: 'Verify SHA-256 Ledger Continuity',
      stepUpTitle: 'Windows Hello / Touch ID Step-Up',
      stepUpDesc: 'Biometric challenge verification for high-value financial transfers.',
    },
    rasp: {
      title: 'Runtime Application Self-Protection (RASP)',
      subtitle: 'Kernel-level debugger detection, memory hook scanning, and fail-closed containment.',
      integrityTier: 'System Integrity Tier',
      debuggerStatus: 'Win32 Debugger Attachment',
      memoryHooks: 'VirtualProtect Memory Hooks',
      failClosed: 'Fail-Closed Containment Active',
      runScan: 'Execute Real-Time RASP Scan',
      resetContainment: 'Reset Containment State',
      clean: 'NOMINAL / CLEAN',
      detected: 'THREAT DETECTED',
    },
    network: {
      title: 'mTLS & SPKI Certificate Pinning Sentinel',
      subtitle: 'Zero-trust proxy defense blocking Burp Suite, Charles, and rogue CA MITM interception.',
      pinStatus: 'SPKI Public Key Hash Pinning',
      mitmTrap: 'MITM Interception Detection',
      evaluate: 'Evaluate Gateway Connection',
      enrollHost: 'Enroll Pinned Endpoint',
      cleanMitm: 'Connection Pure // No Proxy Detected',
    },
    fido2: {
      title: 'Physical Hardware Security Key (FIDO2 / YubiKey)',
      subtitle: 'Hardware-bound cryptographic signing with mandatory user physical touch presence.',
      devicesFound: 'Enumerate CTAP2 Tokens',
      registerKey: 'Register FIDO2 Credential',
      signAssertion: 'Sign with Hardware Token',
      touchRequired: 'Touch Presence Required',
      touchConfirmed: 'Physical Touch Confirmed',
    },
    p2p: {
      title: 'Decentralized P2P Zero-Knowledge Vault Sync',
      subtitle: 'Noise-style direct workstation synchronization with symmetric session key derivation.',
      initSession: 'Initiate P2P Handshake',
      completeHandshake: 'Complete Mutual Handshake',
      encryptVault: 'Encrypt Differential Bundle',
      decryptVault: 'Decrypt & Verify Bundle',
      macVerified: 'HMAC-SHA256 Authenticity Tag Verified',
    },
    common: {
      status: 'Status',
      actions: 'Actions',
      verified: 'VERIFIED',
      denied: 'DENIED',
      refresh: 'Refresh',
      loading: 'Processing...',
      copy: 'Copy',
      success: 'Operation Successful',
      error: 'Security Policy Denial',
    },
  },
  si: {
    appTitle: 'ඒජිස් ඩෙස්ක්ටොප් (AEGIS)',
    appSubtitle: 'ශුන්‍ය විශ්වාස (ZERO-TRUST) ආරක්ෂක පද්ධතිය',
    systemReady: 'සියලුම ආරක්ෂක ස්ථර සක්‍රීයයි',
    tabs: {
      security: 'ආරක්ෂක තත්ත්වය',
      banking: 'මූල්‍ය ආරක්ෂණ පද්ධතිය',
      rasp: 'RASP ස්වයං-ආරක්ෂණය',
      network: 'mTLS සහ Pinning ආරක්ෂණය',
      fido2: 'FIDO2 / YubiKey යතුර',
      p2p: 'P2P Zero-Knowledge සමමුහුර්තය',
      vault: 'Argon2 රහස් සේප්පුව',
      filesystem: 'වැටකෙයියා කළ ලිපිගොනු',
      updater: 'ස්වයංක්‍රීය යාවත්කාලීන',
      plugins: 'ප්ලගීන පද්ධතිය',
      wasm: 'Wasm Fuel වැලිපිල්ල',
      diagnostics: 'පද්ධති රෝග විනිශ්චය',
      logs: 'විගණන සටහන් (Logs)',
      db: 'ගුප්තකේතන දත්ත සමුදාය',
      tpm: 'TPM PCR රහස් කලාපය',
      rbac: 'ආයතනික RBAC පද්ධතිය',
      traffic: 'ජාල ගමනාගමන නිරීක්ෂකය',
      compliance: 'ආරක්ෂක විගණන සහතිකය',
      delta: 'ද්විමය ඩෙල්ටා යාවත්කාලීන',
      pq: 'පශ්චාත්-ක්වොන්ටම් KEM',
      piv: 'ස්මාර්ට් කාඩ් සහ PIV',
      tray: 'පද්ධති තැටිය සහ Killswitch',
      pqDsa: 'පශ්චාත්-ක්වොන්ටම් අත්සන්',
      nativeIpc: 'අතිවේගී ස්වදේශික IPC',
      memGuard: 'මතකයේ සංකේතන ආරක්ෂණය',
      kernelSandbox: 'කර්නල් මට්ටමේ සීමා පවුර',
      zkProof: 'ශුන්‍ය-දැනුම් අනුකූලතා සාධන',
    },
    delta: {
      title: 'අවකල්‍ය ද්විමය ඩෙල්ටා ස්වයංක්‍රීය යාවත්කාලීන පද්ධතිය',
      subtitle: 'බයිට් මට්ටමේ ද්විමය වෙනස්කම් (Diffing), SHA-256 අඛණ්ඩතා පරීක්ෂාව සහ ආරක්ෂිත Rollback පද්ධතිය.',
      createPatch: 'ඩෙල්ටා පැච් එකක් සාදන්න',
      verifyPatch: 'ඩෙල්ටා පැච් එකේ අඛණ්ඩතාව පරීක්ෂා කරන්න',
      applyPatch: 'යාවත්කාලීනය ක්ෂණිකව ස්ථාපනය කරන්න',
      bytesSaved: 'ඉතිරි වූ දත්ත ප්‍රමාණය (Bandwidth)',
      ratio: 'දත්ත සම්පීඩන කාර්යක්ෂමතාව',
    },
    pq: {
      title: 'පශ්චාත්-ක්වොන්ටම් දෙමුහුන් ගුප්තකේතනය (NIST FIPS 203)',
      subtitle: 'ක්වොන්ටම් පරිගණක ප්‍රහාර (Shor\'s Algorithm) වලට ඔරොත්තු දෙන X25519 සහ ML-KEM-768 දෙමුහුන් පද්ධතිය.',
      genKeypair: 'දෙමුහුන් යතුරු යුගලක් සාදන්න (1216B PK / 2432B SK)',
      encaps: 'ක්වොන්ටම්-ආරක්ෂිත Ciphertext (1120B) ගුප්තකේතනය කරන්න',
      decaps: 'රහස් යතුරෙන් විකේතනය කර හවුල් රහස තහවුරු කරන්න',
      specMetadata: 'NIST FIPS 203 තාක්ෂණික විස්තර බලන්න',
      sharedSecret: '256-bit දෙමුහුන් ක්වොන්ටම් හවුල් රහස',
    },
    piv: {
      title: 'ස්මාර්ට් කාඩ් සහ YubiKey PIV කළමනාකරණය (NIST SP 800-73-4)',
      subtitle: 'දෘඩාංග PKCS#11 Slots (9A Auth, 9C Digital Signature, 9D Key Mgmt, 9E Card Auth) සහ 3-වර අවහිරතා ආරක්ෂණය.',
      verifyPin: 'ස්මාර්ට් කාඩ් පරිශීලක PIN එක තහවුරු කරන්න',
      logout: 'කාඩ්පතේ සැසිය අගුලු දමන්න (Logout)',
      signSlot: 'දෘඩාංග ඩිජිටල් අත්සනක් තබන්න',
      certExport: 'Slot X.509 සහතික තොරතුරු ලබාගන්න',
      retries: 'ඉතිරිව ඇති PIN උත්සාහයන්',
      lockoutStatus: 'දෘඩාංග අගුලු දැමීමේ තත්ත්වය',
    },
    tray: {
      title: 'ස්වදේශික පද්ධති තැටි ඩීමන් සහ ක්ෂණික Killswitch පාලකය',
      subtitle: 'පසුබිමේ ක්‍රියාත්මක ආරක්ෂණ නිරීක්ෂකය, සජීවී ආරක්ෂණ ලාංඡනය, ක්ෂණික ජාල විසන්ධි කිරීම සහ අනුමැති අනතුරු ඇඟවීම්.',
      quickKillswitch: 'ක්ෂණික හදිසි වායු-පරතර (Air-Gap) Killswitch',
      lockWorkstation: 'වහාම Zero-Trust අගුලු දමන්න',
      statusBadge: 'ආරක්ෂණ කලාපයේ තත්ත්වය',
      menuActions: 'තැටි මෙනු ක්‍රියාමාර්ගයක් තෝරන්න',
    },
    pqDsa: {
      title: 'පශ්චාත්-ක්වොන්ටම් ඩිජිටල් අත්සන් (NIST FIPS 204 ML-DSA-65 / Dilithium)',
      subtitle: 'Ed25519 සමඟ ඒකාබද්ධ වූ දැලිස්-පාදක (Lattice-based) ද්විත්ව-ආරක්ෂණ ක්වොන්ටම් ව්‍යාජ-නොකළ හැකි අත්සන් පද්ධතිය.',
      genKeypair: 'දෙමුහුන් යතුරු යුගලක් සාදන්න (1984B PK / 4064B SK)',
      sign: 'රහස් යතුරෙන් දත්ත පෙළ ඩිජිටල්ව අත්සන් කරන්න',
      verify: 'දෙමුහුන් සම්භාව්‍ය + ක්වොන්ටම් අත්සන සත්‍යාපනය කරන්න',
      specMetadata: 'NIST FIPS 204 ප්‍රමිතියේ තාක්ෂණික විස්තර',
      signatureStatus: 'ද්විත්ව-ආවරණ අත්සන සාර්ථකව තහවුරු විය',
    },
    nativeIpc: {
      title: 'ක්ෂුද්‍ර තත්පර මට්ටමේ ස්වදේශික OS IPC නාලිකාව (Named Pipes & Sockets)',
      subtitle: 'TCP/IP loopback ප්‍රමාදය ඉවත් කර සෘජු කර්නල් ACLs සහිත Zero-copy ද්විමය පද්ධති සන්නිවේදනය.',
      runBenchmark: 'වේගය සහ කාර්යක්ෂමතාව (Benchmark) පරීක්ෂා කරන්න',
      status: 'ස්වදේශික ප්‍රවාහන ඩීමන් තත්ත්වය',
      endpoint: 'OS කර්නල් අන්ත ලක්ෂ්‍යය',
      latency: 'සාමාන්‍ය රවුම් ගමන් කාලය (Latency)',
      throughput: 'තත්පරයකට පණිවිඩ සංඛ්‍යාව (Throughput)',
    },
    memGuard: {
      title: 'කර්නල් මට්ටමේ RAM මතක සංකේතනය සහ Anti-Dump ආරක්ෂකයා',
      subtitle: 'මතකයේ පවතින රහස් සංකේතනය (DPAPI) සහ පිටු මාරුවීම් වැළැක්වීම (VirtualLock) මඟින් Mimikatz dump අවහිර කිරීම.',
      protectSecret: 'RAM මතකයේ රහස සංකේතනය කර සුරකින්න',
      toggleAntiDump: 'Anti-Dump මුරපොළ සක්‍රීය/අක්‍රීය කරන්න',
      dpapiStatus: 'DPAPI RAM ආරක්ෂණ තත්ත්වය',
      lockedPages: 'භෞතික මතකයේ රඳවා ඇති පිටු ගණන',
    },
    kernelSandbox: {
      title: 'OS කර්නල් මට්ටමේ සීමා පවුර (Windows Job Objects & Landlock LSM)',
      subtitle: 'අනවසර Child Process ක්‍රියාත්මක වීම්, ඩෙස්ක්ටොප් හැසිරවීම් සහ උපරිම මතක සීමා පනවන කර්නල් ආරක්ෂණය.',
      enforce: 'කර්නල් ප්‍රතිපත්තිය බලාත්මක කරන්න',
      backend: 'ක්‍රියාකාරී කර්නල් සැන්ඩ්බොක්ස් තාක්ෂණය',
      isolationTier: 'OS සීමා කිරීමේ මට්ටම (Confinement Tier)',
      childProcessBlocked: 'අනුබද්ධ ක්‍රියාවලි (Child Processes) අවහිර කර ඇත',
    },
    zkProof: {
      title: 'රහස්‍යතා-ආරක්ෂිත ශුන්‍ය-දැනුම් (ZK) අනුකූලතා සාධන',
      subtitle: 'සැබෑ ගිණුම් ශේෂ හෝ මුදල් ප්‍රමාණය හෙළි නොකර අගයන් නියමිත පරාසයක පවතින බව ඔප්පු කරන Bulletproofs පද්ධතිය.',
      generateProof: 'ZK පරාස සාධනයක් (Range Proof) සාදන්න',
      verifyProof: 'පරාස ප්‍රකාශනය සත්‍යාපනය කරන්න',
      statement: 'සත්‍යාපිත ගණිතමය ප්‍රකාශය',
      soundness: 'ගණනය කිරීමේ නිරවද්‍යතාව (Soundness)',
    },
    db: {
      title: 'ගුප්තකේතනය කළ අභ්‍යන්තර දත්ත සමුදාය (ChaCha20-Poly1305)',
      subtitle: 'ChaCha20-Poly1305 AEAD සත්‍යාපිත ගුප්තකේතනය, ACID ගනුදෙනු සහ අනවසර වෙනස්කම් වැළැක්වීමේ ගබඩාව.',
      queryAccounts: 'ගිණුම් ලේඛනය විමසන්න (Accounts Query)',
      insertRecord: 'නව වාර්තාවක් ඇතුළත් කරන්න',
      startTx: 'ගනුදෙනුවක් (Transaction) අරඹන්න',
      commitTx: 'ගනුදෙනුව සුරකින්න (Commit)',
      rollbackTx: 'ගනුදෙනුව ආපසු හරවන්න (Rollback)',
      stats: 'දත්ත සමුදායේ සෞඛ්‍යය සහ විස්තර',
    },
    tpm: {
      title: 'දෘඩාංග TPM 2.0 PCR-Sealed ගුප්තකේතන කලාපය',
      subtitle: 'දෘඩාංග මට්ටමේ Root of Trust සහ Platform Configuration Registers (0, 2, 7, 11) මඟින් රහස් මුද්‍රා තැබීම.',
      pcrList: 'මනින ලද දෘඩාංග PCR රෙජිස්ටර',
      sealSecret: 'වත්මන් PCR එකතුවට රහසක් මුද්‍රා තබන්න (Seal)',
      unsealSecret: 'මුද්‍රාව ඉවත් කර රහස කියවන්න (Unseal)',
      tamperSim: 'Bootloader/Firmware වෙනස්වීමක් අනුකරණය කරන්න',
      tamperDesc: 'අනවසර rootkit හෝ වෙනස් කළ kernel එකක් අනුකරණය කරමින් Fail-Closed ලෙස ප්‍රවේශය අවහිර කරයි.',
    },
    rbac: {
      title: 'ආයතනික ශ්‍රේණිගත RBAC අවසර පාලනය',
      subtitle: 'ආයතනික තනතුරු 5ක්, Basel III සහ PCI-DSS කාර්ය වෙන්කිරීම (SoD) සහ ද්විත්ව අනුමැතිය.',
      activeRole: 'ක්‍රියාකාරී ආයතනික අනන්‍යතාව',
      switchRole: 'තනතුර මාරු කරන්න',
      checkPerm: 'ක්‍රියාකාරී අවසරය පරීක්ෂා කරන්න',
      matrix: 'සම්පූර්ණ ආයතනික අවසර වගුව',
      auditLog: 'ප්‍රවේශ විගණන වාර්තාව (Audit Trail)',
    },
    traffic: {
      title: 'පිටතට යන ජාල ගමනාගමන නිරීක්ෂක සහ ගිනිපවුර (Firewall)',
      subtitle: 'Zero-trust පැකට් පරීක්ෂාව, ප්‍රතිචාර කාලය, තර්ජන ලකුණු සහ හදිසි Air-Gap අගුල (Killswitch).',
      killswitchActive: 'හදිසි AIR-GAP අගුල ක්‍රියාත්මකයි (100% ක් අවහිරයි)',
      killswitchInactive: 'ජාල පෙරහන් ක්‍රියාකාරීයි (පිටතට සබඳතා අවසර)',
      toggleKillswitch: 'හදිසි Air-Gap අගුල ක්‍රියාත්මක/අක්‍රිය කරන්න',
      simulateTraffic: 'පරීක්ෂණ ජාල පැකට්ටුවක් යවන්න',
      clearHistory: 'පැකට් ඉතිහාසය මකන්න',
      inspected: 'පරීක්ෂා කළ මුළු පැකට්',
      blocked: 'අවහිර කළ තර්ජන සංඛ්‍යාව',
    },
    compliance: {
      title: 'ස්වයංක්‍රීය ආරක්ෂක විගණන සහතික ජනකය (OWASP ASVS 5.0)',
      subtitle: 'සියලුම ආරක්ෂක දත්ත එකතු කර OWASP ASVS 5.0 (143/143) සහතිකය Ed25519 අත්සනින් නිකුත් කිරීම.',
      generateCert: 'තහවුරු කළ සහතිකය ලබාගන්න',
      verifyCert: 'ඩිජිටල් අත්සන පරීක්ෂා කරන්න',
      viewPrintable: 'මුද්‍රණය කළ හැකි සහතිකය බලන්න',
      asvsScore: 'විගණන ලකුණු ප්‍රමාණය',
    },
    banking: {
      title: 'ආයතනික බැංකු හා මූල්‍ය ආරක්ෂණ පද්ධතිය',
      screenTitle: 'Win32 DWM තිර ග්‍රහණ වැළැක්වීම (Anti-Screen Capture)',
      screenDesc: 'OBS Studio, Zoom හෝ malware මඟින් තිරය record කිරීම වැළැක්වීමේ දෘඩාංග මට්ටමේ ආරක්ෂාව.',
      screenProtected: 'DWM තිර ආරක්ෂණය සක්‍රීයයි (WDA_EXCLUDEFROMCAPTURE)',
      screenUnprotected: 'තිර ග්‍රහණය සඳහා අවසර දී ඇත',
      enableProtection: 'තිර ආරක්ෂණ පලිහ සක්‍රීය කරන්න',
      disableProtection: 'ආරක්ෂණය අක්‍රීය කරන්න',
      makerCheckerTitle: 'Maker-Checker ද්විත්ව අනුමැතිය (Four-Eyes)',
      makerDesc: 'මූල්‍ය ගනුදෙනු තනි පුද්ගලයෙකුට සිදු කළ නොහැකි ලෙස දෙදෙනෙකුගේ අත්සන් අනිවාර්ය කිරීමේ නීතිය.',
      proposeTx: 'නව ගනුදෙනුවක් යෝජනා කරන්න (Maker)',
      approveTx: 'ගනුදෙනුව අනුමත කරන්න (Checker)',
      rejectTx: 'ගනුදෙනුව ප්‍රතික්ෂේප කරන්න',
      ledgerTitle: 'ගුප්තකේතන ලෙජරය (Immutable Ledger)',
      verifyLedger: 'SHA-256 ලෙජරයේ අඛණ්ඩතාව තහවුරු කරන්න',
      stepUpTitle: 'Windows Hello / Touch ID ජෛවමිතික තහවුරු කිරීම',
      stepUpDesc: 'විශාල වටිනාකමකින් යුතු මුදල් හුවමාරු සඳහා ඇඟිලි සලකුණු හෝ මුහුණු හඳුනාගැනීම.',
    },
    rasp: {
      title: 'යෙදුම් මට්ටමේ ස්වයං-ආරක්ෂණය (RASP & Anti-Tamper)',
      subtitle: 'Debugger එකතු වීම්, මතක කොකු (Memory Hooks) හඳුනාගැනීම සහ Fail-Closed අගුලු දැමීම.',
      integrityTier: 'පද්ධති අඛණ්ඩතා මට්ටම',
      debuggerStatus: 'Win32 Debugger පරීක්ෂාව',
      memoryHooks: 'VirtualProtect මතක කොකු පරීක්ෂාව',
      failClosed: 'Fail-Closed ආරක්ෂණ අගුල සක්‍රීයයි',
      runScan: 'RASP ක්ෂණික පරීක්ෂාවක් සිදු කරන්න',
      resetContainment: 'ආරක්ෂණ අගුල යළි සකසන්න',
      clean: 'ආරක්ෂිතයි / නිරවුල්ය',
      detected: 'තර්ජනයක් හඳුනාගන්නා ලදී',
    },
    network: {
      title: 'mTLS සහ SPKI සහතික Pinning ආරක්ෂකය',
      subtitle: 'Burp Suite, Charles වැනි MITM proxy මඟින් දත්ත පැහැරගැනීම මුළුමනින්ම වළක්වන පද්ධතිය.',
      pinStatus: 'SPKI Public Key Hash Pinning',
      mitmTrap: 'MITM මැදිහත්වීම් හඳුනාගැනීම',
      evaluate: 'ජාල සබඳතාව පරීක්ෂා කරන්න',
      enrollHost: 'නව සේවාදායකයක් Pin කරන්න',
      cleanMitm: 'සබඳතාව ආරක්ෂිතයි // Proxy අනාවරණය නොවීය',
    },
    fido2: {
      title: 'භෞතික ආරක්ෂක යතුර (FIDO2 / YubiKey)',
      subtitle: 'යතුරු පුවරුව භෞතිකව ස්පර්ශ කිරීමෙන් තොරව අත්සන් තැබිය නොහැකි CTAP2 දෘඩාංග ආරක්ෂාව.',
      devicesFound: 'සම්බන්ධිත FIDO2 යතුරු සොයන්න',
      registerKey: 'නව FIDO2 යතුරක් ලියාපදිංචි කරන්න',
      signAssertion: 'දෘඩාංග යතුරෙන් අත්සන් කරන්න',
      touchRequired: 'භෞතික ස්පර්ශය අවශ්‍යයි',
      touchConfirmed: 'භෞතික ස්පර්ශය තහවුරු විය',
    },
    p2p: {
      title: 'විමධ්‍යගත P2P Zero-Knowledge සේප්පු සමමුහුර්තය',
      subtitle: 'මධ්‍යම සේවාදායකයන් රහිතව පරිගණක දෙකක් අතර සේප්පු රහසිගතව sync කිරීම (Noise Protocol).',
      initSession: 'P2P Handshake ආරම්භ කරන්න',
      completeHandshake: 'ද්විපාර්ශ්වික අත්සන සම්පූර්ණ කරන්න',
      encryptVault: 'සමමුහුර්ත මිටිය ගුප්තකේතනය කරන්න',
      decryptVault: 'මිටිය විකේතනය කර තහවුරු කරන්න',
      macVerified: 'HMAC-SHA256 අඛණ්ඩතා ටැගය තහවුරු විය',
    },
    common: {
      status: 'තත්ත්වය',
      actions: 'ක්‍රියාමාර්ග',
      verified: 'තහවුරු විය',
      denied: 'ප්‍රතික්ෂේප විය',
      refresh: 'නැවුම් කරන්න',
      loading: 'ක්‍රියාත්මක වෙමින්...',
      copy: 'පිටපත් කරන්න',
      success: 'ක්‍රියාව සාර්ථකයි',
      error: 'ආරක්ෂක ප්‍රතිපත්ති අවහිරයකි',
    },
  },
};

