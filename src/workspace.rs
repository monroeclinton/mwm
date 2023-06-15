use smithay::desktop::{Space, Window};

struct Workspace {
    windows: Vec<Window>,
    active_window: Option<usize>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            active_window: None,
        }
    }
}

pub struct Workspaces {
    workspaces: Vec<Workspace>,
    active_workspace: usize,
    previous_workspace: usize,
}

impl Workspaces {
    pub fn new() -> Self {
        Self {
            workspaces: (0..=8).map(|_| Workspace::new()).collect(),
            active_workspace: 0,
            previous_workspace: 0,
        }
    }

    pub fn active(&self) -> usize {
        return self.active_workspace;
    }

    pub fn set_active(&mut self, workspace: usize, space: &mut Space<Window>) {
        self.previous_workspace = self.active_workspace;
        self.active_workspace = workspace;
        self.refresh_geometry(space);
    }

    pub fn set_active_window(&mut self, window: Window) {
        let mut workspace = &mut self.workspaces[self.active_workspace];
        workspace.active_window = workspace.windows.iter().position(|w| w == &window);
    }

    pub fn insert_window(&mut self, workspace: usize, window: Window) {
        self.workspaces[workspace].windows.push(window.clone());
    }

    pub fn move_window(&mut self, workspace: usize, space: &mut Space<Window>) {
        if let Some(active_window) = self.workspaces[self.active_workspace].active_window {
            let window = self.workspaces[self.active_workspace].windows[active_window].clone();

            // Remove window from active workspace.
            self.workspaces[self.active_workspace]
                .windows
                .retain(|w| w != &window);
            self.insert_window(workspace, window);
            // Insert and update layout of windows.
            self.refresh_geometry(space);
        }
    }

    pub fn refresh_geometry(&mut self, space: &mut Space<Window>) {
        // Remove dead elements.
        space.refresh();

        // Hide the previous active workspace.
        self.workspaces[self.previous_workspace]
            .windows
            .iter()
            .for_each(|window| space.unmap_elem(window));

        // Get the first output available.
        let output = space.outputs().next().cloned().unwrap();

        // Find the size of the output.
        let output_geometry = space.output_geometry(&output).unwrap();
        let output_width = output_geometry.size.w;
        let output_height = output_geometry.size.h;

        // The gap between windows in px.
        let gap = 6;

        // Get windows from active workspace.
        let windows = &mut self.workspaces[self.active_workspace].windows;

        // The total number of windows.
        let elements_count = windows.len() as i32;

        for (i, window) in windows.iter().enumerate() {
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

            // Resize the window to a suggested size. The client may not resize to this exact size,
            // for example a terminal emulator might resize to the closest size based on monospaced
            // rows and columns.
            window.toplevel().with_pending_state(|state| {
                state.size = Some((width, height).into());
            });
            // Send a xdg_toplevel::configure event because of the state change.
            window.toplevel().send_configure();

            // Move window to new position.
            space.map_element(window.clone(), (x, y), false);
        }
    }
}