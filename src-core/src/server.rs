use crate::ipc::{IpcRequest, IpcRouter};
use crate::security::{HandshakeSession, OriginValidator};
use serde_json::json;
use std::sync::Arc;
use tiny_http::{Header, Method, Response, Server, StatusCode};

pub struct CoreServer {
    server: Server,
    router: Arc<IpcRouter>,
    session: Arc<HandshakeSession>,
    validator: Arc<OriginValidator>,
    csp: String,
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
        Ok(Self {
            server,
            router,
            session,
            validator,
            csp,
        })
    }

    pub fn run(&self) {
        println!("[Aegis Core] IPC Bridge Server actively listening on http://{}", self.server.server_addr());

        for mut request in self.server.incoming_requests() {
            let origin_header = request
                .headers()
                .iter()
                .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case("origin"))
                .map(|h| h.value.as_str().to_string());

            let is_allowed_origin = match &origin_header {
                Some(orig) => self.validator.is_allowed(Some(orig)),
                None => true, // direct non-browser native bridge or local tool
            };

            let cors_origin = origin_header.clone().unwrap_or_else(|| "*".to_string());

            // Handle Preflight OPTIONS
            if *request.method() == Method::Options {
                let response = Response::empty(StatusCode(204))
                    .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], cors_origin.as_bytes()).unwrap())
                    .with_header(Header::from_bytes(&b"Access-Control-Allow-Methods"[..], &b"GET, POST, OPTIONS"[..]).unwrap())
                    .with_header(Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"Content-Type, Authorization"[..]).unwrap());
                let _ = request.respond(response);
                continue;
            }

            // Origin security enforcement
            if !is_allowed_origin {
                eprintln!("[Aegis Security] Blocked unauthorized origin: {:?}", origin_header);
                let err_json = json!({ "error": "Blocked by Aegis Origin Policy" }).to_string();
                let resp = Response::from_string(err_json)
                    .with_status_code(StatusCode(403))
                    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
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
                        "server": "Aegis Core 1.0 (Rust)",
                        "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
                    })
                    .to_string();

                    let resp = Response::from_string(payload)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
                        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], cors_origin.as_bytes()).unwrap());
                    let _ = request.respond(resp);
                }

                (Method::Post, "/ipc") => {
                    let mut body = String::new();
                    if request.as_reader().read_to_string(&mut body).is_ok() {
                        let ipc_res = match serde_json::from_str::<IpcRequest>(&body) {
                            Ok(req) => self.router.handle_request(req),
                            Err(e) => crate::ipc::IpcResponse::err("0", &format!("Malformed IPC JSON: {}", e)),
                        };

                        let res_json = serde_json::to_string(&ipc_res).unwrap();
                        let resp = Response::from_string(res_json)
                            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
                            .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], cors_origin.as_bytes()).unwrap());
                        let _ = request.respond(resp);
                    } else {
                        let resp = Response::from_string(json!({ "error": "Could not read body" }).to_string())
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
