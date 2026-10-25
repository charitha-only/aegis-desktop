use crate::window::{NativeWindowPolicy, WindowConfig};

fn smoke_enabled() -> bool {
    std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true")
        && std::env::var("RUNNER_ENVIRONMENT").as_deref() == Ok("github-hosted")
        && std::env::var("AEGIS_NATIVE_SMOKE").as_deref() == Ok("1")
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn run(
    url: &str,
    config: &WindowConfig,
    data_root: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
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
    }
    impl ApplicationHandler<bool> for App {
        fn user_event(&mut self, event_loop: &ActiveEventLoop, ready: bool) {
            if ready {
                self.ready = true;
                println!("AEGIS_NATIVE_SMOKE_READY");
            } else {
                self.error = Some("Native fixture document did not load before timeout".into());
            }
            event_loop.exit();
        }
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.window.is_some() {
                return;
            }
            let mut build = || -> Result<(Window, wry::WebView), Box<dyn std::error::Error>> {
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
                let policy = self.policy.clone();
                let proxy = self.proxy.clone();
                let smoke = self.smoke;
                let webview = wry::WebViewBuilder::new_with_web_context(&mut self.web_context)
                    .with_url(&self.url)
                    .with_incognito(true)
                    .with_devtools(false)
                    .with_document_title_changed_handler(move |title| {
                        if smoke && title == "Aegis Native Smoke Fixture" {
                            let _ = proxy.send_event(true);
                        }
                    })
                    .with_navigation_handler(move |target| policy.allows_navigation(&target))
                    .with_new_window_req_handler(|_, _| wry::NewWindowResponse::Deny)
                    .with_permission_handler(|_| wry::PermissionResponse::Deny)
                    .with_download_started_handler(|_, _| false)
                    .build(&window)?;
                window.set_visible(true);
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
        fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
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
    let smoke = smoke_enabled();
    if smoke {
        let proxy = event_loop.create_proxy();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(20));
            let _ = proxy.send_event(false);
        });
    }
    let mut app = App {
        url: url.into(),
        config: config.clone(),
        policy,
        window: None,
        webview: None,
        web_context: wry::WebContext::new(Some(webview_data)),
        error: None,
        proxy: event_loop.create_proxy(),
        smoke,
        ready: false,
    };
    event_loop.run_app(&mut app)?;
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
) -> Result<(), Box<dyn std::error::Error>> {
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
            view.connect_load_changed(move |view, event| {
                if event == webkit6::LoadEvent::Finished
                    && view.title().as_deref() == Some("Aegis Native Smoke Fixture")
                {
                    ready.set(true);
                    println!("AEGIS_NATIVE_SMOKE_READY");
                    if let Some(app) = app.upgrade() {
                        app.quit();
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
        window.present();
        crate::window::WindowManager::mark_native_active();
    });
    if smoke {
        let app = app.downgrade();
        gtk::glib::timeout_add_local_once(std::time::Duration::from_secs(20), move || {
            if let Some(app) = app.upgrade() {
                app.quit();
            }
        });
    }
    let result = app.run_with_args::<&str>(&[]);
    if result != gtk::glib::ExitCode::SUCCESS {
        return Err("Native GTK application failed".into());
    }
    if smoke && !ready.get() {
        return Err("Native fixture document did not load before timeout".into());
    }
    Ok(())
}
