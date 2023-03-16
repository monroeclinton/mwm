mod data;
mod state;

use smithay::{
    backend::{
        input::{Event, InputEvent, KeyboardKeyEvent},
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
    let mut event_loop: EventLoop<data::Data> = EventLoop::try_new()?;
    let mut display: Display<state::State> = Display::new()?;

    let socket = ListeningSocketSource::new_auto()?;
    let socket_name = socket.socket_name().to_os_string();

    // Add wayland socket to event loop
    event_loop
        .handle()
        .insert_source(socket, |stream, _, data| {
            data.display
                .handle()
                .insert_client(stream, Arc::new(data::ClientData))
                .unwrap();
        })?;

    // Add display to event loop
    event_loop.handle().insert_source(
        Generic::new(
            display.backend().poll_fd().as_raw_fd(),
            Interest::READ,
            Mode::Level,
        ),
        |_, _, data| {
            data.display.dispatch_clients(&mut data.state).unwrap();

            Ok(PostAction::Continue)
        },
    )?;

    let dh = display.handle();
    let compositor_state = CompositorState::new::<state::State>(&dh);
    let shm_state = ShmState::new::<state::State>(&dh, vec![]);
    let output_manager_state = OutputManagerState::new_with_xdg_output::<state::State>(&dh);
    let xdg_shell_state = XdgShellState::new::<state::State>(&dh);
    let mut seat_state = SeatState::<state::State>::new();
    let space = Space::<Window>::default();
    let data_device_state = DataDeviceState::new::<state::State>(&dh);

    let mut seat: Seat<state::State> = seat_state.new_wl_seat(&dh, "mwm_seat");
    seat.add_keyboard(Default::default(), 500, 500)?;
    seat.add_pointer();

    let state = state::State {
        compositor_state,
        data_device_state,
        seat_state,
        shm_state,
        space,
        output_manager_state,
        xdg_shell_state,
    };

    let mut data = data::Data { state, display };

    let (mut backend, mut winit) = winit::init::<Gles2Renderer>().unwrap();

    let size = backend.window_size().physical_size;

    let mode = output::Mode {
        size,
        refresh: 60_000,
    };

    let physical_properties = output::PhysicalProperties {
        size: (0, 0).into(),
        subpixel: output::Subpixel::Unknown,
        make: "mwm".into(),
        model: "Winit".into(),
    };

    let output = output::Output::new("winit".to_string(), physical_properties);
    output.create_global::<state::State>(&data.display.handle());
    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );
    output.set_preferred(mode);
    data.state.space.map_output(&output, (0, 0));

    std::env::set_var("WAYLAND_DISPLAY", &socket_name);

    let start_time = std::time::Instant::now();
    let timer = Timer::immediate();
    let mut damage_tracked_renderer = DamageTrackedRenderer::from_output(&output);

    event_loop
        .handle()
        .insert_source(timer, move |_, _, data| {
            let display = &mut data.display;
            let state = &mut data.state;

            winit
                .dispatch_new_events(|event| {
                    if let WinitEvent::Input(event) = event {
                        if let InputEvent::Keyboard { event } = event {
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
                                    if keysym.modified_sym() == keysyms::KEY_t | keysyms::KEY_T {
                                        FilterResult::Intercept(1)
                                    } else {
                                        FilterResult::Forward
                                    }
                                },
                            );

                            if Some(1) == action {
                                std::process::Command::new("weston-terminal")
                                    .spawn()
                                    .unwrap();
                            }
                        }
                    }
                })
                .unwrap();

            let size = backend.window_size().physical_size;
            let damage = Rectangle::from_loc_and_size((0, 0), size);

            backend.bind().unwrap();

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

            backend.submit(Some(&[damage])).unwrap();

            state.space.elements().for_each(|window| {
                window.send_frame(
                    &output,
                    start_time.elapsed(),
                    Some(Duration::ZERO),
                    |_, _| Some(output.clone()),
                )
            });

            state.space.refresh();
            display.flush_clients().unwrap();

            TimeoutAction::ToDuration(Duration::from_millis(16))
        })
        .unwrap();

    event_loop.run(None, &mut data, |_| {})?;

    Ok(())
}
