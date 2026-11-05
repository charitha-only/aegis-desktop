import {aegis, type ManagedWindowId} from '../aegis-bridge';

/** Supply a bundled page URL on this app's origin; creation does not confirm page readiness. */
export async function openManagedChild(childPageUrl: string): Promise<ManagedWindowId> {
  const capabilities = await aegis.getWindowCapabilities();
  if (!capabilities.success || capabilities.data?.multiWindowSupported !== true) {
    throw new Error(capabilities.error || 'Managed native windows are unavailable or not permitted');
  }
  const created = await aegis.createManagedWindow({url: childPageUrl, title: 'Application child', width: 800, height: 600});
  if (!created.success || !created.data) throw new Error(created.error || 'Creation returned no managed handle');
  return created.data.windowId;
}

/** Close only the handle retained by this feature, rather than every app-wide listed child. */
export async function closeOwnedChild(windowId: ManagedWindowId): Promise<void> {
  const closed = await aegis.closeManagedWindow(windowId);
  if (!closed.success) throw new Error(closed.error || 'Managed child close was not confirmed');
}


/** An accepted request is not proof the OS focused the child. */
export async function requestOwnedChildFocus(windowId: ManagedWindowId): Promise<void> {
  const result = await aegis.focusManagedWindow(windowId);
  if (!result.success) throw new Error(result.error || 'Focus request was not accepted');
}

/** false requests unminimize; actual OS state is not confirmed by the receipt. */
export async function requestOwnedChildMinimized(windowId: ManagedWindowId, minimized: boolean): Promise<void> {
  const result = await aegis.setManagedWindowMinimized(windowId, minimized);
  if (!result.success) throw new Error(result.error || 'Minimize request was not accepted');
}
