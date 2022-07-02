use crate::config::Config;
use systemstat::{System, Platform, saturating_sub_bytes};

pub struct Surface {
    context: cairo::Context,
    surface: cairo::XCBSurface,
    bar_width: f64,
    bar_height: f64,
}

impl Surface {
    pub fn new(
        surface: cairo::XCBSurface,
        bar_width: f64,
        bar_height: f64,
    ) -> Self {
        let context = cairo::Context::new(&surface)
            .expect("Unable to find context of surface.");

        Self {
            context,
            surface,
            bar_width,
            bar_height
        }
    }

    pub fn flush(&self) {
        self.surface.flush();
    }

    pub fn clear_surface(&mut self, config: &Config) {
        let context = &self.context;
        set_source_rgb(context, config.background_color);
        context.paint().expect("Unable to clear surface.");
    }


    pub fn workspaces(&self, config: &Config, workspaces: Vec<&str>, active_workspace: usize) {
        let context = &self.context;

        let font_face = cairo::FontFace::toy_create(
            config.font_family.as_str(),
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal
        ).expect("Unable to create font face in statusbar.");

        let bar_height = config.height as f64;
        let workspace_width = config.workspace_width as f64;
        let mut offset = 0.0;

        let mut workspace_index = 1;
        for workspace in workspaces {
            if workspace_index == active_workspace {
                set_source_rgb(&context, config.background_active_color);

                context.rectangle(
                    offset,
                    0.0,
                    workspace_width,
                    bar_height
                );

                context.fill()
                    .expect("Unable to create active rectangle.");
            }

            let extents = context.text_extents(workspace)
                .expect("Unable to find text text extents of statusbar workspace.");

            context.set_font_face(&font_face);
            context.set_font_size(config.font_size as f64);

            if workspace_index == active_workspace {
                set_source_rgb(&context, config.font_active_color);
            } else {
                set_source_rgb(&context, config.font_color);
            }

            context.move_to(
                offset + ((workspace_width - extents.width) / 2.0),
                (bar_height + extents.height) / 2.0
            );
            context.show_text(workspace)
                .expect("Cannot position text on surface in statusbar.");

            offset += workspace_width;
            workspace_index += 1;
        }
    }

    pub fn bar_title(&self, config: &Config, window_name: Option<String>) {
        let title = if let Some(name) = window_name {
            format!("[{}] {}@{}", name, whoami::username(), whoami::hostname())
        } else {
            format!("{}@{}", whoami::username(), whoami::hostname())
        };

        let context = &self.context;

        let font_face = cairo::FontFace::toy_create(
            config.font_family.as_str(),
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal
        ).expect("Unable to create font face in statusbar.");

        let extents = context.text_extents(title.as_str())
            .expect("Unable to find text text extents of statusbar workspace.");

        context.set_font_face(&font_face);
        context.set_font_size(config.font_size as f64);
        context.move_to(
            (self.bar_width - extents.width) / 2.0,
            (self.bar_height + extents.height) / 2.0
        );

        set_source_rgb(&context, config.font_color);

        context.show_text(title.as_str())
            .expect("Cannot position text on surface in statusbar.");
    }

    pub fn draw_info(&self, config: &Config) {
        let sys = System::new();

        let memory = match sys.memory() {
            Ok(mem) => {
                let used_memory = saturating_sub_bytes(mem.total, mem.free);
                format!(
                    "Mem: {} ({:.0}% used)",
                    used_memory,
                    (used_memory.as_u64() as f64 / mem.total.as_u64() as f64) * 100.0,
                )
            },
            _ => format!("Mem: Memory error.")
        };

        let date = format!(
            "Time: {}",
            chrono::Local::now().format("%b %d (%a) %I:%M%p")
        );

        let blocks = vec![memory, date];

        let context = &self.context;

        let font_face = cairo::FontFace::toy_create(
            config.font_family.as_str(),
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal
        ).expect("Unable to create font face in statusbar.");

        let mut offset= 0.0;

        for text in blocks {
            let extents = context.text_extents(text.as_str())
                .expect("Unable to find text text extents of statusbar workspace.");

            context.set_font_face(&font_face);
            context.set_font_size(config.font_size as f64);
            context.move_to(
                self.bar_width - offset - extents.width,
                (self.bar_height + extents.height) / 2.0
            );

            set_source_rgb(&context, config.font_color);

            context.show_text(text.as_str())
                .expect("Cannot position text on surface in statusbar.");

            offset += extents.width + 20.0;
        }
    }
}

fn set_source_rgb(context: &cairo::Context, color: u32) {
    context.set_source_rgb(
        (color >> 16 & 255) as f64 / 255.0,
        (color >> 8 & 255) as f64 / 255.0,
        (color & 255) as f64 / 255.0
    );
}
