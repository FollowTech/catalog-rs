use std::path::PathBuf;

// #![windows_subsystem = "windows"]
use catalog_lib::{get_cur_path, CatalogInfo};
use iced::{
    alignment::Horizontal,
    theme::Palette,
    widget::{button, center, column, container, row, text, text_input},
    Alignment::Center,
    Background, Border, Element,
    Length::Fill,
    Size, Task, Theme,
};

#[derive(Debug)]
enum Catalog {
    Loading,
    Loaded(State),
}

#[derive(Debug, Default, Clone)]
struct State {
    size: Size,
    title: String,
    catalog_info: CatalogInfo,
    error: String,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(State),
    GoToSelectCatalog,
    GoToSeleceIc,
    CatalogChanged,
    StartUpdate,
    ButtonClicked(State),
    // CreateTask,
    // FilterChanged(Filter),
    // TaskMessage(usize, TaskMessage),
    // TabPressed { shift: bool },
    // ToggleFullscreen(window::Mode),
}

impl State {
    async fn load() -> State {
        let paths = catalog_lib::get_catalog_and_ic_paths(get_cur_path()).await;
        // let paths: Result<(String, String), CatalogError> = Ok(("s".into(), "ss".into()));
        let size = catalog_lib::get_window_size();
        match paths {
            Ok(catalog_info) => State {
                catalog_info,
                title: "Welcome to the Home Page".into(),
                size,
                ..Default::default()
            },
            Err(e) => {
                println!("load Error: {}", e);
                State {
                    error: e.to_string(),
                    ..Default::default()
                }
            }
        }
    }

    async fn process(&self) -> State {
        let result = catalog_lib::handle(&self.catalog_info).await;
        match result {
            Ok(_) => self.clone(),
            Err(e) => {
                println!("process Error: {}", e);
                State {
                    error: e.to_string(),
                    ..self.clone()
                }
            }
        }
    }
}

impl Catalog {
    fn new() -> (Self, Task<Message>) {
        (Self::Loading, Task::perform(State::load(), Message::Loaded))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match self {
            Catalog::Loading => {
                match message {
                    Message::Loaded(state) => {
                        *self = Catalog::Loaded(state);
                    }
                    _ => {}
                };
                Task::none()
                // text_input::focus("new-task")
            }
            Catalog::Loaded(state) => {
                println!("Catalog-update-Catalog::Loaded(state)--{:?}", state);
                let command = match message {
                    Message::GoToSelectCatalog => {
                        file_selection(&mut state.catalog_info.cab_path, &mut state.error)
                    }
                    Message::GoToSeleceIc => {
                        file_selection(&mut state.catalog_info.ic_path, &mut state.error)
                    }
                    Message::CatalogChanged => {
                        match catalog_lib::check_catalog_info(&state.catalog_info) {
                            Ok(_) => {
                                state.error = Default::default();
                                Task::done(Message::StartUpdate)
                            }
                            Err(e) => {
                                *(&mut state.error) = e.to_string();
                                Task::none()
                            }
                        }
                    }
                    Message::StartUpdate => {
                        let state = state.clone();
                        Task::perform(async move { state.process().await }, Message::ButtonClicked)
                    }
                    // Message::GoToHomePage => todo!(),
                    Message::ButtonClicked(new_state) => Task::none(),
                    _ => {
                        println!("ss");
                        Task::none()
                    }
                };

                Task::batch(vec![command])
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match self {
            Catalog::Loading => loading_message(),
            Catalog::Loaded(State {
                title,
                catalog_info,
                error,
                size,
            }) => {
                let border_sytle = |theme: &Theme, status: text_input::Status| {
                    let palette = theme.extended_palette();
                    text_input::Style {
                        background: Background::Color(palette.background.base.color),
                        border: Border {
                            radius: 6.0.into(),
                            width: 1.0,
                            color: palette.background.strong.color,
                        },
                        icon: palette.background.weak.text,
                        placeholder: palette.background.strong.color,
                        value: palette.background.base.text,
                        selection: palette.primary.weak.color,
                    }
                };

                container(
                    column![
                        text("Welcome to the Home Page")
                            .width(Fill)
                            .height(size.height / 2.0)
                            .color([0.5, 0.5, 0.5])
                            .align_x(Center), // .on_submit(Message::CreateTask),
                        row!(
                            text_input(
                                "请选择你的catalog cab文件?",
                                catalog_info.cab_path.to_str().unwrap_or("")
                            )
                            // .on_input(Message1::InputCatalogPathChanged)
                            .style(border_sytle)
                            .align_x(Center),
                            button(text("Catalog").align_x(Horizontal::Center))
                                .width(100)
                                .on_press(Message::GoToSelectCatalog),
                        )
                        .spacing(20),
                        row!(
                            text_input(
                                "请选择你的ic文件?",
                                catalog_info.ic_path.to_str().unwrap_or("")
                            )
                            // .on_input(Message::InputIcPathChanged)
                            // .on_submit(Message::CreateTask)
                            .style(border_sytle)
                            .align_x(Center),
                            button(text("IC").align_x(Horizontal::Center))
                                .width(100)
                                .on_press(Message::GoToSeleceIc),
                        )
                        .spacing(20),
                        button(text("Start Update")).on_press(Message::CatalogChanged),
                        text(error).color([1.0, 0.0, 0.0])
                    ]
                    .align_x(Horizontal::Center)
                    .spacing(30)
                    .max_width(800),
                )
                .padding(20)
                .center_x(Fill)
                .center_y(Fill)
                .into()
            }
        }
    }

    fn theme(&self) -> Theme {
        Theme::custom("CatalogTheme".into(), Palette::DARK)
    }

    // fn subscription(&self) -> Subscription<Message> {
    //     use keyboard::key;

    //     keyboard::on_key_press(|key, modifiers| {
    //         let keyboard::Key::Named(key) = key else {
    //             return None;
    //         };

    //         match (key, modifiers) {
    //             (key::Named::Tab, _) => Some(Message::TabPressed {
    //                 shift: modifiers.shift(),
    //             }),
    //             (key::Named::ArrowUp, keyboard::Modifiers::SHIFT) => {
    //                 Some(Message::ToggleFullscreen(window::Mode::Fullscreen))
    //             }
    //             (key::Named::ArrowDown, keyboard::Modifiers::SHIFT) => {
    //                 Some(Message::ToggleFullscreen(window::Mode::Windowed))
    //             }
    //             _ => None,
    //         }
    //     })
    // }
}

fn file_selection(path: &mut PathBuf, error: &mut String) -> Task<Message> {
    match catalog_lib::open_file_dialog() {
        Ok(file_path) => {
            *path = file_path;
            Task::done(Message::CatalogChanged)
        }
        Err(e) => {
            *error = e.to_string();
            Task::done(Message::CatalogChanged)
        }
    }
}

fn loading_message<'a>() -> Element<'a, Message> {
    center(text("Loading...").width(Fill).align_x(Center).size(50)).into()
}

/// 主函数
fn main() -> iced::Result {
    let windows_size = catalog_lib::get_window_size();
    // iced::application("Catalog", update, view)
    //     .window(Settings {
    //         size: Size::new((screen_width / 2) as f32, (screen_height / 2) as f32),
    //         ..Settings::default()
    //     })
    //     .theme(theme)
    //     .run()

    iced::application("Catalog", Catalog::update, Catalog::view)
        // .subscription(Todos::subscription)
        // .font(include_bytes!("../fonts/icons.ttf").as_slice())
        .window_size(windows_size)
        .theme(Catalog::theme)
        .run_with(Catalog::new)
}
