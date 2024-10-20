use sctk::compositor::{self, CompositorHandler};
use sctk::output::{self, OutputHandler};
use sctk::registry::ProvidesRegistryState;
use sctk::shell::xdg::window::{self, Window, WindowConfigure};
use sctk::shell::{xdg, WaylandSurface};
use smithay_client_toolkit as sctk;
use wayland_client::protocol::wl_output::{Transform, WlOutput};
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{protocol, Connection, QueueHandle};

#[derive(Debug)]
struct Status {
    compositor_state: compositor::CompositorState,
    shell_state: xdg::XdgShell,
    registry: sctk::registry::RegistryState,
    output_state: output::OutputState,
    shm: sctk::shm::Shm,
    pool: sctk::shm::slot::SlotPool,
    buffer: Option<sctk::shm::slot::Buffer>,
    width: u32,
    height: u32,
    req_exit: bool,
    window: Window,
    queue_handler: QueueHandle<Status>,
}

impl ProvidesRegistryState for Status {
    fn registry(&mut self) -> &mut sctk::registry::RegistryState {
        &mut self.registry
    }

    sctk::registry_handlers!();
}

impl OutputHandler for Status {
    fn output_state(&mut self) -> &mut output::OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, conn: &Connection, qh: &QueueHandle<Self>, output: WlOutput) {}

    fn update_output(&mut self, conn: &Connection, qh: &QueueHandle<Self>, output: WlOutput) {}

    fn output_destroyed(&mut self, conn: &Connection, qh: &QueueHandle<Self>, output: WlOutput) {}
}

impl CompositorHandler for Status {
    fn scale_factor_changed(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &WlSurface,
        new_factor: i32,
    ) {
        // Ignored
    }

    fn transform_changed(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &WlSurface,
        new_transform: Transform,
    ) {
        // Ignored
    }

    fn frame(&mut self, conn: &Connection, qh: &QueueHandle<Self>, surface: &WlSurface, time: u32) {
        // TODO
    }

    fn surface_enter(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &WlSurface,
        output: &WlOutput,
    ) {
        // Ignored
    }

    fn surface_leave(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &WlSurface,
        output: &WlOutput,
    ) {
        // Ignored
    }
}

impl window::WindowHandler for Status {
    fn request_close(&mut self, conn: &Connection, qh: &QueueHandle<Self>, window: &Window) {
        self.req_exit = true;
    }

    fn configure(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        window: &Window,
        configure: WindowConfigure,
        serial: u32,
    ) {
        let width = self.width;
        let height = self.height;
        let stride = self.width * 4;
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
        {
            let shift = 0;
            canvas
                .chunks_exact_mut(4)
                .enumerate()
                .for_each(|(index, chunk)| {
                    let x = ((index + shift as usize) % width as usize) as u32;
                    let y = (index / width as usize) as u32;

                    let a = 0xff;
                    let r = u32::min(((width - x) * 0xFF) / width, ((height - y) * 0xFF) / height);
                    let g = u32::min((x * 0xFF) / width, ((height - y) * 0xFF) / height);
                    let b = u32::min(((width - x) * 0xFF) / width, (y * 0xFF) / height);
                    let color: u32 = (a << 24) & (r << 26) + (g << 8) + b;
                    let array: &mut [u8; 4] = chunk.try_into().unwrap();
                    *array = color.to_le_bytes();
                });
        }

        self.window
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);

        self.window
            .wl_surface()
            .frame(&self.queue_handler, self.window.wl_surface().clone());

        buffer.attach_to(self.window.wl_surface()).unwrap();
        self.window.commit();
    }
}

impl sctk::shm::ShmHandler for Status {
    fn shm_state(&mut self) -> &mut sctk::shm::Shm {
        &mut self.shm
    }
}

sctk::delegate_registry!(Status);
sctk::delegate_compositor!(Status);
sctk::delegate_xdg_shell!(Status);
sctk::delegate_xdg_window!(Status);
sctk::delegate_output!(Status);
sctk::delegate_shm!(Status);

fn main() {
    env_logger::init();

    let conn = Connection::connect_to_env().unwrap();

    let (globals, mut event_queue) = wayland_client::globals::registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let compositor_state = compositor::CompositorState::bind(&globals, &qh).unwrap();
    let shell_state = xdg::XdgShell::bind(&globals, &qh).unwrap();
    let shm = sctk::shm::Shm::bind(&globals, &qh).unwrap();

    let surface = compositor_state.create_surface(&qh);
    let window = shell_state.create_window(surface, window::WindowDecorations::ServerDefault, &qh);
    window.set_app_id("CraBar");
    window.set_min_size(Some((256, 256)));
    window.commit();

    let pool = sctk::shm::slot::SlotPool::new(262144, &shm).unwrap();

    let mut state = Status {
        compositor_state,
        shell_state,
        output_state: output::OutputState::new(&globals, &qh),
        registry: sctk::registry::RegistryState::new(&globals),
        req_exit: false,
        pool,
        shm,
        width: 256,
        height: 256,
        buffer: None,
        window,
        queue_handler: qh,
    };

    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();
        if state.req_exit {
            std::process::exit(0);
        }
    }
}
