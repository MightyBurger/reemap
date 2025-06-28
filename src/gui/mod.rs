// The following code is from here:
// https://github.com/emilk/egui/blob/main/crates/egui_glow/examples/pure_glow.rs

mod glutin_ctx;
pub mod reemapp;

use glutin_ctx::GlutinWindowContext;

const TITLE: &str = "Reemap";
const SIZE: winit::dpi::LogicalSize<f64> = winit::dpi::LogicalSize {
    width: 800.0,
    height: 600.0,
};
const START_VISIBLE: bool = true;

use std::sync::Arc;
use tray_icon::TrayIcon;

#[derive(Debug)]
enum ReemapGuiEvent {
    Redraw(std::time::Duration),
    SetWindowVisibility(bool),
    TrayIconEvent(tray_icon::TrayIconEvent),
    TrayMenuEvent(tray_icon::menu::MenuEvent),
}

pub trait TrayApp {
    fn update(&mut self, ctx: &egui::Context);
}

struct GlowApp<T: TrayApp> {
    proxy: winit::event_loop::EventLoopProxy<ReemapGuiEvent>,
    gl_window: Option<GlutinWindowContext>,
    gl: Option<Arc<glow::Context>>,
    egui_glow: Option<egui_glow::EguiGlow>,
    repaint_delay: std::time::Duration,
    tray_icon: Option<TrayIcon>,
    app_data: T,
}

impl<T: TrayApp> GlowApp<T> {
    fn new(proxy: winit::event_loop::EventLoopProxy<ReemapGuiEvent>, app_data: T) -> Self {
        Self {
            proxy,
            gl_window: None,
            gl: None,
            egui_glow: None,
            repaint_delay: std::time::Duration::MAX,
            tray_icon: None,
            app_data,
        }
    }

    fn set_visible(&self, visible: bool, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(ref gl_window) = self.gl_window {
            gl_window.window().set_visible(visible);
            if visible {
                gl_window.window().request_redraw();
            }
        }
        if !visible {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        }
    }
}

impl<T: TrayApp> winit::application::ApplicationHandler<ReemapGuiEvent> for GlowApp<T> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let (gl_window, gl) = create_display(event_loop);
        let gl = std::sync::Arc::new(gl);
        gl_window.window().set_visible(START_VISIBLE);

        let egui_glow = egui_glow::EguiGlow::new(event_loop, gl.clone(), None, None, true);

        let event_loop_proxy = egui::mutex::Mutex::new(self.proxy.clone());
        egui_glow
            .egui_ctx
            .set_request_repaint_callback(move |info| {
                event_loop_proxy
                    .lock()
                    .send_event(ReemapGuiEvent::Redraw(info.delay))
                    .expect("Cannot send event")
            });
        self.gl_window = Some(gl_window);
        self.gl = Some(gl);
        self.egui_glow = Some(egui_glow);

        let tray_menu = {
            let menu = tray_icon::menu::Menu::new();
            let configure_btn =
                tray_icon::menu::MenuItem::with_id("MENU_ID_CONFIGURE", "Configure", true, None);
            let exit_btn = tray_icon::menu::MenuItem::with_id("MENU_ID_EXIT", "Exit", true, None);
            menu.append_items(&[&configure_btn, &exit_btn])
                .expect("could not initialize tray menu");

            menu
        };

        // note: creating this has the side effect of creating the tray icon
        let tray_icon = {
            let path = concat!(env!("CARGO_MANIFEST_DIR"), "/resource/icon.png");
            let icon = load_icon(std::path::Path::new(path));

            tray_icon::TrayIconBuilder::new()
                .with_menu(Box::new(tray_menu))
                .with_tooltip("tooltip test")
                .with_icon(icon)
                .with_title("title")
                .build()
                .unwrap()
        };

        self.tray_icon = Some(tray_icon);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        use winit::event::WindowEvent;

        // If the user closes the window, continue running in the background.
        if matches!(event, WindowEvent::CloseRequested) {
            self.set_visible(false, event_loop);
            return;
        }

        // Possibly unnecessary?
        if matches!(event, WindowEvent::Destroyed) {
            event_loop.exit();
            return;
        }

        if matches!(event, WindowEvent::RedrawRequested) {
            self.egui_glow
                .as_mut()
                .unwrap()
                .run(self.gl_window.as_mut().unwrap().window(), |cc| {
                    self.app_data.update(cc)
                });

            event_loop.set_control_flow(if self.repaint_delay.is_zero() {
                self.gl_window.as_mut().unwrap().window().request_redraw();
                winit::event_loop::ControlFlow::Poll
            } else if let Some(repaint_after_instant) =
                std::time::Instant::now().checked_add(self.repaint_delay)
            {
                winit::event_loop::ControlFlow::WaitUntil(repaint_after_instant)
            } else {
                winit::event_loop::ControlFlow::Wait
            });

            self.egui_glow
                .as_mut()
                .unwrap()
                .paint(self.gl_window.as_mut().unwrap().window());

            self.gl_window.as_mut().unwrap().swap_buffers().unwrap();
            return;
        }

        if let winit::event::WindowEvent::Resized(physical_size) = &event {
            self.gl_window.as_mut().unwrap().resize(*physical_size);
        }

        let event_response = self
            .egui_glow
            .as_mut()
            .unwrap()
            .on_window_event(self.gl_window.as_mut().unwrap().window(), &event);

        if event_response.repaint {
            self.gl_window.as_mut().unwrap().window().request_redraw();
        }
    }

    fn user_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: ReemapGuiEvent,
    ) {
        match event {
            ReemapGuiEvent::Redraw(delay) => self.repaint_delay = delay,
            ReemapGuiEvent::SetWindowVisibility(visible) => {
                self.set_visible(visible, event_loop);
            }
            ReemapGuiEvent::TrayIconEvent(tray_icon::TrayIconEvent::DoubleClick {
                button: tray_icon::MouseButton::Left,
                ..
            }) => {
                self.set_visible(true, event_loop);
                if let Some(ref gl_window) = self.gl_window {
                    gl_window.window().focus_window();
                    gl_window.window().set_minimized(false);
                }
            }
            ReemapGuiEvent::TrayMenuEvent(tray_icon::menu::MenuEvent {
                id: tray_icon::menu::MenuId(id),
            }) => match id.as_str() {
                "MENU_ID_CONFIGURE" => {
                    self.set_visible(true, event_loop);
                    if let Some(ref gl_window) = self.gl_window {
                        gl_window.window().focus_window();
                        gl_window.window().set_minimized(false);
                    }
                }
                "MENU_ID_EXIT" => event_loop.exit(),
                _ => {
                    #[cfg(debug_assertions)]
                    panic!("unrecognized menu ID")
                }
            },
            _ => (),
        }
    }

    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        if let winit::event::StartCause::ResumeTimeReached { .. } = &cause {
            self.gl_window.as_mut().unwrap().window().request_redraw();
        }
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.egui_glow.as_mut().unwrap().destroy();
    }
}

fn create_display(
    event_loop: &winit::event_loop::ActiveEventLoop,
) -> (GlutinWindowContext, glow::Context) {
    let glutin_window_context = unsafe { GlutinWindowContext::new(event_loop) };
    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            let s = std::ffi::CString::new(s)
                .expect("failed to construct C string from string for gl proc address");

            glutin_window_context.get_proc_address(&s)
        })
    };

    (glutin_window_context, gl)
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("failed to open icon")
}

pub fn run<T>(app: T)
where
    T: TrayApp,
{
    let event_loop = winit::event_loop::EventLoop::<ReemapGuiEvent>::with_user_event()
        .build()
        .unwrap();

    let proxy = event_loop.create_proxy();
    tray_icon::TrayIconEvent::set_event_handler(Some(move |event| {
        proxy
            .send_event(ReemapGuiEvent::TrayIconEvent(event))
            .expect("event loop should exist");
    }));

    let proxy = event_loop.create_proxy();
    tray_icon::menu::MenuEvent::set_event_handler(Some(move |event| {
        proxy
            .send_event(ReemapGuiEvent::TrayMenuEvent(event))
            .expect("event loop should exist");
    }));

    let event_loop_proxy = event_loop.create_proxy();

    let mut glow_app = GlowApp::new(event_loop_proxy, app);

    event_loop
        .run_app(&mut glow_app)
        .expect("failed to run app");
}
