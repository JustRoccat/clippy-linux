#![allow(dead_code)]

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tiny_skia::Pixmap;
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_region, wl_shm, wl_surface},
    Connection, Dispatch, QueueHandle,
};

use crate::assets::AssetManager;
use crate::ui::{AppState, UiRenderer, WINDOW_H, WINDOW_W};

pub struct WaylandApp {
    pub qh: QueueHandle<Self>,
    pub registry: RegistryState,
    pub output_state: OutputState,
    pub shm: Shm,
    pub compositor: CompositorState,
    pub renderer: Arc<UiRenderer>,
    pub surface: LayerSurface,
    pub pool: SlotPool,
    pub width: u32,
    pub height: u32,
    pub first_configure: bool,
    pub dirty: bool,
}

impl Dispatch<wl_region::WlRegion, ()> for WaylandApp {
    fn event(
        _state: &mut Self,
        _proxy: &wl_region::WlRegion,
        _event: wl_region::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl WaylandApp {
    pub fn run(state: Arc<Mutex<AppState>>, assets: Arc<AssetManager>) {
        let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
        let (globals, mut event_queue) =
            registry_queue_init(&conn).expect("Failed to init registry");
        let qh = event_queue.handle();

        let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
        let layer_shell = LayerShell::bind(&globals, &qh).expect("Layer shell not supported");
        let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");
        let output_state = OutputState::new(&globals, &qh);

        let renderer = Arc::new(UiRenderer::new(state, assets));

        let wl_surf = compositor.create_surface(&qh);

        let region: wl_region::WlRegion = compositor.wl_compositor().create_region(&qh, ());
        wl_surf.set_input_region(Some(&region));
        region.destroy();

        let layer = layer_shell.create_layer_surface(
            &qh,
            wl_surf,
            Layer::Overlay,
            Some("clippy-linux"),
            None,
        );
        layer.set_anchor(Anchor::BOTTOM | Anchor::RIGHT);
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer.set_exclusive_zone(-1);
        layer.set_margin(0, 0, 0, 0);
        layer.set_size(WINDOW_W as u32, WINDOW_H as u32);
        layer.commit();

        let pool = SlotPool::new((WINDOW_W * WINDOW_H * 4) as usize, &shm)
            .expect("Failed to create SHM pool");

        let mut app = Self {
            qh,
            registry: RegistryState::new(&globals),
            output_state,
            shm,
            compositor,
            renderer,
            surface: layer,
            pool,
            width: WINDOW_W as u32,
            height: WINDOW_H as u32,
            first_configure: true,
            dirty: true,
        };

        loop {
            event_queue
                .blocking_dispatch(&mut app)
                .expect("dispatch failed");
            if app.dirty {
                app.draw();
            }
        }
    }

    fn draw(&mut self) {
        let width = self.width;
        let height = self.height;
        let stride = width as i32 * 4;

        let (buffer, canvas) = match self.pool.create_buffer(
            width as i32,
            height as i32,
            stride,
            wl_shm::Format::Argb8888,
        ) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Failed to create buffer: {e:?}");
                return;
            }
        };

        let mut pixmap = Pixmap::new(width, height).expect("Failed to create pixmap");
        self.renderer.render(&mut pixmap);

        let rgba = pixmap.data();
        canvas
            .chunks_exact_mut(4)
            .enumerate()
            .for_each(|(i, chunk)| {
                let r = rgba[i * 4];
                let g = rgba[i * 4 + 1];
                let b = rgba[i * 4 + 2];
                let a = rgba[i * 4 + 3];
                chunk[0] = b;
                chunk[1] = g;
                chunk[2] = r;
                chunk[3] = a;
            });

        self.surface
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);

        self.surface
            .wl_surface()
            .frame(&self.qh, self.surface.wl_surface().clone());
        buffer
            .attach_to(self.surface.wl_surface())
            .expect("buffer attach");
        self.surface.commit();

        self.dirty = false;
    }
}

impl LayerShellHandler for WaylandApp {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        std::process::exit(0);
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if configure.new_size.0 == 0 || configure.new_size.1 == 0 {
            self.width = WINDOW_W as u32;
            self.height = WINDOW_H as u32;
        } else {
            self.width = configure.new_size.0;
            self.height = configure.new_size.1;
        }
        if self.first_configure {
            self.first_configure = false;
        }
        self.dirty = true;
    }
}

impl ShmHandler for WaylandApp {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl OutputHandler for WaylandApp {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl CompositorHandler for WaylandApp {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        self.dirty = true;
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
        self.dirty = true;
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.dirty = true;
    }
}

impl ProvidesRegistryState for WaylandApp {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }
    registry_handlers![OutputState];
}

delegate_compositor!(WaylandApp);
delegate_output!(WaylandApp);
delegate_shm!(WaylandApp);
delegate_layer!(WaylandApp);
delegate_registry!(WaylandApp);
