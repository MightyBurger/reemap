// Copyright 2025 Jordan Johnson
// SPDX-License-Identifier: Apache-2.0 OR MIT

// The following code is loosely based on this example:
// https://github.com/emilk/egui/blob/main/crates/egui_glow/examples/pure_glow.rs
// Other code uses some concepts from eframe.

mod glutin_ctx;
pub mod reemapp;

use crate::buttons;

use glutin_ctx::GlutinWindowContext;
use tracing::{trace, warn};

const TITLE: &str = "Reemap";
const SIZE: winit::dpi::LogicalSize<f64> = winit::dpi::LogicalSize {
    width: 800.0,
    height: 600.0,
};
const START_VISIBLE: bool = false;

// To understand where this constant comes from, look at build.rs.
// The build script extracts images from the .ico file and saves those images to a temporary
// compile-time location. The build script then sets ICON{32, 256}_RAW_PATH to point to those files.
const ICON32_RAW_RGBA: &[u8] = include_bytes!(env!("ICON32_RAW_PATH"));
const ICON32_WIDTH: u32 = 32;
const ICON32_HEIGHT: u32 = 32;

const ICON256_RAW_RGBA: &[u8] = include_bytes!(env!("ICON256_RAW_PATH"));
const ICON256_WIDTH: u32 = 256;
const ICON256_HEIGHT: u32 = 256;

use std::sync::Arc;
use std::time::Instant;
use tray_icon::TrayIcon;

#[derive(Debug)]
#[allow(dead_code)] // I'd like to keep SetWindowVisibility here in case the GUI ever needs it.
pub enum ReemapGuiEvent {
    RequestRepaint {
        when: Instant,
        cumulative_pass_nr: u64,
    },
    SetWindowVisibility(bool),
    TrayIconEvent(tray_icon::TrayIconEvent),
    TrayMenuEvent(tray_icon::menu::MenuEvent),
    ChangedProfile(Option<String>),
    ButtonPressed(buttons::Button),
}

// Just something to pass along a little more info to the app.
// Only used in Reemap to pass along information of what button was pressed last.
#[derive(Debug, Default, Clone)]
pub struct TrayAppCtx {
    last_pressed_button: Option<buttons::Button>,
}

pub trait TrayApp {
    fn update(&mut self, egui_ctx: &egui::Context, app_ctx: &TrayAppCtx);
}

struct GlowApp<T: TrayApp> {
    proxy: winit::event_loop::EventLoopProxy<ReemapGuiEvent>,
    gl_window: Option<GlutinWindowContext>,
    gl: Option<Arc<glow::Context>>,
    egui_glow: Option<egui_glow::EguiGlow>,
    next_repaint_time: Option<Instant>,
    tray_icon: Option<TrayIcon>,
    app_ctx: TrayAppCtx,
    app_data: T,
}

impl<T: TrayApp> GlowApp<T> {
    fn new(proxy: winit::event_loop::EventLoopProxy<ReemapGuiEvent>, app_data: T) -> Self {
        Self {
            proxy,
            gl_window: None,
            gl: None,
            egui_glow: None,
            next_repaint_time: Some(Instant::now()),
            tray_icon: None,
            app_ctx: TrayAppCtx::default(),
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

    fn check_repaint_time(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let now = Instant::now();

        match self.next_repaint_time {
            Some(time) if now > time => {
                trace!(?time, "GUI repainting now!");
                // GUI needs to repaint now!
                // This change to Poll will be undone shortly, before the message loop is empty.
                // I believe this just prevents new_events from triggering, preventing an endless loop
                // of calling request_redraw.
                event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
                if let Some(gl_window) = self.gl_window.as_mut() {
                    gl_window.window().request_redraw();
                    self.next_repaint_time = None;
                } else {
                    warn!("Window wasn't open yet. Repainting again soon.");
                }
            }
            Some(time) => {
                // GUI will need to repaint, but not yet. Just wait a minute.
                event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(time));
            }
            None => {
                // GUI doesn't have a need to re-paint.
                event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
            }
        };
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
                trace!(?info, "request repaint callback");
                let when = Instant::now() + info.delay;
                let cumulative_pass_nr = info.current_cumulative_pass_nr;
                event_loop_proxy
                    .lock()
                    .send_event(ReemapGuiEvent::RequestRepaint {
                        when,
                        cumulative_pass_nr,
                    })
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
            let icon =
                tray_icon::Icon::from_rgba(ICON32_RAW_RGBA.to_vec(), ICON32_WIDTH, ICON32_HEIGHT)
                    .expect("failed to open icon");

            tray_icon::TrayIconBuilder::new()
                .with_menu(Box::new(tray_menu))
                .with_tooltip(TITLE)
                .with_icon(icon)
                .with_title(TITLE)
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
            trace!("redraw requested");
            self.egui_glow
                .as_mut()
                .unwrap()
                .run(self.gl_window.as_mut().unwrap().window(), |cc| {
                    self.app_data.update(cc, &self.app_ctx)
                });

            unsafe {
                use glow::HasContext as _;
                const CLEAR_COLOR: [f32; 4] = [0.1, 0.1, 0.1, 1.0];
                self.gl.as_mut().unwrap().clear_color(
                    CLEAR_COLOR[0],
                    CLEAR_COLOR[1],
                    CLEAR_COLOR[2],
                    CLEAR_COLOR[3],
                );
                self.gl.as_mut().unwrap().clear(glow::COLOR_BUFFER_BIT);
            }

            self.egui_glow
                .as_mut()
                .unwrap()
                .paint(self.gl_window.as_mut().unwrap().window());

            self.gl_window.as_mut().unwrap().swap_buffers().unwrap();
            self.check_repaint_time(event_loop);
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
            ReemapGuiEvent::RequestRepaint {
                when,
                cumulative_pass_nr,
            } => {
                let current_pass_nr = self
                    .egui_glow
                    .as_ref()
                    .unwrap()
                    .egui_ctx
                    .cumulative_pass_nr();
                trace!(
                    ?when,
                    ?cumulative_pass_nr,
                    ?current_pass_nr,
                    "request repaint user event"
                );

                if current_pass_nr == cumulative_pass_nr
                    || current_pass_nr == cumulative_pass_nr + 1
                {
                    self.next_repaint_time = Some(when);
                }
            }
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
            ReemapGuiEvent::TrayIconEvent(_) => {}
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
            ReemapGuiEvent::ChangedProfile(name) => {
                let title = match name {
                    Some(name) => format!("{name} - Reemap"),
                    None => "Reemap".to_string(),
                };
                if let Some(ref gl_window) = self.gl_window {
                    gl_window.window().set_title(&title);
                }
            }
            ReemapGuiEvent::ButtonPressed(button) => {
                self.app_ctx.last_pressed_button = Some(button);
                self.next_repaint_time = Some(std::time::Instant::now());
            }
        }
        self.check_repaint_time(event_loop);
    }

    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        if let winit::event::StartCause::ResumeTimeReached {
            start,
            requested_resume,
        } = &cause
        {
            trace!(?start, ?requested_resume, "woke up, checking repaint time");
        }
        self.check_repaint_time(event_loop);
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

// Pass in the event_loop so the caller has the opportunity to create a proxy first.
pub fn run<T>(app: T, event_loop: winit::event_loop::EventLoop<ReemapGuiEvent>)
where
    T: TrayApp,
{
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
