use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor, delegate_data_device, delegate_output, delegate_seat, delegate_shm,
    delegate_xdg_shell,
    desktop::{Space, Window},
    input::{pointer::CursorImageStatus, SeatHandler, SeatState},
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::protocol::{wl_buffer, wl_seat, wl_surface::WlSurface},
    },
    utils::{Clock, Logical, Monotonic, Point, Serial},
    wayland::{
        buffer::BufferHandler,
        compositor::{with_states, CompositorHandler, CompositorState},
        data_device::{
            ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
        },
        output::OutputManagerState,
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
            XdgToplevelSurfaceData,
        },
        shm::{ShmHandler, ShmState},
    },
};

pub struct State {
    pub clock: Clock<Monotonic>,
    pub compositor_state: CompositorState,
    pub data_device_state: DataDeviceState,
    pub seat_state: SeatState<Self>,
    pub shm_state: ShmState,
    pub space: Space<Window>,
    pub cursor_status: CursorImageStatus,
    pub pointer_location: Point<f64, Logical>,
    pub output_manager_state: OutputManagerState,
    pub xdg_shell_state: XdgShellState,
}

impl State {
    pub fn refresh_geometry(&mut self) {
        let space = &mut self.space;

        // Get the first output available.
        let output = space.outputs().next().cloned().unwrap();

        // Find the size of the output.
        let output_geometry = space.output_geometry(&output).unwrap();
        let output_width = output_geometry.size.w;
        let output_height = output_geometry.size.h;

        // The gap between windows in px.
        let gap = 6;

        // The total number of windows.
        let elements_count = space.elements().count() as i32;

        // A vec to store the windows and their new sizes. This is used because space is a mutable
        // reference. The for loop will take ownership, meaning space.map_element can't be called
        // until the loop is finished.
        let mut resizes = vec![];

        for (i, window) in space.elements().enumerate() {
            // Move the window to start at the gap size creating a gap around the window.
            let (mut x, mut y) = (gap, gap);
            // The width/height should be subtracted from twice the gap size, since there are gaps
            // on both sides of the window.
            let (mut width, mut height) = (output_width - gap * 2, output_height - gap * 2);

            // If there is more than one window, subtract an additional gap from the width and
            // divide the width in two giving room for another window.
            if elements_count > 1 {
                width -= gap;
                width /= 2;
            }

            // Size the windows on the stack (the non-master windows).
            if i > 0 {
                // Get the height on the stack by dividing the height by the total number of
                // elements on the stack.
                height /= elements_count - 1;

                // Offset the x value by the width and gap.
                x += width + gap;
                // Offset the y value by the total number of windows above on the stack.
                y += height * (i as i32 - 1);
            }

            // Make all the windows on the stack, after the first one, have a gap on the top.
            if i > 1 {
                height -= gap;
                // By adding the gap to y, the window is pushed down, causing the gap.
                y += gap;
            }

            resizes.push((window.clone(), (width, height), (x, y)));
        }

        // Loop through the resizes vec and update the window state.
        for (window, dimensions, position) in resizes {
            // Resize the window to a suggested size. The client may not resize to this exact size,
            // for example a terminal emulator might resize to the closest size based on monospaced
            // rows and columns.
            window.toplevel().with_pending_state(|state| {
                state.size = Some(dimensions.into());
            });
            // Send a xdg_toplevel::configure event because of the state change.
            window.toplevel().send_configure();

            // Move window to new position.
            space.map_element(window, position, false);
        }
    }
}

// Implement required handlers for State. We call the delegate_* macro to automatically implement
// required traits from wayland_server.

impl BufferHandler for State {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    // Called on every buffer commit in Wayland to update a surface. This has the new state of the
    // surface.
    fn commit(&mut self, surface: &WlSurface) {
        // Let Smithay take the surface buffer so that desktop helpers get the new surface state.
        on_commit_buffer_handler(surface);

        // Find the window with the xdg toplevel surface to update.
        if let Some(window) = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface)
            .cloned()
        {
            // Refresh the window state.
            window.on_commit();

            // Find if the window has been configured yet.
            let initial_configure_sent = with_states(surface, |states| {
                states
                    .data_map
                    .get::<XdgToplevelSurfaceData>()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .initial_configure_sent
            });

            if !initial_configure_sent {
                // Configure window size/attributes.
                window.toplevel().send_configure();
            }
        }
    }
}
delegate_compositor!(State);

impl ClientDndGrabHandler for State {}
impl ServerDndGrabHandler for State {}

impl DataDeviceHandler for State {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}
delegate_data_device!(State);

impl SeatHandler for State {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn cursor_image(
        &mut self,
        _: &smithay::input::Seat<Self>,
        image: smithay::input::pointer::CursorImageStatus,
    ) {
        self.cursor_status = image;
    }

    fn focus_changed(&mut self, _: &smithay::input::Seat<Self>, _: Option<&WlSurface>) {}
}
delegate_seat!(State);

impl ShmHandler for State {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}
delegate_shm!(State);

impl XdgShellHandler for State {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        // Add the window to the space so we can use it elsewhere in our application, such as the
        // CompositorHandler.
        let window = Window::new(surface);
        self.space.map_element(window, (0, 0), false);

        // Resize and reposition all the windows.
        self.refresh_geometry();
    }

    fn new_popup(&mut self, _: PopupSurface, _: PositionerState) {}

    fn move_request(&mut self, _: ToplevelSurface, _: wl_seat::WlSeat, _: Serial) {}

    fn resize_request(
        &mut self,
        _: ToplevelSurface,
        _: wl_seat::WlSeat,
        _: Serial,
        _: xdg_toplevel::ResizeEdge,
    ) {
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {}
}
delegate_xdg_shell!(State);

delegate_output!(State);
