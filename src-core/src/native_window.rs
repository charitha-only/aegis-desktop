struct NativeActivity;
impl Drop for NativeActivity {
    fn drop(&mut self) {
        crate::window::WindowManager::mark_native_inactive();
    }
}
use crate::window::{NativeWindowPolicy, WindowConfig};

fn smoke_enabled() -> bool {
    std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true")
        && std::env::var("RUNNER_ENVIRONMENT").as_deref() == Ok("github-hosted")
        && std::env::var("AEGIS_NATIVE_SMOKE").as_deref() == Ok("1")
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn smoke_stage(enabled: bool, started: std::time::Instant, stage: &str) {
    if enabled {
        eprintln!(
            "AEGIS_NATIVE_SMOKE_STAGE stage={stage} elapsed_ms={}",
            started.elapsed().as_millis()
        );
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn run(
    url: &str,
    config: &WindowConfig,
    data_root: &std::path::Path,
    host: Option<crate::window_requests::WindowHost>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _activity = NativeActivity;
    let smoke = smoke_enabled();
    let started = std::time::Instant::now();
    smoke_stage(smoke, started, "host_started");
    use winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
        window::{Window, WindowId},
    };
    struct App {
        url: String,
        config: WindowConfig,
        policy: NativeWindowPolicy,
        window: Option<Window>,
        webview: Option<wry::WebView>,
        web_context: wry::WebContext,
        error: Option<String>,
        proxy: winit::event_loop::EventLoopProxy<bool>,
        smoke: bool,
        ready: bool,
        started: std::time::Instant,
        host: Option<crate::window_requests::WindowHost>,
        children: std::collections::BTreeMap<String, (wry::WebView, Window)>,
        next_child: u64,
    }
    impl ApplicationHandler<bool> for App {
        fn user_event(&mut self, event_loop: &ActiveEventLoop, ready: bool) {
            if ready {
                self.ready = true;
                smoke_stage(self.smoke, self.started, "fixture_ready");
                println!("AEGIS_NATIVE_SMOKE_READY");
            } else {
                smoke_stage(self.smoke, self.started, "deadline");
                self.error = Some("Native fixture document did not load before timeout".into());
            }
            event_loop.exit();
        }
        fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
            use crate::window_requests::WindowOperation;
            while let Some(request) = self.host.as_ref().and_then(|host| host.next()) {
                let result = match &request.operation {
                    WindowOperation::List => Ok(
                        serde_json::json!({"windowIds":self.children.keys().collect::<Vec<_>>() }),
                    ),
                    WindowOperation::SetMinimized { id, minimized } => {
                        match self.children.get(id) {
                            Some((_, window)) => {
                                window.set_minimized(*minimized);
                                Ok(
                                    serde_json::json!({"windowId":id,"action":"setMinimized","requested":true,"requestedMinimized":minimized,"stateConfirmed":false}),
                                )
                            }
                            None => Err("Unknown managed window ID".into()),
                        }
                    }
                    WindowOperation::Focus(id) => match self.children.get(id) {
                        Some((_, window)) => {
                            window.focus_window();
                            Ok(
                                serde_json::json!({"windowId":id,"action":"focus","requested":true,"stateConfirmed":false}),
                            )
                        }
                        None => Err("Unknown managed window ID".into()),
                    },
                    WindowOperation::Close(id) => match self.children.remove(id) {
                        Some((view, window)) => {
                            drop(view);
                            drop(window);
                            Ok(serde_json::json!({"closed":true,"windowId":id}))
                        }
                        None => Err("Unknown managed window ID".into()),
                    },
                    WindowOperation::Create { url, config } => {
                        (|| -> Result<serde_json::Value, String> {
                            if self.children.len() >= 8 {
                                return Err("Managed child window limit reached".into());
                            }
                            let next = self
                                .next_child
                                .checked_add(1)
                                .ok_or("Managed window ID exhausted")?;
                            let policy = NativeWindowPolicy::new(url, config)?;
                            if !self.policy.allows_navigation(url) {
                                return Err("Child origin rejected".into());
                            }
                            let window = event_loop
                                .create_window(
                                    Window::default_attributes()
                                        .with_title(&config.title)
                                        .with_inner_size(winit::dpi::LogicalSize::new(
                                            config.width,
                                            config.height,
                                        ))
                                        .with_resizable(config.resizable)
                                        .with_decorations(config.decorations)
                                        .with_visible(false),
                                )
                                .map_err(|e| e.to_string())?;
                            let view =
                                wry::WebViewBuilder::new_with_web_context(&mut self.web_context)
                                    .with_url(url)
                                    .with_incognito(true)
                                    .with_devtools(false)
                                    .with_navigation_handler(move |target| {
                                        policy.allows_navigation(&target)
                                    })
                                    .with_new_window_req_handler(|_, _| {
                                        wry::NewWindowResponse::Deny
                                    })
                                    .with_permission_handler(|_| wry::PermissionResponse::Deny)
                                    .with_download_started_handler(|_, _| false)
                                    .build(&window)
                                    .map_err(|e| e.to_string())?;
                            let id = format!("child-{}", self.next_child);
                            self.next_child = next;
                            window.set_visible(true);
                            self.children.insert(id.clone(), (view, window));
                            Ok(
                                serde_json::json!({"windowId":id,"backend":"NATIVE_EMBEDDED","documentReady":false}),
                            )
                        })()
                    }
                };
                request.finish(result);
            }
            event_loop.set_control_flow(if self.host.is_some() {
                ControlFlow::WaitUntil(
                    std::time::Instant::now() + std::time::Duration::from_millis(50),
                )
            } else {
                ControlFlow::Wait
            });
        }
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.window.is_some() {
                return;
            }
            smoke_stage(self.smoke, self.started, "resumed");
            let mut build = || -> Result<(Window, wry::WebView), Box<dyn std::error::Error>> {
                smoke_stage(self.smoke, self.started, "primary_window_build");
                let window = event_loop.create_window(
                    Window::default_attributes()
                        .with_title(&self.config.title)
                        .with_inner_size(winit::dpi::LogicalSize::new(
                            self.config.width,
                            self.config.height,
                        ))
                        .with_resizable(self.config.resizable)
                        .with_decorations(self.config.decorations)
                        .with_visible(false),
                )?;
                smoke_stage(self.smoke, self.started, "primary_window_created");
                let policy = self.policy.clone();
                let started = self.started;
                let proxy = self.proxy.clone();
                let smoke = self.smoke;
                smoke_stage(smoke, started, "primary_webview_build");
                let webview = wry::WebViewBuilder::new_with_web_context(&mut self.web_context)
                    .with_url(&self.url)
                    .with_incognito(true)
                    .with_devtools(false)
                    .with_on_page_load_handler(move |event, _| {
                        let stage = match event {
                            wry::PageLoadEvent::Started => "primary_load_started",
                            wry::PageLoadEvent::Finished => "primary_load_finished",
                        };
                        smoke_stage(smoke, started, stage);
                    })
                    .with_document_title_changed_handler(move |title| {
                        if smoke {
                            eprintln!("NATIVE_SMOKE_TITLE: {title}");
                        }
                        if smoke && title == "Aegis Native Smoke Fixture" {
                            let _ = proxy.send_event(true);
                        }
                    })
                    .with_navigation_handler(move |target| policy.allows_navigation(&target))
                    .with_new_window_req_handler(|_, _| wry::NewWindowResponse::Deny)
                    .with_permission_handler(|_| wry::PermissionResponse::Deny)
                    .with_download_started_handler(|_, _| false)
                    .build(&window)?;
                smoke_stage(smoke, started, "primary_webview_created");
                window.set_visible(true);
                smoke_stage(smoke, started, "primary_visible");
                crate::window::WindowManager::mark_native_active();
                Ok((window, webview))
            };
            match build() {
                Ok((window, webview)) => {
                    self.window = Some(window);
                    self.webview = Some(webview);
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                    event_loop.exit();
                }
            }
        }
        fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
            if self.window.as_ref().is_none_or(|window| window.id() != id) {
                if let Some(key) = self
                    .children
                    .iter()
                    .find(|(_, (_, window))| window.id() == id)
                    .map(|(key, _)| key.clone())
                {
                    match event {
                        WindowEvent::CloseRequested => {
                            if let Some((view, window)) = self.children.remove(&key) {
                                drop(view);
                                drop(window);
                            }
                        }
                        WindowEvent::Resized(size) => {
                            if let Some((view, _)) = self.children.get(&key) {
                                if let Err(error) = view.set_bounds(wry::Rect {
                                    position: wry::dpi::PhysicalPosition::new(0, 0).into(),
                                    size: wry::dpi::PhysicalSize::new(size.width, size.height)
                                        .into(),
                                }) {
                                    eprintln!("Child resize failed: {error}");
                                    if let Some((view, window)) = self.children.remove(&key) {
                                        drop(view);
                                        drop(window);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                return;
            }
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(size) => {
                    if let Some(view) = &self.webview {
                        if let Err(error) = view.set_bounds(wry::Rect {
                            position: wry::dpi::PhysicalPosition::new(0, 0).into(),
                            size: wry::dpi::PhysicalSize::new(size.width, size.height).into(),
                        }) {
                            self.error = Some(error.to_string());
                            event_loop.exit();
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let policy = NativeWindowPolicy::new(url, config)?;
    let webview_data = data_root.join("WebViewCache");
    std::fs::create_dir_all(&webview_data)?;
    let event_loop = EventLoop::<bool>::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    smoke_stage(smoke, started, "event_loop_created");
    if smoke {
        let proxy = event_loop.create_proxy();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(45));
            let _ = proxy.send_event(false);
        });
    }
    let web_context = wry::WebContext::new(Some(webview_data));
    smoke_stage(smoke, started, "web_context_created");
    let mut app = App {
        url: url.into(),
        config: config.clone(),
        policy,
        window: None,
        webview: None,
        web_context,
        error: None,
        proxy: event_loop.create_proxy(),
        smoke,
        ready: false,
        started,
        host,
        children: std::collections::BTreeMap::new(),
        next_child: 1,
    };
    smoke_stage(smoke, started, "event_loop_running");
    event_loop.run_app(&mut app)?;
    let retained_children = app.children.len();
    app.children.clear();
    if smoke {
        println!("AEGIS_NATIVE_SMOKE_CHILDREN_DROPPED={retained_children}");
    }
    drop(app.webview.take());
    drop(app.window.take());
    if let Some(error) = app.error {
        return Err(error.into());
    }
    if smoke && !app.ready {
        return Err("Native window closed before fixture readiness".into());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn run(
    url: &str,
    config: &WindowConfig,
    _data_root: &std::path::Path,
    host: Option<crate::window_requests::WindowHost>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _activity = NativeActivity;
    use gtk::prelude::*;
    use webkit6::{gtk, prelude::*};
    let policy = NativeWindowPolicy::new(url, config)?;
    let url = url.to_owned();
    let config = config.clone();
    let smoke = smoke_enabled();
    let ready = std::rc::Rc::new(std::cell::Cell::new(false));
    let loaded = ready.clone();
    let app = gtk::Application::builder()
        .application_id("org.aegis.desktop")
        .flags(gtk::gio::ApplicationFlags::NON_UNIQUE)
        .build();
    let children = std::rc::Rc::new(std::cell::RefCell::new(std::collections::BTreeMap::<
        String,
        (gtk::ApplicationWindow, webkit6::WebView),
    >::new()));
    let child_map = children.clone();
    let child_policy = policy.clone();
    let child_app = app.downgrade();
    let mut next_child = 1u64;
    let source = host.map(|host| gtk::glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        use crate::window_requests::WindowOperation;
        let Some(app) = child_app.upgrade() else {
            return gtk::glib::ControlFlow::Break;
        };
        while let Some(request) = host.next() {
            let result = match &request.operation {
                WindowOperation::List => Ok(
                    serde_json::json!({"windowIds":child_map.borrow().keys().collect::<Vec<_>>() }),
                ),
                    WindowOperation::SetMinimized {id,minimized} => match child_map.borrow().get(id) {
                    Some((window,_)) => {if *minimized {window.minimize();}else{window.unminimize();}Ok(serde_json::json!({"windowId":id,"action":"setMinimized","requested":true,"requestedMinimized":minimized,"stateConfirmed":false}))},
                    None => Err("Unknown managed window ID".into()),
                },
                WindowOperation::Focus(id) => match child_map.borrow().get(id) {
                    Some((window,_)) => {window.present();Ok(serde_json::json!({"windowId":id,"action":"focus","requested":true,"stateConfirmed":false}))},
                    None => Err("Unknown managed window ID".into()),
                },
                WindowOperation::Close(id) => {
                    let entry = child_map.borrow_mut().remove(id);
                    match entry {
                        Some((window, _)) => {
                            window.close();
                            Ok(serde_json::json!({"closed":true,"windowId":id}))
                        }
                        None => Err("Unknown managed window ID".into()),
                    }
                }
                WindowOperation::Create { url, config } => {
                    (|| -> Result<serde_json::Value, String> {
                        if child_map.borrow().len() >= 8 {
                            return Err("Managed child window limit reached".into());
                        }
                        let next = next_child
                            .checked_add(1)
                            .ok_or("Managed window ID exhausted")?;
                        if !child_policy.allows_navigation(url) {
                            return Err("Child origin rejected".into());
                        }
                        let policy = NativeWindowPolicy::new(url, config)?;
                        let session = webkit6::NetworkSession::new_ephemeral();
                        session.connect_download_started(|_, download| download.cancel());
                        let view = webkit6::WebView::builder()
                            .network_session(&session)
                            .build();
                        view.set_settings(
                            &webkit6::Settings::builder()
                                .enable_developer_extras(false)
                                .build(),
                        );
                        view.connect_permission_request(|_, request| {
                            request.deny();
                            true
                        });
                        view.connect_create(|_, _| None);
                        view.connect_run_file_chooser(|_, request| {
                            request.cancel();
                            true
                        });
                        view.connect_decide_policy(move |_, decision, kind| {
                            if kind == webkit6::PolicyDecisionType::NavigationAction {
                                let allowed = decision
                                    .downcast_ref::<webkit6::NavigationPolicyDecision>()
                                    .and_then(|d| d.navigation_action())
                                    .and_then(|a| a.request())
                                    .and_then(|r| r.uri())
                                    .is_some_and(|uri| policy.allows_navigation(&uri));
                                if allowed {
                                    decision.use_();
                                } else {
                                    decision.ignore();
                                }
                                true
                            } else if kind == webkit6::PolicyDecisionType::NewWindowAction {
                                decision.ignore();
                                true
                            } else {
                                false
                            }
                        });
                        let window = gtk::ApplicationWindow::builder()
                            .application(&app)
                            .title(&config.title)
                            .default_width(config.width as i32)
                            .default_height(config.height as i32)
                            .resizable(config.resizable)
                            .decorated(config.decorations)
                            .child(&view)
                            .build();
                        let id = format!("child-{next_child}");
                        next_child = next;
                        let weak = std::rc::Rc::downgrade(&child_map);
                        let closed = id.clone();
                        window.connect_close_request(move |_| {
                            if let Some(map) = weak.upgrade() {
                                map.borrow_mut().remove(&closed);
                            }
                            gtk::glib::Propagation::Proceed
                        });
                        view.load_uri(url);
                        window.present();
                        child_map.borrow_mut().insert(id.clone(), (window, view));
                        Ok(
                            serde_json::json!({"windowId":id,"backend":"NATIVE_EMBEDDED","documentReady":false}),
                        )
                    })()
                }
            };
            request.finish(result);
        }
        gtk::glib::ControlFlow::Continue
    }));
    app.connect_activate(move |app| {
        let session = webkit6::NetworkSession::new_ephemeral();
        session.connect_download_started(|_, download| download.cancel());
        let view = webkit6::WebView::builder()
            .network_session(&session)
            .build();
        let settings = webkit6::Settings::builder()
            .enable_developer_extras(false)
            .build();
        view.set_settings(&settings);
        if smoke {
            let app = app.downgrade();
            let ready = loaded.clone();
            let finished = std::rc::Rc::new(std::cell::Cell::new(false));
            let title_finished = finished.clone();
            let title_app = app.clone();
            let title_ready = ready.clone();
            view.connect_title_notify(move |view| {
                eprintln!("NATIVE_SMOKE_TITLE: {:?}", view.title());
                if title_finished.get()
                    && view.title().as_deref() == Some("Aegis Native Smoke Fixture")
                {
                    title_ready.set(true);
                    println!("AEGIS_NATIVE_SMOKE_READY");
                    if let Some(app) = title_app.upgrade() {
                        app.quit();
                    }
                }
            });
            view.connect_load_failed(|_, _, _, error| {
                eprintln!("NATIVE_SMOKE_LOAD_FAILED: {error}");
                false
            });
            view.connect_web_process_terminated(|_, reason| {
                eprintln!("NATIVE_SMOKE_WEB_PROCESS_TERMINATED: {reason:?}");
            });
            view.connect_load_changed(move |view, event| {
                eprintln!("NATIVE_SMOKE_LOAD: {event:?}; title={:?}", view.title());
                if event == webkit6::LoadEvent::Finished {
                    finished.set(true);
                    if view.title().as_deref() == Some("Aegis Native Smoke Fixture") {
                        ready.set(true);
                        println!("AEGIS_NATIVE_SMOKE_READY");
                        if let Some(app) = app.upgrade() {
                            app.quit();
                        }
                    }
                }
            });
        }
        view.connect_permission_request(|_, request| {
            request.deny();
            true
        });
        view.connect_create(|_, _| None);
        view.connect_run_file_chooser(|_, request| {
            request.cancel();
            true
        });
        let navigation = policy.clone();
        view.connect_decide_policy(move |_, decision, kind| {
            if kind == webkit6::PolicyDecisionType::NavigationAction {
                let allowed = decision
                    .downcast_ref::<webkit6::NavigationPolicyDecision>()
                    .and_then(|d| d.navigation_action())
                    .and_then(|a| a.request())
                    .and_then(|r| r.uri())
                    .is_some_and(|uri| navigation.allows_navigation(&uri));
                if allowed {
                    decision.use_();
                } else {
                    decision.ignore();
                }
                true
            } else if kind == webkit6::PolicyDecisionType::NewWindowAction {
                decision.ignore();
                true
            } else {
                false
            }
        });
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title(&config.title)
            .default_width(config.width as i32)
            .default_height(config.height as i32)
            .resizable(config.resizable)
            .decorated(config.decorations)
            .child(&view)
            .build();
        view.load_uri(&url);
        let quit_app = app.downgrade();
        window.connect_close_request(move |_| {
            if let Some(app) = quit_app.upgrade() {
                app.quit();
            }
            gtk::glib::Propagation::Proceed
        });
        window.present();
        crate::window::WindowManager::mark_native_active();
    });
    if smoke {
        let app = app.downgrade();
        gtk::glib::timeout_add_local_once(std::time::Duration::from_secs(45), move || {
            if let Some(app) = app.upgrade() {
                app.quit();
            }
        });
    }
    let result = app.run_with_args::<&str>(&[]);
    if let Some(source) = source {
        source.remove();
    }
    let retained = std::mem::take(&mut *children.borrow_mut());
    let retained_children = retained.len();
    for (_, (window, _)) in retained {
        window.close();
    }
    if smoke {
        println!("AEGIS_NATIVE_SMOKE_CHILDREN_DROPPED={retained_children}");
    }
    if result != gtk::glib::ExitCode::SUCCESS {
        return Err("Native GTK application failed".into());
    }
    if smoke && !ready.get() {
        return Err("Native fixture document did not load before timeout".into());
    }
    Ok(())
}
