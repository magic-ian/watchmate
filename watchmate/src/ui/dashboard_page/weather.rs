use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use infinitime::{bt, fdo::weather as weather_provider, zbus};
use relm4::{gtk, Component, ComponentParts, ComponentSender, JoinHandle, RelmWidgetExt};
use std::sync::Arc;

#[derive(Debug)]
pub enum Input {
    Device(Option<Arc<bt::InfiniTime>>),
    WeatherUpdateSessionStart,
    WeatherUpdateSessionEnded,
    ProviderAdded(String),
    ProviderRemoved(String),
}

#[derive(Debug)]
pub enum CommandOutput {
    None,
    DBusConnection(zbus::Connection),
}

#[derive(Default)]
pub struct Model {
    provider_names: gtk::StringList,
    infinitime: Option<Arc<bt::InfiniTime>>,
    update_task: Option<JoinHandle<()>>,
    dbus_session: Option<Arc<zbus::Connection>>,
    dropdown: gtk::DropDown,
}

impl Model {
    fn stop_update_task(&mut self) {
        if self.update_task.take().map(|h| h.abort()).is_some() {
            log::info!("Weather Update session stopped");
        }
    }
}

#[relm4::component(pub)]
impl Component for Model {
    type CommandOutput = CommandOutput;
    type Init = ();
    type Input = Input;
    type Output = ();
    type Widgets = Widgets;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_margin_all: 12,
            set_spacing: 10,

            gtk::Label {
                set_label: "Weather Provider",
                set_halign: gtk::Align::Start,
            },

            if model.provider_names.n_items() == 0 {
                gtk::Label {
                    set_label: "Not configured",
                    set_hexpand: true,
                    set_halign: gtk::Align::End,
                    add_css_class: "dim-label",
                }
            } else {
                #[local]
                dropdown -> gtk::DropDown {
                    set_hexpand: true,
                    #[watch]
                    set_model: Some(&model.provider_names),
                    connect_selected_notify => Input::WeatherUpdateSessionStart,
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let dropdown = gtk::DropDown::default();
        let model = Self {
            dropdown: dropdown.clone(),
            ..Default::default()
        };
        let widgets = view_output!();
        sender.oneshot_command(async move {
            match zbus::Connection::session().await {
                Ok(connection) => CommandOutput::DBusConnection(connection),
                Err(error) => {
                    log::error!("Failed to establish D-Bus session connection: {error}");
                    CommandOutput::None
                }
            }
        });
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            Input::Device(infinitime) => {
                self.infinitime = infinitime;
                match self.infinitime {
                    Some(_) => sender.input(Input::WeatherUpdateSessionStart),
                    None => self.stop_update_task(),
                }
            }
            Input::WeatherUpdateSessionStart => {
                if let Some(infinitime) = self.infinitime.clone() {
                    if let Some(dbus_session) = self.dbus_session.clone() {
                        self.stop_update_task();
                        let task_handle = relm4::spawn(async move {
                            // Discover weather providers
                            match weather_provider::discover_weather_providers(&dbus_session).await {
                                Ok(providers) => {
                                    for provider in providers {
                                        sender.input(Input::ProviderAdded(provider));
                                    }
                                    // Note: Real implementation would also set up a stream
                                    // to watch for providers being added/removed
                                    log::info!("Weather provider discovery completed");
                                }
                                Err(error) => {
                                    log::error!("Failed to discover weather providers: {error}");
                                }
                            }
                            // Keep infinitime in scope to prevent unused warning
                            let _ = infinitime;
                            sender.input(Input::WeatherUpdateSessionEnded);
                        });
                        self.update_task = Some(task_handle);
                    }
                }
            }
            Input::WeatherUpdateSessionEnded => {
                log::info!("Weather update session ended");
                self.update_task = None;
            }
            Input::ProviderAdded(name) => {
                self.provider_names.append(&name);
                log::info!("Weather provider discovered: {name}");
            }
            Input::ProviderRemoved(name) => {
                // Find and remove the provider from the list
                for i in 0..self.provider_names.n_items() {
                    if let Some(item) = self.provider_names.string(i) {
                        if item.as_str() == name {
                            self.provider_names.remove(i);
                            log::info!("Weather provider removed: {name}");
                            break;
                        }
                    }
                }
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            CommandOutput::None => {}
            CommandOutput::DBusConnection(connection) => {
                self.dbus_session = Some(Arc::new(connection));
                sender.input(Input::WeatherUpdateSessionStart);
            }
        }
    }
}
