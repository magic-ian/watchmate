use futures::StreamExt;
use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use infinitime::{bt, fdo::weather, zbus};
use relm4::{gtk, Component, ComponentParts, ComponentSender, JoinHandle, RelmWidgetExt};
use std::sync::Arc;

#[derive(Debug)]
pub enum Input {
    Device(Option<Arc<bt::InfiniTime>>),
    WeatherSessionStart,
    WeatherSessionEnded,
    ProviderUpdateSessionStart,
    ProviderUpdateSessionEnded,
    ProviderAdded(weather::WeatherProvider),
    ProviderRemoved(String),
}

#[derive(Debug)]
pub enum CommandOutput {
    None,
    DBusConnection(zbus::Connection),
}

#[derive(Default)]
pub struct Model {
    provider_handles: Vec<weather::WeatherProvider>,
    provider_names: gtk::StringList,
    infinitime: Option<Arc<bt::InfiniTime>>,
    weather_task: Option<JoinHandle<()>>,
    update_task: Option<JoinHandle<()>>,
    dbus_session: Option<Arc<zbus::Connection>>,
    dropdown: gtk::DropDown,
}

impl Model {
    fn stop_weather_task(&mut self) {
        if self.weather_task.take().map(|h| h.abort()).is_some() {
            log::info!("Weather session stopped");
        }
    }

    fn stop_update_task(&mut self) {
        if self.update_task.take().map(|h| h.abort()).is_some() {
            log::info!("Weather provider list update session stopped");
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

            if model.provider_handles.is_empty() {
                gtk::Label {
                    set_label: "Not available",
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
                    connect_selected_notify => Input::WeatherSessionStart,
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
                    Some(_) => sender.input(Input::WeatherSessionStart),
                    None => self.stop_weather_task(),
                }
            }
            Input::WeatherSessionStart => {
                if let Some(infinitime) = self.infinitime.clone() {
                    let index = self.dropdown.selected() as usize;
                    if index < self.provider_handles.len() {
                        // Stop current weather session
                        self.stop_weather_task();
                        // Start new weather session
                        let provider = self.provider_handles[index].clone();
                        let dbus_session = self.dbus_session.clone();
                        let task_handle = relm4::spawn(async move {
                            // This is where we would periodically fetch weather data
                            // and send it to the watch. For now, just log it.
                            log::info!(
                                "Weather session started for provider: {}",
                                provider.name
                            );
                            // TODO: Implement periodic weather updates
                            sender.input(Input::WeatherSessionEnded);
                        });
                        self.weather_task = Some(task_handle);
                    }
                }
            }
            Input::WeatherSessionEnded => {
                self.provider_handles.clear();
                self.provider_names = gtk::StringList::new(&[]);
                self.weather_task = None;
            }
            Input::ProviderUpdateSessionStart => {
                if let Some(dbus_session) = self.dbus_session.clone() {
                    self.stop_update_task();
                    let task_handle = relm4::spawn(async move {
                        match weather::get_providers_update_stream(&dbus_session).await {
                            Ok(stream) => {
                                stream
                                    .for_each(|event| {
                                        let sender_ = sender.clone();
                                        async move {
                                            match event {
                                                weather::ProvidersListEvent::ProviderAdded(
                                                    provider,
                                                ) => {
                                                    sender_.input(Input::ProviderAdded(provider));
                                                }
                                                weather::ProvidersListEvent::ProviderRemoved(
                                                    service,
                                                ) => {
                                                    sender_.input(Input::ProviderRemoved(service));
                                                }
                                            }
                                        }
                                    })
                                    .await
                            }
                            Err(error) => {
                                log::error!(
                                    "Failed to start provider list update session: {error}"
                                )
                            }
                        }
                        sender.input(Input::ProviderUpdateSessionEnded);
                    });
                    self.update_task = Some(task_handle);
                }
            }
            Input::ProviderUpdateSessionEnded => {
                log::info!("Restarting provider list update session");
                sender.input(Input::ProviderUpdateSessionStart);
            }
            Input::ProviderAdded(provider) => {
                self.provider_names.append(&provider.name);
                self.provider_handles.push(provider.clone());
                log::info!("Weather provider started: {}", provider.name);
            }
            Input::ProviderRemoved(service_name) => {
                if let Some(index) = self
                    .provider_handles
                    .iter()
                    .position(|p| p.service_name == service_name)
                {
                    let name = self.provider_names.string(index as u32).unwrap();
                    self.provider_names.remove(index as u32);
                    self.provider_handles.remove(index);
                    log::info!("Weather provider stopped: {name}");
                    if self.provider_handles.is_empty() {
                        self.stop_weather_task();
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
                let connection_arc = Arc::new(connection);
                self.dbus_session = Some(connection_arc.clone());
                
                // Initialize provider list
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    match weather::get_providers(&connection_arc).await {
                        Ok(providers) => {
                            for provider in providers {
                                sender_clone.input(Input::ProviderAdded(provider));
                            }
                        }
                        Err(error) => {
                            log::error!("Failed to get initial provider list: {error}");
                        }
                    }
                });
                
                sender.input(Input::ProviderUpdateSessionStart);
            }
        }
    }
}
