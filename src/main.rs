mod data;
mod state;

use smithay::{
    backend::{
        input::{Event, InputEvent, KeyState, KeyboardKeyEvent},
        renderer::{
            damage::DamageTrackedRenderer, element::surface::WaylandSurfaceRenderElement,
            gles2::Gles2Renderer,
        },
        winit::{self, WinitEvent},
    },
    desktop::{space::render_output, Space, Window},
    input::{
        keyboard::{keysyms, FilterResult},
        Seat, SeatState,
    },
    output,
    reexports::{
        calloop::{
            generic::Generic,
            timer::{TimeoutAction, Timer},
            EventLoop, Interest, Mode, PostAction,
        },
        wayland_server::Display,
    },
    utils::{Rectangle, Transform, SERIAL_COUNTER},
    wayland::{
        compositor::CompositorState, data_device::DataDeviceState, output::OutputManagerState,
        shell::xdg::XdgShellState, shm::ShmState, socket::ListeningSocketSource,
    },
};
use std::{os::unix::prelude::AsRawFd, sync::Arc, time::Duration};

fn main() -> anyhow::Result<(), anyhow::Error> {
    // Use calloop::EventLoop to process events from various sources
    let mut event_loop: EventLoop<data::Data> = EventLoop::try_new()?;

    // A struct that is used to store the state of the compositor, and manage a Backend to
    // receive events and dispatch messages
    let mut display: Display<state::State> = Display::new()?;

    // A Wayland ListeningSocket that implements calloop::EventSource and can be used as an
    // even source in an EventLoop. Wayland clients will connect to this socket to receive and
    // send events.
    let socket = ListeningSocketSource::new_auto()?;
    let socket_name = socket.socket_name().to_os_string();

    // Add Wayland socket to event loop
    // The EventLoop takes a source (socket), then a closure that produces an event, metadata, and
    // data. The event in this case is a UnixStream produced by the socket, no metadata, and the
    // data is the data::Data specified when created the event_loop variable.
    event_loop
        .handle()
        .insert_source(socket, |stream, _, data| {
            data.display
                .handle()
                .insert_client(stream, Arc::new(data::ClientData))
                .unwrap();
        })?;

    // Add Display to event loop
    // The EventLoop can take a Generic struct which is a struct containing a file descriptor that
    // calloop monitors for producing events. This file descriptor is created from winit below.
    // We only need to read for the fd, level triggering will report events whenever the
    // EventLoop polls.
    event_loop.handle().insert_source(
        Generic::new(
            display.backend().poll_fd().as_raw_fd(),
            Interest::READ,
            Mode::Level,
        ),
        |_, _, data| {
            // Dispatch requests received from clients to callbacks for clients. The callbacks will
            // probably need to access the current compositor state, so that is passed along.
            data.display.dispatch_clients(&mut data.state).unwrap();

            // Above the ListeningSocketSource handled the event loop by specifying PostAction.
            // Here we implement our own Generic event source, so we must return a
            // PostAction::Continue to tell the event loop to continue listening for events.
            Ok(PostAction::Continue)
        },
    )?;

    // Get the DisplayHandle which can be used to add Wayland clients, get clients,
    // create/disable/remove global objects, send events, etc.
    let dh = display.handle();

    // We will now add global objects to the display.

    // The compositor for our compositor.
    let compositor_state = CompositorState::new::<state::State>(&dh);
    // Shared memory buffer for sharing a pixel buffers with clients.
    let shm_state = ShmState::new::<state::State>(&dh, vec![]);
    // The output region (like a monitor or a region of space) for the compositor,
    // we use xdg for this.
    let output_manager_state = OutputManagerState::new_with_xdg_output::<state::State>(&dh);
    // Used for desktop applications, defines two types of Wayland surfaces, "toplevel" (for the
    // main application area) and "popup" (for dialogs/tooltips/etc).
    let xdg_shell_state = XdgShellState::new::<state::State>(&dh);
    // A seat is a group of input devices like keyboards, pointers, etc. This manages the seat
    // state.
    let mut seat_state = SeatState::<state::State>::new();
    // A space to map windows on.
    let space = Space::<Window>::default();
    // Manage copy/paste and drag-and-drop from inputs.
    let data_device_state = DataDeviceState::new::<state::State>(&dh);

    // Create a new seat from the seat state, we pass in a name .
    let mut seat: Seat<state::State> = seat_state.new_wl_seat(&dh, "mwm_seat");
    // Add a keyboard with repeat rate and delay in milliseconds. The repeat is the time to
    // repeat, then delay is how long to wait until the next repeat.
    seat.add_keyboard(Default::default(), 500, 500)?;
    // Add pointer to seat.
    seat.add_pointer();

    // Create the state of our compositor, store all the global objects we use so we can access
    // them in our application.
    let state = state::State {
        compositor_state,
        data_device_state,
        seat_state,
        shm_state,
        space,
        output_manager_state,
        xdg_shell_state,
    };

    // The data stored in EventLoop, we need access to the Display and compositor state.
    let mut data = data::Data { state, display };

    // Use winit, which is a library for handling windows and their events. Use OpenGL ES 2.0 as
    // the renderer.
    let (mut backend, mut winit) = winit::init::<Gles2Renderer>().unwrap();

    // Get size of winit window.
    let size = backend.window_size().physical_size;

    // Specify the output to be the size of winit window and the refresh rate in millihertz.
    let mode = output::Mode {
        size,
        // 60 fps
        refresh: 60_000,
    };

    // Tells the client what the physical properties of the output are.
    // We don't set correct state until we add the physical properties to an output.
    let physical_properties = output::PhysicalProperties {
        // Size in mm
        size: (0, 0).into(),
        // How the physical pixels are organized, (like HorizontalRgb vs VerticalBgr). Just leave
        // as unknown for normal outputs.
        subpixel: output::Subpixel::Unknown,
        make: "mwm".into(),
        model: "Winit".into(),
    };

    // Create a new output
    let output = output::Output::new("winit".to_string(), physical_properties);
    // Clients can access the global objects to get the physical properties.
    output.create_global::<state::State>(&data.display.handle());
    // Set the state to use winit.
    output.change_current_state(
        // Contains size/refresh from winit.
        Some(mode),
        // Wayland starts upside down, so flip it.
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );
    // Set the prefereed mode to use.
    output.set_preferred(mode);
    // Set the output of a space with coordinates for the upper left corner of the surface.
    data.state.space.map_output(&output, (0, 0));

    // Set the enviroment variable that Wayland clients can use. They get the socket and connect to
    // it.
    std::env::set_var("WAYLAND_DISPLAY", &socket_name);

    // Create a timer and start time for the EventLoop.
    // TODO: Use ping for a tighter event loop.
    let start_time = std::time::Instant::now();
    let timer = Timer::immediate();

    // Track parts of the output that are "damaged". When a window draws on a new part
    // of the surface, that section must be redrawn.
    let mut damage_tracked_renderer = DamageTrackedRenderer::from_output(&output);

    // Create a event loop with a timer, pump event loop by returning a Duration.
    event_loop
        .handle()
        .insert_source(timer, move |_, _, data| {
            let display = &mut data.display;
            let state = &mut data.state;

            // Process events from winit event loop
            winit
                .dispatch_new_events(|event| {
                    if let WinitEvent::Input(event) = event {
                        if let InputEvent::Keyboard { event } = event {
                            // If we received a keyboard event, get the keyboard from the seat
                            // and process a key input.
                            let serial = SERIAL_COUNTER.next_serial();
                            let time = Event::time_msec(&event);
                            let press_state = event.state();
                            let action = seat.get_keyboard().unwrap().input::<u8, _>(
                                state,
                                event.key_code(),
                                press_state,
                                serial,
                                time,
                                |_, _, keysym| {
                                    // If the user pressed the letter T, return the action value of
                                    // 1.
                                    if press_state == KeyState::Pressed
                                        && keysym.modified_sym() == keysyms::KEY_t | keysyms::KEY_T
                                    {
                                        FilterResult::Intercept(1)
                                    } else {
                                        FilterResult::Forward
                                    }
                                },
                            );

                            // If the action equals 1, spawn a weston-terminal.
                            if Some(1) == action {
                                std::process::Command::new("weston-terminal")
                                    .spawn()
                                    .unwrap();
                            }
                        }
                    }
                })
                .unwrap();

            // Create a damage area the size of the backend output.
            let size = backend.window_size().physical_size;
            let damage = Rectangle::from_loc_and_size((0, 0), size);

            backend.bind().unwrap();

            // Render output by providing backend renderer, the output, the space, and the
            // damage_tracked_renderer for tracking where the surface is damaged.
            render_output::<_, WaylandSurfaceRenderElement<Gles2Renderer>, _, _>(
                &output,
                backend.renderer(),
                0,
                [&state.space],
                &[],
                &mut damage_tracked_renderer,
                [0.1, 0.1, 0.1, 1.0],
            )
            .unwrap();

            // Submit the back buffer to the display, causing the surface to be redrawn.
            backend.submit(Some(&[damage])).unwrap();

            // For each of the windows send the frame callbacks to windows telling them to draw.
            state.space.elements().for_each(|window| {
                window.send_frame(
                    &output,
                    start_time.elapsed(),
                    Some(Duration::ZERO),
                    |_, _| Some(output.clone()),
                )
            });

            // Refresh space state and handle certain events like enter/leave for outputs/windows.
            state.space.refresh();

            // Flush the outgoing buffers containing events so the clients get them.
            display.flush_clients().unwrap();

            // Wait 16 milliseconds until next event.
            TimeoutAction::ToDuration(Duration::from_millis(16))
        })
        .unwrap();

    // Run the event loop
    event_loop.run(None, &mut data, |_| {})?;

    Ok(())
}
