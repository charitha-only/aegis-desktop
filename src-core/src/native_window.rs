use crate::window::{NativeWindowPolicy, WindowConfig};

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn run(url: &str, config: &WindowConfig) -> Result<(), Box<dyn std::error::Error>> {
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
        error: Option<String>,
    }
    impl ApplicationHandler for App {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.window.is_some() {
                return;
            }
            let build = || -> Result<(Window, wry::WebView), Box<dyn std::error::Error>> {
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
                let webview = wry::WebViewBuilder::new()
                    .with_url(&self.url)
                    .with_incognito(true)
                    .with_devtools(false)
                    .with_navigation_handler(move |target| policy.allows_navigation(&target))
                    .with_new_window_req_handler(|_, _| wry::NewWindowResponse::Deny)
                    .with_permission_handler(|_| wry::PermissionResponse::Deny)
                    .with_download_started_handler(|_, _| false)
                    .build(&window)?;
                window.set_visible(true);
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
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App {
        url: url.into(),
        config: config.clone(),
        policy,
        window: None,
        webview: None,
        error: None,
    };
    event_loop.run_app(&mut app)?;
    if let Some(error) = app.error {
        return Err(error.into());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn run(url: &str, config: &WindowConfig) -> Result<(), Box<dyn std::error::Error>> {
    use gtk::prelude::*;
    use webkit6::{gtk, prelude::*};
    let policy = NativeWindowPolicy::new(url, config)?;
    let url = url.to_owned();
    let config = config.clone();
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
    });
    let result = app.run_with_args::<&str>(&[]);
    if result != gtk::glib::ExitCode::SUCCESS {
        return Err("Native GTK application failed".into());
    }
    Ok(())
}
