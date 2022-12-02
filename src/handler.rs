use crate::event::EventContext;
use crate::plugin::PluginHandler;
use crate::plugins;

pub struct Handler {
    plugins: Vec<Box<dyn PluginHandler>>,
}

impl Default for Handler {
    fn default() -> Self {
        Self {
            plugins: vec![
                Box::new(plugins::Commands::default()),
                Box::new(plugins::ConfigureWindow::default()),
                Box::new(plugins::DestroyWindow::default()),
                Box::new(plugins::MapWindow::default()),
                Box::new(plugins::UnmapWindow::default()),
                Box::new(plugins::WindowSelector::default()),
                Box::new(plugins::WindowSizer::default()),
                Box::new(plugins::Workspaces::default()),
            ],
        }
    }
}

impl Handler {
    #[tracing::instrument(skip_all, fields(window = ectx.event.window()))]
    pub fn on_client_message(&mut self, ectx: EventContext<xcb::ClientMessageEvent>) {
        self.plugins
            .iter_mut()
            .for_each(|plugin| plugin.on_client_message(ectx.clone()).unwrap());
    }

    #[tracing::instrument(
        skip_all,
        fields(
            event_window = ectx.event.event(),
            key_code = ectx.event.detail(),
            state = ectx.event.state()
        )
    )]
    pub fn on_key_press(&mut self, ectx: EventContext<xcb::KeyPressEvent>) {
        self.plugins
            .iter_mut()
            .for_each(|plugin| plugin.on_key_press(ectx.clone()).unwrap());
    }

    #[tracing::instrument(
        skip_all,
        fields(
            window = ectx.event.window(),
            width = ectx.event.width(),
            height = ectx.event.height(),
            x = ectx.event.x(),
            y = ectx.event.y()
        )
    )]
    pub fn on_configure_request(&mut self, ectx: EventContext<xcb::ConfigureRequestEvent>) {
        self.plugins
            .iter_mut()
            .for_each(|plugin| plugin.on_configure_request(ectx.clone()).unwrap());
    }

    #[tracing::instrument(skip_all, fields(window = ectx.event.window()))]
    pub fn on_map_request(&mut self, ectx: EventContext<xcb::MapRequestEvent>) {
        self.plugins
            .iter_mut()
            .for_each(|plugin| plugin.on_map_request(ectx.clone()).unwrap());
    }

    #[tracing::instrument(
        skip_all,
        fields(
            window = ectx.event.window(),
            atom = ectx.event.atom(),
        )
    )]
    pub fn on_property_notify(&mut self, ectx: EventContext<xcb::PropertyNotifyEvent>) {
        self.plugins
            .iter_mut()
            .for_each(|plugin| plugin.on_property_notify(ectx.clone()).unwrap());
    }

    #[tracing::instrument(
        skip_all,
        fields(
            event_window = ectx.event.event(),
            state = ectx.event.state(),
            mode = ectx.event.mode(),
        )
    )]
    pub fn on_enter_notify(&mut self, ectx: EventContext<xcb::EnterNotifyEvent>) {
        self.plugins
            .iter_mut()
            .for_each(|plugin| plugin.on_enter_notify(ectx.clone()).unwrap());
    }

    #[tracing::instrument(skip_all, fields(window = ectx.event.window()))]
    pub fn on_unmap_notify(&mut self, ectx: EventContext<xcb::UnmapNotifyEvent>) {
        self.plugins
            .iter_mut()
            .for_each(|plugin| plugin.on_unmap_notify(ectx.clone()).unwrap());
    }

    #[tracing::instrument(skip_all, fields(window = ectx.event.window()))]
    pub fn on_destroy_notify(&mut self, ectx: EventContext<xcb::DestroyNotifyEvent>) {
        self.plugins
            .iter_mut()
            .for_each(|plugin| plugin.on_destroy_notify(ectx.clone()).unwrap());
    }
}
