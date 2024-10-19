use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::registry::ProvidesRegistryState;

#[derive(Debug)]
struct Status {
    compositor_state: smithay_client_toolkit::compositor::CompositorState,
    shell_state: smithay_client_toolkit::shell::xdg::XdgShell,
    registry: smithay_client_toolkit::registry::RegistryState,
    output_state: smithay_client_toolkit::output::OutputState,
    req_exit: bool
}

impl ProvidesRegistryState for Status {
    fn registry(&mut self) -> &mut smithay_client_toolkit::registry::RegistryState {
        &mut self.registry
    }

    smithay_client_toolkit::registry_handlers!();
}

impl smithay_client_toolkit::output::OutputHandler for Status {
    fn output_state(&mut self) -> &mut smithay_client_toolkit::output::OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }
}

impl smithay_client_toolkit::compositor::CompositorHandler for Status {
    fn scale_factor_changed(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        surface: &wayland_client::protocol::wl_surface::WlSurface,
        new_factor: i32,
    ) {
        // Ignored
    }

    fn transform_changed(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        surface: &wayland_client::protocol::wl_surface::WlSurface,
        new_transform: wayland_client::protocol::wl_output::Transform,
    ) {
        // Ignored
    }

    fn frame(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        surface: &wayland_client::protocol::wl_surface::WlSurface,
        time: u32,
    ) {
        // TODO
    }

    fn surface_enter(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        surface: &wayland_client::protocol::wl_surface::WlSurface,
        output: &wayland_client::protocol::wl_output::WlOutput,
    ) {
        // Ignored
    }

    fn surface_leave(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        surface: &wayland_client::protocol::wl_surface::WlSurface,
        output: &wayland_client::protocol::wl_output::WlOutput,
    ) {
        // Ignored
    }
}

impl smithay_client_toolkit::shell::xdg::window::WindowHandler for Status {
    fn request_close(&mut self, conn: &wayland_client::Connection, qh: &wayland_client::QueueHandle<Self>, window: &smithay_client_toolkit::shell::xdg::window::Window) {
        self.req_exit = true;
    }

    fn configure(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        window: &smithay_client_toolkit::shell::xdg::window::Window,
        configure: smithay_client_toolkit::shell::xdg::window::WindowConfigure,
        serial: u32,
    ) {
        // TODO
    }
    // add code here
}

smithay_client_toolkit::delegate_registry!(Status);
smithay_client_toolkit::delegate_compositor!(Status);
smithay_client_toolkit::delegate_xdg_shell!(Status);
smithay_client_toolkit::delegate_xdg_window!(Status);
smithay_client_toolkit::delegate_output!(Status);

fn main() {
    let conn = wayland_client::Connection::connect_to_env().unwrap();

    let (globals, mut event_queue) = wayland_client::globals::registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let mut state = Status {
        compositor_state: smithay_client_toolkit::compositor::CompositorState::bind(&globals, &qh).unwrap(),
        shell_state: smithay_client_toolkit::shell::xdg::XdgShell::bind(&globals, &qh).unwrap(),
        output_state: smithay_client_toolkit::output::OutputState::new(&globals, &qh),
        registry: smithay_client_toolkit::registry::RegistryState::new(&globals),
        req_exit: false,
    };

    let surface = state.compositor_state.create_surface(&qh);

    let window = state.shell_state.create_window(surface, smithay_client_toolkit::shell::xdg::window::WindowDecorations::ServerDefault, &qh);
    window.set_app_id("CraBar");
    window.commit();

    loop {
        event_queue.dispatch_pending(&mut state);
        if state.req_exit {
            std::process::exit(0);
        }
    }
}
