use crate::cli;
use crate::text::Paintable;
use crate::text::Paint;
use sctk::compositor::{self, CompositorHandler};
use sctk::output::{self, OutputHandler};
use sctk::registry::ProvidesRegistryState;
use sctk::shell::xdg::window::{self, Window, WindowConfigure};
use sctk::shell::WaylandSurface;
use sctk::shm::slot;
use smithay_client_toolkit as sctk;
use smithay_client_toolkit::shm::slot::Buffer;
use wayland_client::protocol::wl_output::{Transform, WlOutput};
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{protocol, Connection, QueueHandle};

pub struct Bar {
    config: cli::Config,
    registry: sctk::registry::RegistryState,
    output_state: output::OutputState,
    shm: sctk::shm::Shm,
    pool: slot::SlotPool,
    buffer: Option<Buffer>,
    width: u32,
    height: u32,
    layer: sctk::shell::wlr_layer::LayerSurface,
    req_exit: bool,
    queue_handler: QueueHandle<Bar>,
}

impl ProvidesRegistryState for Bar {
    fn registry(&mut self) -> &mut sctk::registry::RegistryState {
        &mut self.registry
    }

    sctk::registry_handlers!();
}

impl OutputHandler for Bar {
    fn output_state(&mut self) -> &mut output::OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: WlOutput) {}

    fn update_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: WlOutput) {}

    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: WlOutput) {
    }
}

impl CompositorHandler for Bar {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _new_factor: i32,
    ) {
        // Ignored
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _new_transform: Transform,
    ) {
        // Ignored
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _time: u32,
    ) {
        self.draw();
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _output: &WlOutput,
    ) {
        // Ignored
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _output: &WlOutput,
    ) {
        // Ignored
    }
}

impl Bar {
    pub fn new(config: cli::Config) -> (Self, wayland_client::EventQueue<Self>) {
        let conn = Connection::connect_to_env().unwrap();

        let (globals, event_queue) = wayland_client::globals::registry_queue_init(&conn).unwrap();
        let qh = event_queue.handle();

        let compositor_state = compositor::CompositorState::bind(&globals, &qh).unwrap();
        let layer_shell = sctk::shell::wlr_layer::LayerShell::bind(&globals, &qh).unwrap();
        let shm = sctk::shm::Shm::bind(&globals, &qh).unwrap();

        let surface = compositor_state.create_surface(&qh);

        let layer = layer_shell.create_layer_surface(
            &qh,
            surface,
            smithay_client_toolkit::shell::wlr_layer::Layer::Top,
            Some("CraBar"),
            None,
        );
        layer.set_anchor(
            sctk::shell::wlr_layer::Anchor::TOP
                | sctk::shell::wlr_layer::Anchor::LEFT
                | sctk::shell::wlr_layer::Anchor::RIGHT,
        );
        layer.set_size(0, 30);
        // Default to no keyboard interactive
        layer.commit();

        let pool = sctk::shm::slot::SlotPool::new(122880, &shm).unwrap();

        (
            Bar {
                config,
                output_state: output::OutputState::new(&globals, &qh),
                registry: sctk::registry::RegistryState::new(&globals),
                req_exit: false,
                pool,
                shm,
                width: 1024,
                height: 30,
                buffer: None,
                layer,
                queue_handler: qh,
            },
            event_queue,
        )
    }

    fn draw(&mut self) {
        let width = self.width;
        let height = self.height;
        let stride = self.width * 4;

        self.layer.set_exclusive_zone(height as i32 + 3);

        let buffer = self.buffer.get_or_insert_with(|| {
            self.pool
                .create_buffer(
                    width as i32,
                    height as i32,
                    stride as i32,
                    protocol::wl_shm::Format::Argb8888,
                )
                .unwrap()
                .0
        });
        let canvas = match self.pool.canvas(buffer) {
            Some(canvas) => canvas,
            None => {
                let (second_buffer, canvas) = self
                    .pool
                    .create_buffer(
                        width as i32,
                        height as i32,
                        stride as i32,
                        protocol::wl_shm::Format::Argb8888,
                    )
                    .unwrap();
                *buffer = second_buffer;
                canvas
            }
        };
        //let mut canvas = andrew::Canvas::new(
        //    &mut canvas,
        //    width as usize,
        //    height as usize,
        //    stride as usize,
        //    andrew::Endian::Big,
        //);
        let mut canvas = crate::text::Canvas::new(height as usize, width as usize, canvas);
        {
            //canvas
            //    .buffer
            //    .chunks_exact_mut(4)
            //    .enumerate()
            //    .for_each(|(_index, chunk)| {
            //        let array: &mut [u8; 4] = chunk.try_into().unwrap();
            //        *array = self.config.background_color().into();
            //    });
            for y in 0..height {
                for x in 0..width {
                    canvas.draw_pixel(x as usize, y as usize, self.config.background_color()).unwrap();
                }
            }
            let mut config = fontconfig::FontConfig::default();
            let font = config.find("sans-serif".to_string(), None);
            let fontpath = font.unwrap().path;
            let fontdata = std::fs::read(fontpath).unwrap();
            let text = crate::text::Text::new(
                "Test".to_string(),
                rusttype::Font::try_from_bytes(&fontdata).unwrap(),
                cli::Color::new(0, 0, 0, 255),
                cli::Color::new(255, 255, 255, 255),
            );
            text.paint(&mut canvas).unwrap();
            //let text = andrew::text::Text::new(
            //    (5, 5),
            //    (&cli::Color::new(0, 0, 0, 0)).into(),
            //    &fontdata,
            //    20.,
            //    1.,
            //    "Test",
            //);
            //text.draw(&mut canvas);
        }

        self.layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);

        self.layer
            .wl_surface()
            .frame(&self.queue_handler, self.layer.wl_surface().clone());

        buffer.attach_to(self.layer.wl_surface()).unwrap();
        self.layer.commit();
    }

    pub fn req_exit(&self) -> bool {
        self.req_exit
    }
}

impl window::WindowHandler for Bar {
    fn request_close(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _window: &Window) {
        self.req_exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _window: &Window,
        _configure: WindowConfigure,
        _serial: u32,
    ) {
        self.draw();
    }
}

impl sctk::shm::ShmHandler for Bar {
    fn shm_state(&mut self) -> &mut sctk::shm::Shm {
        &mut self.shm
    }
}

impl sctk::shell::wlr_layer::LayerShellHandler for Bar {
    fn closed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
    ) {
        self.req_exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
        configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if configure.new_size == (0, 0) {
            self.width = 1024;
            self.height = 30;
        } else {
            self.width = configure.new_size.0;
            self.height = configure.new_size.1;
        }
        self.draw();
    }
}

sctk::delegate_registry!(Bar);
sctk::delegate_compositor!(Bar);
sctk::delegate_xdg_shell!(Bar);
sctk::delegate_xdg_window!(Bar);
sctk::delegate_output!(Bar);
sctk::delegate_shm!(Bar);
sctk::delegate_layer!(Bar);
