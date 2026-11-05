use crate::window::{NativeWindowPolicy, WindowConfig};

use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

pub enum WindowOperation {
    Create { url: String, config: WindowConfig },
    List,
    Close(String),
    Focus(String),
    SetMinimized { id: String, minimized: bool },
}
pub struct WindowRequest {
    pub operation: WindowOperation,
    reply: SyncSender<Result<serde_json::Value, String>>,
    cancelled: Arc<AtomicBool>,
}
impl WindowRequest {
    pub fn finish(self, result: Result<serde_json::Value, String>) {
        let _ = self.reply.try_send(result);
    }
}
#[derive(Clone)]
pub struct WindowClient {
    policy: NativeWindowPolicy,
    sender: SyncSender<WindowRequest>,
}
pub struct WindowHost {
    receiver: Receiver<WindowRequest>,
}
pub struct WindowTicket {
    receiver: Receiver<Result<serde_json::Value, String>>,
    cancelled: Arc<AtomicBool>,
}
/// Owned transport only; a platform adapter must drain it on its UI event loop.
pub fn window_channel(policy: NativeWindowPolicy) -> (WindowClient, WindowHost) {
    let (sender, receiver) = mpsc::sync_channel(16);
    (WindowClient { policy, sender }, WindowHost { receiver })
}
impl WindowClient {
    pub fn create(&self, payload: &serde_json::Value) -> Result<WindowTicket, String> {
        let (url, config) = self.policy.prepare_child_request(payload)?;
        self.submit(WindowOperation::Create { url, config })
    }
    pub fn list(&self) -> Result<WindowTicket, String> {
        self.submit(WindowOperation::List)
    }
    pub fn close(&self, id: &str) -> Result<WindowTicket, String> {
        if !id.starts_with("child-")
            || id.len() > 32
            || !id[6..].bytes().all(|b| b.is_ascii_digit())
            || id.len() == 6
        {
            return Err("Invalid managed window ID".into());
        }
        self.submit(WindowOperation::Close(id.into()))
    }
    pub fn focus(&self, id: &str) -> Result<WindowTicket, String> {
        if !id.starts_with("child-")
            || id.len() > 32
            || id.len() == 6
            || !id[6..].bytes().all(|b| b.is_ascii_digit())
        {
            return Err("Invalid managed window ID".into());
        }
        self.submit(WindowOperation::Focus(id.into()))
    }
    pub fn set_minimized(&self, id: &str, minimized: bool) -> Result<WindowTicket, String> {
        if !id.starts_with("child-")
            || id.len() > 32
            || id.len() == 6
            || !id[6..].bytes().all(|b| b.is_ascii_digit())
        {
            return Err("Invalid managed window ID".into());
        }
        self.submit(WindowOperation::SetMinimized {
            id: id.into(),
            minimized,
        })
    }
    fn submit(&self, operation: WindowOperation) -> Result<WindowTicket, String> {
        let (reply, receiver) = mpsc::sync_channel(1);
        let cancelled = Arc::new(AtomicBool::new(false));
        self.sender
            .try_send(WindowRequest {
                operation,
                reply,
                cancelled: cancelled.clone(),
            })
            .map_err(|error| match error {
                mpsc::TrySendError::Full(_) => "Native window request queue is full".to_string(),
                mpsc::TrySendError::Disconnected(_) => {
                    "Native window host is unavailable".to_string()
                }
            })?;
        Ok(WindowTicket {
            receiver,
            cancelled,
        })
    }
}
impl WindowHost {
    pub fn next(&self) -> Option<WindowRequest> {
        while let Ok(request) = self.receiver.try_recv() {
            if !request.cancelled.load(Ordering::Acquire) {
                return Some(request);
            }
        }
        None
    }
}
impl Drop for WindowTicket {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
    }
}
impl WindowTicket {
    pub fn wait(self, timeout: Duration) -> Result<serde_json::Value, String> {
        self.receiver.recv_timeout(timeout).map_err(|error| {
            match error {
                mpsc::RecvTimeoutError::Timeout => {
                    "Native window result timed out; outcome is unknown"
                }
                mpsc::RecvTimeoutError::Disconnected => "Native window host ended without a result",
            }
            .to_string()
        })?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_requests_validate_origin_and_return_only_host_confirmed_results() {
        let policy =
            NativeWindowPolicy::new("http://127.0.0.1:48291/", &WindowConfig::default()).unwrap();
        let (client, host) = window_channel(policy);
        assert!(client
            .create(&serde_json::json!({"url":"http://127.0.0.1:48292/"}))
            .is_err());
        let ticket = client
            .create(&serde_json::json!({"title":"Editor"}))
            .unwrap();
        let request = host.next().unwrap();
        match &request.operation {
            WindowOperation::Create { url, config } => {
                assert_eq!(url, "http://127.0.0.1:48291/");
                assert_eq!(config.title, "Editor");
            }
            _ => panic!("expected create"),
        }
        request.finish(Ok("child-1".into()));
        assert_eq!(
            ticket.wait(std::time::Duration::from_secs(1)).unwrap(),
            "child-1"
        );
    }
    #[test]
    fn dropped_or_timed_out_tickets_are_not_dispatched_if_still_queued() {
        let policy =
            NativeWindowPolicy::new("http://127.0.0.1:48291/", &WindowConfig::default()).unwrap();
        let (client, host) = window_channel(policy);
        let ticket = client.create(&serde_json::json!({})).unwrap();
        drop(ticket);
        assert!(host.next().is_none());
        let ticket = client.create(&serde_json::json!({})).unwrap();
        assert!(ticket
            .wait(Duration::ZERO)
            .unwrap_err()
            .contains("outcome is unknown"));
        assert!(host.next().is_none());
    }

    #[test]
    fn busy_and_closed_hosts_fail_without_fabricating_success() {
        let policy =
            NativeWindowPolicy::new("http://127.0.0.1:48291/", &WindowConfig::default()).unwrap();
        let (client, host) = window_channel(policy);
        let mut tickets = Vec::new();
        for _ in 0..16 {
            tickets.push(client.create(&serde_json::json!({})).unwrap());
        }
        assert!(
            matches!(client.create(&serde_json::json!({})), Err(error) if error.contains("queue is full"))
        );
        let request = host.next().unwrap();
        request.finish(Err("TEST_ONLY creation failure".into()));
        assert_eq!(
            tickets.remove(0).wait(Duration::from_secs(1)).unwrap_err(),
            "TEST_ONLY creation failure"
        );
        drop(host);
        for ticket in tickets {
            assert!(ticket
                .wait(Duration::from_secs(1))
                .unwrap_err()
                .contains("without a result"));
        }
        assert!(
            matches!(client.create(&serde_json::json!({})), Err(error) if error.contains("unavailable"))
        );
    }
    #[test]
    fn lifecycle_transport_keeps_list_and_close_distinct_and_rejects_foreign_ids() {
        let policy =
            NativeWindowPolicy::new("http://127.0.0.1:48291/", &WindowConfig::default()).unwrap();
        let (client, host) = window_channel(policy);
        for id in ["primary", "", "child-", "child-1/..", "child-💥"] {
            assert!(client.close(id).is_err());
        }
        let ticket = client.list().unwrap();
        let request = host.next().unwrap();
        assert!(matches!(&request.operation, WindowOperation::List));
        request.finish(Ok(serde_json::json!({"windowIds":["child-1"]})));
        assert_eq!(
            ticket.wait(Duration::from_secs(1)).unwrap(),
            serde_json::json!({"windowIds":["child-1"]})
        );
        let ticket = client.close("child-1").unwrap();
        let request = host.next().unwrap();
        assert!(matches!(&request.operation, WindowOperation::Close(id) if id=="child-1"));
        request.finish(Err("Unknown managed window ID".into()));
        assert!(ticket.wait(Duration::from_secs(1)).is_err());
    }
    #[test]
    fn focus_requests_target_only_managed_ids_and_do_not_confirm_os_state() {
        let policy =
            NativeWindowPolicy::new("http://127.0.0.1:48291/", &WindowConfig::default()).unwrap();
        let (client, host) = window_channel(policy);
        assert!(client.focus("primary").is_err());
        let ticket = client.focus("child-1").unwrap();
        let request = host.next().unwrap();
        assert!(matches!(&request.operation,WindowOperation::Focus(id) if id=="child-1"));
        request.finish(Ok(serde_json::json!({"windowId":"child-1","action":"focus","requested":true,"stateConfirmed":false})));
        let result = ticket.wait(Duration::from_secs(1)).unwrap();
        assert_eq!(result["stateConfirmed"], false);
        assert_eq!(result["action"], "focus");
    }

    #[test]
    fn minimize_and_restore_requests_preserve_the_requested_boolean() {
        let policy =
            NativeWindowPolicy::new("http://127.0.0.1:48291/", &WindowConfig::default()).unwrap();
        let (client, host) = window_channel(policy);
        assert!(client.set_minimized("primary", true).is_err());
        for minimized in [true, false] {
            let ticket = client.set_minimized("child-1", minimized).unwrap();
            let request = host.next().unwrap();
            assert!(
                matches!(&request.operation,WindowOperation::SetMinimized{id,minimized:flag} if id=="child-1" && *flag==minimized)
            );
            request.finish(Ok(serde_json::json!({"windowId":"child-1","action":"setMinimized","requested":true,"requestedMinimized":minimized,"stateConfirmed":false})));
            let result = ticket.wait(Duration::from_secs(1)).unwrap();
            assert_eq!(result["requestedMinimized"], minimized);
            assert_eq!(result["stateConfirmed"], false);
        }
    }
}
