export interface BridgeOriginOptions {
  production: boolean;
  appOrigin?: string;
  configuredOrigin?: string;
}
export function resolveBridgeOrigin(options: BridgeOriginOptions): string;
