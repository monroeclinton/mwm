use crate::config::Config;

pub struct Surface {
    surface: cairo::XCBSurface,
}

impl Surface {
    pub fn new(surface: cairo::XCBSurface) -> Self {
        Self {
            surface,
        }
    }

    pub fn flush(&self) {
        self.surface.flush();
    }

    pub fn clear_surface(&mut self, config: &Config) {
        let context = cairo::Context::new(&self.surface)
            .expect("Unable to find context of surface.");

        set_source_rgb(&context, config.background_color);
        context.paint().expect("Unable to clear surface.");
    }

    pub fn draw_title(
        &mut self,
        config: &Config,
        window_height: f64,
        item_height: f64,
  ) {
        let font_size = config.font_size as f64;
        let title = "Command:";

        self.surface.set_size(config.width as i32, window_height as i32)
            .expect("Unable to resize surface.");

        let context = cairo::Context::new(&self.surface)
            .expect("Unable to find context of surface.");

        let font_face = cairo::FontFace::toy_create(
            config.font_family.as_str(),
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal
        ).expect("Unable to create font face.");

        let extents = context.text_extents(title)
            .expect("Unable to find text extents of title.");

        let title_width = extents.width * 2.0 + font_size;

        set_source_rgb(&context, config.background_active_color);

        context.rectangle(
            0.0,
            0.0,
            title_width,
            item_height,
        );

        context.fill()
            .expect("Unable to draw input box.");

        set_source_rgb(&context, config.font_active_color);

        context.set_font_face(&font_face);
        context.set_font_size(font_size);
        context.move_to(
            font_size,
            (item_height + extents.height) / 2.0
        );

        context.show_text(title)
            .expect("Cannot position title text.");
    }

    pub fn draw_items(
        &self, 
        commands: &Vec<String>, 
        config: &Config,
        item_height: f64,
        selection_index: usize,
    ) {
        let font_size = config.font_size as f64;
        let window_width = config.width as f64;

        let context = cairo::Context::new(&self.surface)
            .expect("Unable to find context of launcher surface.");

        let font_face = cairo::FontFace::toy_create(
            config.font_family.as_str(),
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal
        ).expect("Unable to create font face.");

        context.set_font_face(&font_face);
        context.set_font_size(font_size);

        for (i, command) in commands.iter().enumerate() {
            if i == selection_index {
                set_source_rgb(&context, config.background_active_color);

                context.rectangle(
                    0.0,
                    item_height * (i + 1) as f64,
                    window_width,
                    item_height,
                );

                context.fill()
                    .expect("Unable to draw selection box.");

                set_source_rgb(&context, config.font_active_color);
            } else {
                set_source_rgb(&context, config.font_color);
            }

            let extents = context.text_extents(command.as_str())
                .expect("Unable to find text extents of command.");

            context.move_to(
                font_size,
                (item_height * (i + 2) as f64) - extents.height / 2.0
            );

            context.show_text(command.as_str())
                .expect("Cannot position command text.");
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
