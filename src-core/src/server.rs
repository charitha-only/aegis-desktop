use crate::ipc::{IpcRequest, IpcRouter};
use crate::security::{HandshakeSession, OriginValidator, RateLimiter};
use serde_json::json;
use std::sync::Arc;
use tiny_http::{Header, Method, Response, Server, StatusCode};

pub struct CoreServer {
    server: Server,
    router: Arc<IpcRouter>,
    session: Arc<HandshakeSession>,
    validator: Arc<OriginValidator>,
    rate_limiter: Arc<RateLimiter>,
    csp: String,
    ui_root: Option<std::path::PathBuf>,
}

impl CoreServer {
    pub fn bind(
        addr: &str,
        router: Arc<IpcRouter>,
        session: Arc<HandshakeSession>,
        validator: Arc<OriginValidator>,
        csp: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let server = Server::http(addr).map_err(|e| format!("HTTP Server Bind Error: {}", e))?;
        // 30 requests per 1-second sliding window, 10s lockout after 5 consecutive failed nonces
        let rate_limiter = Arc::new(RateLimiter::new(30, 1, 10));

        Ok(Self {
            server,
            router,
            session,
            validator,
            rate_limiter,
            csp,
            ui_root: std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("ui")))
                .filter(|p| p.join("index.html").is_file()),
        })
    }

    pub fn run(&self) {
        println!(
            "[Aegis Core] IPC Bridge Server actively listening on http://{}",
            self.server.server_addr()
        );
        println!("[Aegis Security] Anti-DoS Rate Limiter active: Max 30 req/s, brute-force lockout active.");

        for mut request in self.server.incoming_requests() {
            let host = request
                .headers()
                .iter()
                .find(|h| h.field.equiv("Host"))
                .map(|h| h.value.as_str())
                .unwrap_or("");
            if host != self.server.server_addr().to_string() {
                let _ = request.respond(Response::empty(StatusCode(403)));
                continue;
            }
            if *request.method() == Method::Get && request.url() != "/handshake" {
                if let Some(root) = &self.ui_root {
                    let route = request.url().split('?').next().unwrap_or("/");
                    let relative = if route == "/" {
                        "index.html"
                    } else {
                        route.trim_start_matches('/')
                    };
                    let path = root.join(relative);
                    let safe = !relative.contains('%')
                        && !relative.contains('\\')
                        && std::path::Path::new(relative)
                            .components()
                            .all(|c| matches!(c, std::path::Component::Normal(_)));
                    if safe {
                        if let (Ok(base), Ok(canon)) = (root.canonicalize(), path.canonicalize()) {
                            if canon.starts_with(base) && canon.is_file() {
                                if let Ok(bytes) = std::fs::read(&canon) {
                                    let mime = match canon
                                        .extension()
                                        .and_then(|e| e.to_str())
                                        .unwrap_or("")
                                    {
                                        "html" => "text/html; charset=utf-8",
                                        "js" => "text/javascript",
                                        "css" => "text/css",
                                        "svg" => "image/svg+xml",
                                        "png" => "image/png",
                                        "woff2" => "font/woff2",
                                        _ => "application/octet-stream",
                                    };
                                    let response = Response::from_data(bytes)
                                        .with_header(
                                            Header::from_bytes("Content-Type", mime).unwrap(),
                                        )
                                        .with_header(
                                            Header::from_bytes(
                                                "Content-Security-Policy",
                                                self.csp.as_bytes(),
                                            )
                                            .unwrap(),
                                        )
                                        .with_header(
                                            Header::from_bytes("X-Content-Type-Options", "nosniff")
                                                .unwrap(),
                                        );
                                    let _ = request.respond(response);
                                    continue;
                                }
                            }
                        }
                    }
                }
                let _ = request.respond(Response::empty(StatusCode(404)));
                continue;
            }
            let origin_header = request
                .headers()
                .iter()
                .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case("origin"))
                .map(|h| h.value.as_str().to_string());

            let is_allowed_origin = match &origin_header {
                Some(orig) => self.validator.is_allowed(Some(orig)),
                None => true, // direct non-browser native bridge or local test
            };

            let cors_origin = origin_header
                .clone()
                .unwrap_or_else(|| format!("http://{}", self.server.server_addr()));

            if !is_allowed_origin {
                let _ = request.respond(Response::empty(StatusCode(403)));
                continue;
            }

            // Handle Preflight OPTIONS
            if *request.method() == Method::Options {
                let response = Response::empty(StatusCode(204))
                    .with_header(
                        Header::from_bytes(
                            &b"Access-Control-Allow-Origin"[..],
                            cors_origin.as_bytes(),
                        )
                        .unwrap(),
                    )
                    .with_header(
                        Header::from_bytes(
                            &b"Access-Control-Allow-Methods"[..],
                            &b"GET, POST, OPTIONS"[..],
                        )
                        .unwrap(),
                    )
                    .with_header(
                        Header::from_bytes(
                            &b"Access-Control-Allow-Headers"[..],
                            &b"Content-Type, Authorization"[..],
                        )
                        .unwrap(),
                    );
                let _ = request.respond(response);
                continue;
            }

            // 1. Origin security enforcement
            if !is_allowed_origin {
                eprintln!(
                    "[Aegis Security] Blocked unauthorized origin: {:?}",
                    origin_header
                );
                let err_json = json!({ "error": "Blocked by Aegis Origin Policy" }).to_string();
                let resp = Response::from_string(err_json)
                    .with_status_code(StatusCode(403))
                    .with_header(
                        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                    );
                let _ = request.respond(resp);
                continue;
            }

            // 2. Sliding Window Anti-DoS & Brute Force Rate Limiting
            let origin_key = origin_header.as_deref().unwrap_or("local_loopback");
            if self.rate_limiter.is_locked_out(origin_key) {
                let err_json = json!({
                    "success": false,
                    "error": "Access Locked Out: Too many consecutive invalid nonces. Backoff penalty active!"
                }).to_string();
                let resp = Response::from_string(err_json)
                    .with_status_code(StatusCode(429))
                    .with_header(
                        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                    )
                    .with_header(
                        Header::from_bytes(
                            &b"Access-Control-Allow-Origin"[..],
                            cors_origin.as_bytes(),
                        )
                        .unwrap(),
                    );
                let _ = request.respond(resp);
                continue;
            }

            if let Err(rate_err) = self.rate_limiter.check_request() {
                let err_json = json!({
                    "success": false,
                    "error": rate_err
                })
                .to_string();
                let resp = Response::from_string(err_json)
                    .with_status_code(StatusCode(429))
                    .with_header(
                        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                    )
                    .with_header(
                        Header::from_bytes(
                            &b"Access-Control-Allow-Origin"[..],
                            cors_origin.as_bytes(),
                        )
                        .unwrap(),
                    );
                let _ = request.respond(resp);
                continue;
            }

            match (request.method(), request.url()) {
                (Method::Get, "/handshake") => {
                    let token = self.session.get_token();
                    let payload = json!({
                        "handshake": "successful",
                        "token": token,
                        "csp": self.csp,
                        "server": "Aegis Core v0.11 (Rust)",
                        "rateLimiter": "Active (30 req/s window)",
                        "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
                    })
                    .to_string();

                    let resp = sensitive_response(payload)
                        .with_header(Header::from_bytes("Cache-Control", "no-store").unwrap())
                        .with_header(
                            Header::from_bytes("X-Content-Type-Options", "nosniff").unwrap(),
                        )
                        .with_header(
                            Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                                .unwrap(),
                        )
                        .with_header(
                            Header::from_bytes(
                                &b"Access-Control-Allow-Origin"[..],
                                cors_origin.as_bytes(),
                            )
                            .unwrap(),
                        );
                    let _ = request.respond(resp);
                }

                (Method::Post, "/ipc") => {
                    let content_type = request
                        .headers()
                        .iter()
                        .find(|h| h.field.equiv("Content-Type"))
                        .map(|h| h.value.as_str().split(';').next().unwrap_or(""))
                        .unwrap_or("");
                    if content_type != "application/json" {
                        let _ = request.respond(Response::empty(StatusCode(415)));
                        continue;
                    }
                    if request.body_length().is_none_or(|n| n > 1024 * 1024) {
                        let _ = request.respond(Response::empty(StatusCode(413)));
                        continue;
                    }
                    let mut body = zeroize::Zeroizing::new(String::new());
                    use std::io::Read;
                    if request
                        .as_reader()
                        .take(1024 * 1024 + 1)
                        .read_to_string(&mut body)
                        .is_ok()
                        && body.len() <= 1024 * 1024
                    {
                        let ipc_res = match serde_json::from_str::<IpcRequest>(&body) {
                            Ok(req) => {
                                if !self.session.validate_nonce(&req.nonce) {
                                    self.rate_limiter.record_nonce_failure(origin_key);
                                }
                                self.router.handle_request(req)
                            }
                            Err(e) => crate::ipc::IpcResponse::err(
                                "0",
                                &format!("Malformed IPC JSON: {}", e),
                            ),
                        };

                        let res_json = serde_json::to_string(&ipc_res).unwrap();
                        let resp = sensitive_response(res_json)
                            .with_header(Header::from_bytes("Cache-Control", "no-store").unwrap())
                            .with_header(
                                Header::from_bytes("X-Content-Type-Options", "nosniff").unwrap(),
                            )
                            .with_header(
                                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                                    .unwrap(),
                            )
                            .with_header(
                                Header::from_bytes(
                                    &b"Access-Control-Allow-Origin"[..],
                                    cors_origin.as_bytes(),
                                )
                                .unwrap(),
                            );
                        let _ = request.respond(resp);
                    } else {
                        let resp = Response::from_string(
                            json!({ "error": "Could not read body" }).to_string(),
                        )
                        .with_status_code(StatusCode(400));
                        let _ = request.respond(resp);
                    }
                }

                _ => {
                    let resp = Response::from_string(json!({ "error": "Not Found" }).to_string())
                        .with_status_code(StatusCode(404));
                    let _ = request.respond(resp);
                }
            }
        }
    }
}

struct WipingBody {
    bytes: zeroize::Zeroizing<Vec<u8>>,
    position: usize,
}
impl std::io::Read for WipingBody {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = buf.len().min(self.bytes.len() - self.position);
        buf[..n].copy_from_slice(&self.bytes[self.position..self.position + n]);
        self.position += n;
        Ok(n)
    }
}
fn sensitive_response(text: String) -> Response<WipingBody> {
    let length = text.len();
    Response::new(
        StatusCode(200),
        vec![],
        WipingBody {
            bytes: zeroize::Zeroizing::new(text.into_bytes()),
            position: 0,
        },
        Some(length),
        None,
    )
}
