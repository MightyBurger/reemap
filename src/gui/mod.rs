// The following code is from here:
// https://github.com/emilk/egui/blob/main/crates/egui_glow/examples/pure_glow.rs

const TITLE: &'static str = "Reemap";
const SIZE: winit::dpi::LogicalSize<f64> = winit::dpi::LogicalSize {
    width: 800.0,
    height: 600.0,
};

use glutin::prelude::NotCurrentGlContext;
use std::{num::NonZeroU32, sync::Arc};
use tray_icon::TrayIcon;
use winit::raw_window_handle::HasWindowHandle as _;

struct GlutinWindowContext {
    window: winit::window::Window,
    gl_context: glutin::context::PossiblyCurrentContext,
    gl_display: glutin::display::Display,
    gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

impl GlutinWindowContext {
    unsafe fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        use glutin::display::GetGlDisplay as _;
        use glutin::display::GlDisplay as _;
        use glutin::prelude::GlSurface as _;
        let winit_window_builder = winit::window::WindowAttributes::default()
            .with_resizable(true)
            .with_inner_size(SIZE)
            .with_title(TITLE)
            .with_visible(false);

        let config_template_builder = glutin::config::ConfigTemplateBuilder::new()
            .prefer_hardware_accelerated(None)
            .with_depth_size(0)
            .with_stencil_size(0)
            .with_transparency(false);

        log::debug!("trying to get gl_config");

        let (mut window, gl_config) = glutin_winit::DisplayBuilder::new()
            .with_preference(glutin_winit::ApiPreference::FallbackEgl)
            .with_window_attributes(Some(winit_window_builder.clone()))
            .build(
                event_loop,
                config_template_builder,
                |mut config_iterator| {
                    config_iterator.next().expect(
                        "failed to find a matching configuration for creating glutin config",
                    )
                },
            )
            .expect("failed to create gl_config");

        let gl_display = gl_config.display();
        log::debug!("found gl_config: {:?}", &gl_config);

        let raw_window_handle = window.as_ref().map(|w| {
            w.window_handle()
                .expect("failed to get window handle")
                .as_raw()
        });
        log::debug!("raw window handle: {:?}", raw_window_handle);

        let context_attributes =
            glutin::context::ContextAttributesBuilder::new().build(raw_window_handle);

        // by default, glutin will try to create a core opengl context. but, if it is not available, try to create a gl-es context using this fallback attributes
        let fallback_context_attributes = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::Gles(None))
            .build(raw_window_handle);

        let not_current_gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    log::debug!("failed to create gl_context with attributes: {:?}. retrying with fallback context attributes: {:?}",
                    &context_attributes, &fallback_context_attributes);
                    gl_config.display().create_context(&gl_config, &fallback_context_attributes).expect("failed to create context even with fallback attributes")
                })
        };

        let window = window.take().unwrap_or_else(|| {
            log::debug!("window doesn't exist yet. creating one now with finalize_window");
            glutin_winit::finalize_window(event_loop, winit_window_builder.clone(), &gl_config)
                .expect("failed to finalize glutin window")
        });

        let (width, height): (u32, u32) = window.inner_size().into();
        let width = NonZeroU32::new(width).unwrap_or(NonZeroU32::MIN);
        let height = NonZeroU32::new(height).unwrap_or(NonZeroU32::MIN);
        let surface_attributes =
            glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
                .build(
                    window
                        .window_handle()
                        .expect("failed to get window handle")
                        .as_raw(),
                    width,
                    height,
                );

        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &surface_attributes)
                .unwrap()
        };
        log::debug!("surface created successfully: {gl_surface:?}. making context current");
        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

        gl_surface
            .set_swap_interval(
                &gl_context,
                glutin::surface::SwapInterval::Wait(NonZeroU32::MIN),
            )
            .unwrap();
        Self {
            window,
            gl_context,
            gl_display,
            gl_surface,
        }
    }

    fn window(&self) -> &winit::window::Window {
        &self.window
    }

    fn resize(&self, physical_size: winit::dpi::PhysicalSize<u32>) {
        use glutin::surface::GlSurface as _;
        if let (Ok(width), Ok(height)) = (
            physical_size.width.try_into(),
            physical_size.height.try_into(),
        ) {
            self.gl_surface.resize(&self.gl_context, width, height);
        }
    }

    fn swap_buffers(&self) -> glutin::error::Result<()> {
        use glutin::surface::GlSurface as _;
        self.gl_surface.swap_buffers(&self.gl_context)
    }

    fn get_proc_address(&self, addr: &std::ffi::CStr) -> *const std::ffi::c_void {
        use glutin::display::GlDisplay as _;
        self.gl_display.get_proc_address(addr)
    }
}

#[derive(Debug)]
pub enum ReemapWindowEvent {
    Redraw(std::time::Duration),
    SetWindowVisibility(bool),
    TrayIconEvent(tray_icon::TrayIconEvent),
    TrayMenuEvent(tray_icon::menu::MenuEvent),
}

struct GlowApp {
    proxy: winit::event_loop::EventLoopProxy<ReemapWindowEvent>,
    gl_window: Option<GlutinWindowContext>,
    gl: Option<Arc<glow::Context>>,
    egui_glow: Option<egui_glow::EguiGlow>,
    repaint_delay: std::time::Duration,
    clear_color: [f32; 3],
    tray_icon: Option<TrayIcon>,
}

impl GlowApp {
    fn new(proxy: winit::event_loop::EventLoopProxy<ReemapWindowEvent>) -> Self {
        Self {
            proxy,
            gl_window: None,
            gl: None,
            egui_glow: None,
            repaint_delay: std::time::Duration::MAX,
            clear_color: [0.1, 0.1, 0.1],
            tray_icon: None,
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

impl winit::application::ApplicationHandler<ReemapWindowEvent> for GlowApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let (gl_window, gl) = create_display(event_loop);
        let gl = std::sync::Arc::new(gl);
        gl_window.window().set_visible(false);

        let egui_glow = egui_glow::EguiGlow::new(event_loop, gl.clone(), None, None, true);

        let event_loop_proxy = egui::mutex::Mutex::new(self.proxy.clone());
        egui_glow
            .egui_ctx
            .set_request_repaint_callback(move |info| {
                event_loop_proxy
                    .lock()
                    .send_event(ReemapWindowEvent::Redraw(info.delay))
                    .expect("Cannot send event")
            });
        self.gl_window = Some(gl_window);
        self.gl = Some(gl);
        self.egui_glow = Some(egui_glow);

        let tray_menu = {
            let menu = tray_icon::menu::Menu::new();
            let item1 = tray_icon::menu::MenuItem::new("item1", true, None);
            dbg!(item1.id());
            if let Err(err) = menu.append(&item1) {
                println!("{err:?}");
            }
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
        let mut redraw = || {
            let mut quit = false;

            self.egui_glow.as_mut().unwrap().run(
                self.gl_window.as_mut().unwrap().window(),
                |egui_ctx| {
                    egui::SidePanel::left("my_side_panel").show(egui_ctx, |ui| {
                        ui.heading("Hello World!");
                        if ui.button("Quit").clicked() {
                            quit = true;
                        }
                        ui.color_edit_button_rgb(self.clear_color.as_mut().try_into().unwrap());
                    });
                },
            );

            if quit {
                event_loop.exit();
            } else {
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
            }

            unsafe {
                use glow::HasContext as _;
                self.gl.as_mut().unwrap().clear_color(
                    self.clear_color[0],
                    self.clear_color[1],
                    self.clear_color[2],
                    1.0,
                );
                self.gl.as_mut().unwrap().clear(glow::COLOR_BUFFER_BIT);
            }

            self.egui_glow
                .as_mut()
                .unwrap()
                .paint(self.gl_window.as_mut().unwrap().window());

            self.gl_window.as_mut().unwrap().swap_buffers().unwrap();
        };

        use winit::event::WindowEvent;
        if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
            event_loop.exit();
            return;
        }

        if matches!(event, WindowEvent::RedrawRequested) {
            redraw();
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
        event: ReemapWindowEvent,
    ) {
        match event {
            ReemapWindowEvent::Redraw(delay) => self.repaint_delay = delay,
            ReemapWindowEvent::SetWindowVisibility(visible) => {
                self.set_visible(visible, event_loop);
            }
            ReemapWindowEvent::TrayIconEvent(tray_icon::TrayIconEvent::DoubleClick {
                button: tray_icon::MouseButton::Left,
                ..
            }) => self.set_visible(true, event_loop),
            ReemapWindowEvent::TrayMenuEvent(other) => {
                dbg!(other);
            }
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

pub fn run() {
    let event_loop = winit::event_loop::EventLoop::<ReemapWindowEvent>::with_user_event()
        .build()
        .unwrap();

    let proxy = event_loop.create_proxy();
    std::thread::spawn(move || {
        let invisible_time = 1000;
        let visible_time = 8000;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(invisible_time));
            proxy
                .send_event(ReemapWindowEvent::SetWindowVisibility(true))
                .unwrap();
            std::thread::sleep(std::time::Duration::from_millis(visible_time));
            proxy
                .send_event(ReemapWindowEvent::SetWindowVisibility(false))
                .unwrap();
        }
    });

    let proxy = event_loop.create_proxy();
    tray_icon::TrayIconEvent::set_event_handler(Some(move |event| {
        proxy
            .send_event(ReemapWindowEvent::TrayIconEvent(event))
            .expect("event loop should exist");
    }));

    let proxy = event_loop.create_proxy();
    tray_icon::menu::MenuEvent::set_event_handler(Some(move |event| {
        proxy
            .send_event(ReemapWindowEvent::TrayMenuEvent(event))
            .expect("event loop should exist");
    }));

    let proxy = event_loop.create_proxy();
    let mut app = GlowApp::new(proxy);
    event_loop.run_app(&mut app).expect("failed to run app");
}
