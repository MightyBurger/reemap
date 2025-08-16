// Copyright 2025 Jordan Johnson
// SPDX-License-Identifier: Apache-2.0 OR MIT

use glutin::prelude::NotCurrentGlContext;
use std::num::NonZeroU32;
use tracing::debug;
use winit::{
    platform::windows::WindowAttributesExtWindows, raw_window_handle::HasWindowHandle as _,
};

pub struct GlutinWindowContext {
    window: winit::window::Window,
    gl_context: glutin::context::PossiblyCurrentContext,
    gl_display: glutin::display::Display,
    gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

impl GlutinWindowContext {
    pub unsafe fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        use super::ICON32_HEIGHT;
        use super::ICON32_RAW_RGBA;
        use super::ICON32_WIDTH;
        use super::ICON256_HEIGHT;
        use super::ICON256_RAW_RGBA;
        use super::ICON256_WIDTH;
        use glutin::display::GetGlDisplay as _;
        use glutin::display::GlDisplay as _;
        use glutin::prelude::GlSurface as _;

        let winit_window_builder = winit::window::WindowAttributes::default()
            .with_inner_size(crate::gui::SIZE)
            .with_resizable(false)
            .with_enabled_buttons({
                let mut enabled_buttons = winit::window::WindowButtons::all();
                enabled_buttons.remove(winit::window::WindowButtons::MAXIMIZE);
                enabled_buttons
            })
            .with_title(crate::gui::TITLE)
            .with_fullscreen(None)
            .with_maximized(false)
            // .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
            .with_theme(Some(winit::window::Theme::Dark))
            .with_window_icon(Some(
                winit::window::Icon::from_rgba(
                    ICON32_RAW_RGBA.to_vec(),
                    ICON32_WIDTH,
                    ICON32_HEIGHT,
                )
                .expect("failed to open icon"),
            ))
            .with_taskbar_icon(Some(
                winit::window::Icon::from_rgba(
                    ICON256_RAW_RGBA.to_vec(),
                    ICON256_WIDTH,
                    ICON256_HEIGHT,
                )
                .expect("failed to open icon"),
            ))
            .with_visible(false);

        let config_template_builder = glutin::config::ConfigTemplateBuilder::new()
            .prefer_hardware_accelerated(None)
            .with_depth_size(0)
            .with_stencil_size(0)
            .with_transparency(false);

        debug!("trying to get gl_config");

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
        debug!("found gl_config: {:?}", &gl_config);

        let raw_window_handle = window.as_ref().map(|w| {
            w.window_handle()
                .expect("failed to get window handle")
                .as_raw()
        });
        debug!("raw window handle: {raw_window_handle:?}");

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
                    debug!("failed to create gl_context with attributes: {:?}. retrying with fallback context attributes: {:?}",
                    &context_attributes, &fallback_context_attributes);
                    gl_config.display().create_context(&gl_config, &fallback_context_attributes).expect("failed to create context even with fallback attributes")
                })
        };

        let window = window.take().unwrap_or_else(|| {
            debug!("window doesn't exist yet. creating one now with finalize_window");
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
        debug!("surface created successfully: {gl_surface:?}. making context current");
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

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn resize(&self, physical_size: winit::dpi::PhysicalSize<u32>) {
        use glutin::surface::GlSurface as _;
        if let (Ok(width), Ok(height)) = (
            physical_size.width.try_into(),
            physical_size.height.try_into(),
        ) {
            self.gl_surface.resize(&self.gl_context, width, height);
        }
    }

    pub fn swap_buffers(&self) -> glutin::error::Result<()> {
        use glutin::surface::GlSurface as _;
        self.gl_surface.swap_buffers(&self.gl_context)
    }

    pub fn get_proc_address(&self, addr: &std::ffi::CStr) -> *const std::ffi::c_void {
        use glutin::display::GlDisplay as _;
        self.gl_display.get_proc_address(addr)
    }
}
