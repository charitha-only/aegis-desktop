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

