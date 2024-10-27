use crate::cli;
use crate::paint::Paint;
use crate::paint::Paintable;
use sctk::compositor::{self, CompositorHandler};
use sctk::output::{self, OutputHandler};
use sctk::registry::ProvidesRegistryState;
use sctk::shell::xdg::window::{self, Window, WindowConfigure};
use sctk::shell::WaylandSurface;
use sctk::shm::slot;
use smithay_client_toolkit as sctk;
use smithay_client_toolkit::seat::pointer::PointerEventKind;
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
    seat_state: sctk::seat::SeatState,
    pointer: Option<wayland_client::protocol::wl_pointer::WlPointer>,
    data: String,
    fontpath: std::path::PathBuf,
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

        let seat_state = smithay_client_toolkit::seat::SeatState::new(&globals, &qh);

        let mut fontconfig = fontconfig::FontConfig::default();
        let font = fontconfig.find("sans-serif".to_string(), None);
        let fontpath = font.unwrap().path;
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
                seat_state,
                pointer: None,
                data: "".into(),
                fontpath,
            },
            event_queue,
        )
    }

    fn draw(&mut self) {
        let width = self.width;
        let height = self.height;
        let stride = self.width * 4;

        self.layer.set_exclusive_zone(height as i32 + 3);

        self.data.clear();
        let stdin = std::io::stdin();
        match stdin.read_line(&mut self.data) {
            Ok(n) => {
                if n == 0 {
                    log::info!("n == 0; exiting");
                    self.req_exit = true;
                    return;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                log::info!("Broken pipe; exit normally");
                self.req_exit = true;
                return;
            }
            Err(ref e) => {
                log::error!("Cannot get new input: {}", e.kind())
            }
        }
        self.data.pop();

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
        let mut canvas = crate::paint::Canvas::new(height as usize, width as usize, canvas);
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
                    canvas
                        .draw_pixel(x as usize, y as usize, self.config.background_color())
                        .unwrap();
                }
            }
            let mut config = fontconfig::FontConfig::default();
            let font = config.find("sans-serif".to_string(), None);
            let fontpath = font.unwrap().path;
            let fontdata = std::fs::read(fontpath).unwrap();
            let font = rusttype::Font::try_from_bytes(&fontdata).unwrap();

            let mut fg = self.config.foreground_color();
            let mut bg = self.config.background_color();
            let margin_top = 5;
            let mut margin_left = 5;
            for part in self
                .data
                .parse::<crate::parse::StyledString>()
                .unwrap_or_default()
                .into_content()
                .into_iter()
            {
                match part {
                    crate::parse::StyledStringPart::String(string) => {
                        let text = crate::paint::Text::new(string, font.clone(), fg, bg);
                        for y in 0..height {
                            for x in margin_left..(text.get_region().0 as usize + margin_left) {
                                canvas.draw_pixel(x + 1, y as usize, bg).unwrap();
                            }
                        }
                        let mut slice = canvas
                            .slice(
                                margin_left,
                                margin_top,
                                (width as usize - margin_left).try_into().unwrap(),
                                (height as usize - margin_top).try_into().unwrap(),
                            )
                            .unwrap();
                        text.paint(&mut slice).unwrap();
                        margin_left += text.get_region().0 as usize;
                    }
                    crate::parse::StyledStringPart::Style(style) => {
                        fg = style.foreground_color().unwrap_or(fg);
                        bg = style.background_color().unwrap_or(bg);
                    }
                }
            }
            //let text = crate::paint::Text::new(
            //    self.data.clone(),
            //    font,
            //    self.config.foreground_color(),
            //    self.config.background_color(),
            //);
            //text.paint(&mut canvas).unwrap();
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

impl sctk::seat::pointer::PointerHandler for Bar {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &protocol::wl_pointer::WlPointer,
        events: &[smithay_client_toolkit::seat::pointer::PointerEvent],
    ) {
        for event in events {
            if event.surface != *self.layer.wl_surface() {
                continue;
            }
            match event.kind {
                PointerEventKind::Release { button, .. } => {
                    let splitted_content = self
                        .data
                        .parse::<crate::parse::StyledString>()
                        .unwrap_or_default()
                        .into_content()
                        .into_iter()
                        .filter_map(|v| match v {
                            crate::parse::StyledStringPart::Style(_) => None,
                            crate::parse::StyledStringPart::String(str) => Some(str),
                        });
                    let mut number = None;
                    let mut margin: f64 = 5.;
                    for (idx, content) in splitted_content.enumerate() {
                        let font = rusttype::Font::try_from_vec(
                            std::fs::read::<&std::path::Path>(self.fontpath.as_ref()).unwrap(),
                        )
                        .unwrap();
                        let text_obj = crate::paint::Text::new(
                            content.to_owned(),
                            font,
                            self.config.foreground_color(),
                            self.config.background_color(),
                        );
                        let (width, _) = text_obj.get_region();
                        let width = width as f64 + margin;
                        let right_bound = margin + width;
                        if (margin..right_bound).contains(&event.position.0) {
                            number = Some(idx);
                        }
                        margin += width;
                    }
                    if let Some(number) = number {
                        log::info!("Pointer release key {} at #{}", button, number);
                    } else {
                        log::info!("Pointer release key {} at nowhere", button);
                    }
                }
                PointerEventKind::Axis {
                    horizontal,
                    vertical,
                    ..
                } => {
                    log::info!(
                        "Mouse wheel rotating h {} v {}",
                        horizontal.discrete,
                        vertical.discrete
                    );
                }
                _ => {}
            }
        }
    }
}

impl sctk::seat::SeatHandler for Bar {
    fn seat_state(&mut self) -> &mut smithay_client_toolkit::seat::SeatState {
        &mut self.seat_state
    }

    fn new_seat(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: protocol::wl_seat::WlSeat,
    ) {
        // Ignored
    }

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: protocol::wl_seat::WlSeat,
        capability: smithay_client_toolkit::seat::Capability,
    ) {
        if capability == sctk::seat::Capability::Pointer && self.pointer.is_none() {
            log::info!("Initializing pointer");
            let pointer = self.seat_state.get_pointer(qh, &seat).unwrap();
            self.pointer = Some(pointer);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: protocol::wl_seat::WlSeat,
        _capability: smithay_client_toolkit::seat::Capability,
    ) {
        todo!()
    }

    fn remove_seat(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: protocol::wl_seat::WlSeat,
    ) {
        todo!()
    }
}

sctk::delegate_registry!(Bar);
sctk::delegate_compositor!(Bar);
sctk::delegate_xdg_shell!(Bar);
sctk::delegate_xdg_window!(Bar);
sctk::delegate_output!(Bar);
sctk::delegate_shm!(Bar);
sctk::delegate_layer!(Bar);
sctk::delegate_seat!(Bar);
sctk::delegate_pointer!(Bar);
