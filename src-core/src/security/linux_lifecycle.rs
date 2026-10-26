//! Optional, revoke-only systemd-logind observer; never enables retained sessions.
use super::{lifecycle::LifecycleObserver, vault_session::VaultSessions};
use async_io::Timer;
use futures_lite::{FutureExt, StreamExt};
use std::{sync::Weak, time::Duration};
use zbus::{fdo::DBusProxy, message::Type, names::BusName, Connection, MatchRule, MessageStream};

const NAME: &str = "org.freedesktop.login1";
const ROOT: &str = "/org/freedesktop/login1";
const DEADLINE: Duration = Duration::from_secs(2);

pub fn start(sessions: Weak<VaultSessions>) {
    std::thread::spawn(move || {
        zbus::block_on(async move {
            // Do not trust DBUS_SYSTEM_BUS_ADDRESS or a user session bus as OS authority.
            let connect = async {
                zbus::connection::Builder::address("unix:path=/run/dbus/system_bus_socket")
                    .method_timeout(DEADLINE)
                    .build()
                    .await
            };
            let connection = connect
                .or(async {
                    Timer::after(DEADLINE).await;
                    Err(zbus::Error::Failure(
                        "System bus connection timed out".into(),
                    ))
                })
                .await;
            let observer = LifecycleObserver::new(sessions);
            if let Ok(connection) = connection {
                if let Ok(monitor) = LogindObserver::attach(connection, observer, 0).await {
                    let _ = monitor.run().await;
                }
            }
            // Drop also revokes on connection/registration failure or task termination.
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{secrets::SecretManager, security::VaultEngine};
    use std::sync::Arc;

    #[test]
    #[ignore = "requires an explicitly owned private D-Bus on hosted Linux CI"]
    fn private_logind_signals_revoke_and_reject_wrong_authority() {
        assert_eq!(std::env::var("GITHUB_ACTIONS").as_deref(), Ok("true"));
        assert_eq!(
            std::env::var("RUNNER_ENVIRONMENT").as_deref(),
            Ok("github-hosted")
        );
        assert_eq!(
            std::env::var("AEGIS_PRIVATE_LOGIND_BUS").as_deref(),
            Ok("1")
        );
        zbus::block_on(async {
            let service = Connection::session().await.unwrap();
            service.request_name(NAME).await.unwrap();
            let dbus = DBusProxy::new(&service).await.unwrap();
            let uid = dbus
                .get_connection_unix_user(service.unique_name().unwrap().clone().into())
                .await
                .unwrap();
            let secrets = Arc::new(SecretManager::new());
            let sessions = Arc::new(VaultSessions::new(secrets.clone()));
            let connect = || async {
                zbus::connection::Builder::session()
                    .method_timeout(DEADLINE)
                    .build()
                    .await
                    .unwrap()
            };
            assert!(LogindObserver::attach(
                connect().await,
                LifecycleObserver::new(Arc::downgrade(&sessions)),
                uid.checked_add(1).unwrap()
            )
            .await
            .is_err());
            let monitor = LogindObserver::attach(
                connect().await,
                LifecycleObserver::new(Arc::downgrade(&sessions)),
                uid,
            )
            .await
            .unwrap();
            let (done_tx, done_rx) = std::sync::mpsc::channel();
            let worker = std::thread::spawn(move || {
                let result = zbus::block_on(monitor.run());
                done_tx.send(result).unwrap();
            });
            let envelope = VaultEngine::encrypt("PRIVATE", "platform-test-password").unwrap();
            let token = sessions
                .unlock("one.vault", &envelope, "platform-test-password")
                .unwrap();
            let imposter = Connection::session().await.unwrap();
            imposter
                .emit_signal(
                    None::<&str>,
                    ROOT,
                    "org.freedesktop.login1.Manager",
                    "PrepareForSleep",
                    &true,
                )
                .await
                .unwrap();
            Timer::after(Duration::from_millis(150)).await;
            assert!(
                sessions.read(&token, "one.vault", &envelope).is_ok(),
                "Untrusted signal sender must not be accepted as logind"
            );
            for (member, value) in [
                ("PrepareForSleep", true),
                ("PrepareForSleep", false),
                ("PrepareForShutdown", true),
            ] {
                let token = sessions
                    .unlock("one.vault", &envelope, "platform-test-password")
                    .unwrap();
                secrets
                    .store_secret("temporary", "private", "platform-test-password")
                    .unwrap();
                service
                    .emit_signal(
                        None::<&str>,
                        ROOT,
                        "org.freedesktop.login1.Manager",
                        member,
                        &value,
                    )
                    .await
                    .unwrap();
                wait_for_clear(&secrets).await;
                assert!(sessions.read(&token, "one.vault", &envelope).is_err());
            }
            for member in ["Lock", "Unlock"] {
                let token = sessions
                    .unlock("one.vault", &envelope, "platform-test-password")
                    .unwrap();
                secrets
                    .store_secret("temporary", "private", "platform-test-password")
                    .unwrap();
                service
                    .emit_signal(
                        None::<&str>,
                        "/org/freedesktop/login1/session/test",
                        "org.freedesktop.login1.Session",
                        member,
                        &(),
                    )
                    .await
                    .unwrap();
                wait_for_clear(&secrets).await;
                assert!(sessions.read(&token, "one.vault", &envelope).is_err());
            }
            let token = sessions
                .unlock("one.vault", &envelope, "platform-test-password")
                .unwrap();
            secrets
                .store_secret("temporary", "private", "platform-test-password")
                .unwrap();
            service.release_name(NAME).await.unwrap();
            assert!(done_rx
                .recv_timeout(Duration::from_secs(5))
                .unwrap()
                .is_err());
            worker.join().unwrap();
            assert!(secrets.list_secrets().is_empty());
            assert!(sessions.read(&token, "one.vault", &envelope).is_err());
            assert!(!sessions.platform_monitor_active());
        });
    }

    async fn wait_for_clear(secrets: &SecretManager) {
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while !secrets.list_secrets().is_empty() && std::time::Instant::now() < deadline {
            Timer::after(Duration::from_millis(10)).await;
        }
        assert!(
            secrets.list_secrets().is_empty(),
            "Authenticated lifecycle event must clear secrets"
        );
    }
}

/// Observe a D-Bus connection at the lifecycle seam. The expected service UID is
/// explicit for embedding/tests; the production adapter always requires UID 0.
/// Neither successful registration nor any message can grant vault access.
pub struct LogindObserver {
    connection: Connection,
    observer: LifecycleObserver,
    owner: zbus::names::OwnedUniqueName,
    messages: MessageStream,
}

impl LogindObserver {
    pub async fn attach(
        connection: Connection,
        observer: LifecycleObserver,
        required_uid: u32,
    ) -> zbus::Result<Self> {
        observer.revoke();
        let bus = DBusProxy::new(&connection).await?;
        let name = BusName::try_from(NAME)?;
        let owner = bus.get_name_owner(name.clone()).await?;
        if bus.get_connection_unix_user(owner.clone().into()).await? != required_uid {
            return Err(zbus::Error::Failure(
                "Unexpected logind service owner".into(),
            ));
        }
        let rule = MatchRule::builder()
            .msg_type(Type::Signal)
            .sender(owner.clone())
            .path_namespace(ROOT)
            .build()?;
        let messages =
            MessageStream::for_match_rule(rule, &connection, std::num::NonZeroUsize::new(32))
                .await?;
        // Subscribe before rechecking ownership, closing registration races.
        if bus.get_name_owner(name.clone()).await? != owner {
            return Err(zbus::Error::Failure("Logind service changed".into()));
        }
        Ok(Self {
            connection,
            observer,
            owner,
            messages,
        })
    }

    pub async fn run(mut self) -> zbus::Result<()> {
        let bus = DBusProxy::new(&self.connection).await?;
        let name = BusName::try_from(NAME)?;
        while self.observer.is_needed() {
            enum Wake {
                Message(Option<zbus::Result<zbus::Message>>),
                Tick,
            }
            let wake = async { Wake::Message(self.messages.next().await) }
                .or(async {
                    Timer::after(Duration::from_secs(1)).await;
                    Wake::Tick
                })
                .await;
            match wake {
                Wake::Message(Some(Ok(_))) => {
                    // Conservatively revoke on all authenticated logind signals:
                    // lock/unlock, sleep/resume, shutdown, session/property changes.
                    self.observer.revoke();
                }
                Wake::Message(Some(Err(error))) => return Err(error),
                Wake::Message(None) => {
                    return Err(zbus::Error::Failure("System bus disconnected".into()))
                }
                Wake::Tick => {}
            }
            // A pinned unique owner prevents replacement services from being trusted.
            if bus.get_name_owner(name.clone()).await? != self.owner {
                return Err(zbus::Error::Failure("Logind service changed".into()));
            }
        }
        Ok(())
    }
}
