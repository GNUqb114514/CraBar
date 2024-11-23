use crate::paint::Paint;
use crate::{
    cli::{self, Color},
    paint::Paintable,
    parse::StyledStringPart,
};
use sctk::compositor::{self, CompositorHandler};
use sctk::output::{self, OutputHandler};
use sctk::registry::ProvidesRegistryState;
use sctk::shell::xdg::window::{self, Window, WindowConfigure};
use sctk::shell::WaylandSurface;
use sctk::shm::slot;
use smithay_client_toolkit as sctk;
use smithay_client_toolkit::seat::pointer::PointerEventKind;
use smithay_client_toolkit::shm::slot::Buffer;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
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
    data: Arc<Mutex<(String, bool)>>,
    condvar: Arc<Condvar>,
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

#[derive(Debug)]
struct Action {
    button: u8,
    cmd: String,
    start: usize,
    end: usize,
}

struct Command {
    string: String,
    fg: Color,
    bg: Color,
    start: usize,
    end: usize,
}

impl Action {
    pub fn offset(&mut self, offset: usize) {
        self.start += offset;
        self.end += offset;
    }

    pub fn into_offset(mut self, offset: usize) -> Self {
        self.offset(offset);
        self
    }
}

impl Command {
    pub fn offset(&mut self, offset: usize) {
        self.start += offset;
        self.end += offset;
    }

    pub fn into_offset(mut self, offset: usize) -> Self {
        self.offset(offset);
        self
    }
}

impl Bar {
    fn get_width(&self, string: &str) -> f32 {
        let font = rusttype::Font::try_from_vec(
            std::fs::read::<&std::path::Path>(self.fontpath.as_ref()).unwrap(),
        )
        .unwrap();
        let text_obj = crate::paint::Text::new(
            string.to_owned(),
            &font,
            self.config.foreground_color(),
            self.config.background_color(),
        );
        let (width, _) = text_obj.get_region();
        width
    }

    fn parse_to_actions(&self) -> Result<Vec<Action>, ()> {
        let mut lcursor = 0;
        let mut rcursor = 0;
        let mut ccursor = 0;
        let mut align = crate::parse::Align::Left;
        let mut pending = None;
        let data = self.data.lock().unwrap();
        let mut left = vec![];
        let mut right = vec![];
        let mut center = vec![];
        for i in data
            .0
            .parse::<crate::parse::StyledString>()
            .map_err(|_| ())?
            .into_content()
        {
            match i {
                StyledStringPart::Style(_) => {} // Styles are irrelevant to action
                // handling
                StyledStringPart::String(string) => match align {
                    crate::parse::Align::Left => {
                        lcursor += self.get_width(&string) as usize;
                    }
                    crate::parse::Align::Right => {
                        rcursor += self.get_width(&string) as usize;
                    }
                    crate::parse::Align::Center => {
                        ccursor += self.get_width(&string) as usize;
                    }
                },
                StyledStringPart::Action(action) => {
                    let (button, cmd) = action.into_tuple();
                    pending = Some(Action {
                        button,
                        cmd,
                        start: match align {
                            crate::parse::Align::Left => lcursor,
                            crate::parse::Align::Center => ccursor,
                            crate::parse::Align::Right => rcursor,
                        },
                        end: 0, // Temp
                    });
                }
                StyledStringPart::ActionEnd => {
                    if let Some(pending) = std::mem::take(&mut pending) {
                        match align {
                            crate::parse::Align::Left => left.push(Action {
                                end: lcursor,
                                ..pending
                            }),
                            crate::parse::Align::Center => center.push(Action {
                                end: ccursor,
                                ..pending
                            }),
                            crate::parse::Align::Right => right.push(Action {
                                end: rcursor,
                                ..pending
                            }),
                        }
                    }
                }
                StyledStringPart::Swap => {} // Styles are irrelevant to action
                StyledStringPart::Align(align_) => {
                    if pending.is_some() {
                        log::error!("Cannot change align in actions!");
                        continue;
                    }
                    align = align_;
                }
            }
        }
        if let Some(pending) = pending {
            log::warn!("Unclosed action block; check your feeding script");
            match align {
                crate::parse::Align::Left => left.push(Action {
                    end: lcursor,
                    ..pending
                }),
                crate::parse::Align::Center => center.push(Action {
                    end: ccursor,
                    ..pending
                }),
                crate::parse::Align::Right => right.push(Action {
                    end: rcursor,
                    ..pending
                }),
            }
        }
        let retval = left
            .into_iter()
            .map(|v| v.into_offset(5))
            .chain(
                center
                    .into_iter()
                    .map(|v| v.into_offset((self.width as usize - ccursor) / 2)),
            )
            .chain(
                right
                    .into_iter()
                    .map(|v| v.into_offset(self.width as usize - 5 - rcursor)),
            )
            .collect();
        Ok(retval)
    }

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
            config.name(),
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
                data: Arc::new(Mutex::new(("".into(), false))),
                fontpath,
                condvar: Arc::new(Condvar::new()),
            },
            event_queue,
        )
    }

    fn draw(&mut self) {
        let width = self.width;
        let height = self.height;
        let stride = self.width * 4;

        self.layer.set_exclusive_zone(height as i32 + 3);

        let mut data = self.data.lock().unwrap();
        #[cfg(feature = "logs")]
        log::info!("Pending on condvar...");
        while !data.1 {
            data = self.condvar.wait(data).unwrap();
        }
        data.1 = false;
        let data = &data.0;
        #[cfg(feature = "logs")]
        log::info!("Got new data: {}", data);

        let mut fg = self.config.foreground_color();
        let mut bg = self.config.background_color();
        let mut lcursor = 5;
        let mut rcursor = 5;
        let mut ccursor = 5;
        let mut align = crate::parse::Align::Left;
        let mut left = vec![];
        let mut right = vec![];
        let mut center = vec![];
        for i in data
            .parse::<crate::parse::StyledString>()
            .unwrap()
            .into_content()
        {
            match i {
                StyledStringPart::Style(style) => {
                    fg = style
                        .foreground_color()
                        .into_color(self.config.foreground_color(), fg);
                    bg = style
                        .background_color()
                        .into_color(self.config.background_color(), bg);
                }
                StyledStringPart::String(string) => match align {
                    crate::parse::Align::Left => {
                        let width = self.get_width(&string) as usize;
                        left.push(Command {
                            fg,
                            bg,
                            string,
                            start: lcursor,
                            end: lcursor + width,
                        });
                        lcursor += width;
                    }
                    crate::parse::Align::Right => {
                        let width = self.get_width(&string) as usize;
                        right.push(Command {
                            fg,
                            bg,
                            string,
                            start: rcursor,
                            end: rcursor + width,
                        });
                        rcursor += width;
                    }
                    crate::parse::Align::Center => {
                        let width = self.get_width(&string) as usize;
                        center.push(Command {
                            fg,
                            bg,
                            string,
                            start: ccursor,
                            end: ccursor + width,
                        });
                        ccursor += width;
                    }
                },
                StyledStringPart::Action(_) => {} // Actions are irrelevant to rendering
                StyledStringPart::ActionEnd => {} // Actions are irrelevant to rendering
                StyledStringPart::Swap => {
                    std::mem::swap(&mut fg, &mut bg);
                }
                StyledStringPart::Align(align_) => {
                    align = align_;
                }
            }
        }
        let cmds = left
            .into_iter()
            .map(|v| v.into_offset(5))
            .chain(
                center
                    .into_iter()
                    .map(|v| v.into_offset((self.width as usize - ccursor) / 2)),
            )
            .chain(
                right
                    .into_iter()
                    .map(|v| v.into_offset(self.width as usize - 5 - rcursor)),
            );

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

            for i in cmds {
                let Command {
                    string,
                    fg,
                    bg,
                    start,
                    end: _,
                } = i;
                let text = crate::paint::Text::new(string, &font, fg, bg);
                text.paint(
                    &mut canvas
                        .slice(
                            start,
                            5,
                            self.width as usize - start,
                            self.height as usize - 5,
                        )
                        .unwrap(),
                )
                .unwrap();
            }
            #[cfg(feature = "logs")]
            log::info!("Painted");
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

    pub fn data(&self) -> Arc<Mutex<(String, bool)>> {
        self.data.clone()
    }

    pub fn condvar(&self) -> Arc<Condvar> {
        self.condvar.clone()
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
                    let splitted_content = self.parse_to_actions().unwrap();
                    let mut matched = None;
                    for (idx, content) in splitted_content.into_iter().enumerate() {
                        if (content.start..content.end).contains(&(event.position.0 as usize))
                            && crate::consts::wayland2bar(button)
                                .is_some_and(|v| v == content.button as u32)
                        {
                            let number = idx;
                            let action = content.cmd;
                            matched = Some((number, action));
                        }
                    }
                    if let Some((number, action)) = matched {
                        #[cfg(feature = "logs")]
                        log::info!(
                            "Pointer release key {} triggering action #{}",
                            button,
                            number
                        );
                        println!("{}", action);
                    } else {
                        #[cfg(feature = "logs")]
                        log::info!(
                            "Pointer release key {} triggering nothing at {}",
                            button,
                            event.position.0
                        );
                    }
                }
                PointerEventKind::Axis { vertical, .. } => {
                    let action = if vertical.discrete > 0 {
                        5
                    } else if vertical.discrete < 0 {
                        4
                    } else {
                        0
                    };
                    let splitted_content = self.parse_to_actions().unwrap();
                    let mut matched = None;
                    for (idx, content) in splitted_content.into_iter().enumerate() {
                        if (content.start..content.end).contains(&(event.position.0 as usize))
                            && action == content.button
                        {
                            let number = idx;
                            let action = content.cmd;
                            matched = Some((number, action));
                        }
                    }
                    if let Some((number, action)) = matched {
                        #[cfg(feature = "logs")]
                        log::info!(
                            "Mouse wheel rotating v {} triggering #{}",
                            vertical.discrete,
                            number,
                        );
                        println!("{}", action);
                    } else {
                        #[cfg(feature = "logs")]
                        log::info!(
                            "Mouse wheel rotating v {} triggering nothing",
                            vertical.discrete,
                        );
                    }
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
