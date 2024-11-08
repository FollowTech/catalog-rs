// #![windows_subsystem = "windows"]
use catalog_lib::error::CatalogError;
use iced::{
    alignment::Horizontal,
    theme::Palette,
    widget::{button, center, column, container, row, text, text_input},
    Alignment::Center,
    Background, Border, Element,
    Length::Fill,
    Task, Theme,
};

#[derive(Debug)]
enum Catalog {
    Loading,
    Loaded(State),
}

#[derive(Debug, Default, Clone)]
struct State {
    size: (f32, f32),
    title: String,
    catalog_path: String,
    ic_path: String,        // dirty: bool,
    get_path_error: String, // saving: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(State),
    // Saved(Result<(), SaveError>),
    InputCatalogPathChanged(String),
    InputIcPathChanged(String),
    GoToSelectCatalog,
    GoToSeleceIc,
    StartUpdate,
    GoToHomePage,
    ButtonClicked(()),
    // CreateTask,
    // FilterChanged(Filter),
    // TaskMessage(usize, TaskMessage),
    // TabPressed { shift: bool },
    // ToggleFullscreen(window::Mode),
}

impl State {
    async fn load() -> State {
        let paths = catalog_lib::get_catalog_and_ic_paths();
        // let paths: Result<(String, String), CatalogError> = Ok(("s".into(), "ss".into()));
        let (width, height) = catalog_lib::get_desktop_window_size();
        match paths {
            Ok(catalog_info) => State {
                catalog_path: catalog_info.cab_path,
                ic_path: catalog_info.ic_path,
                title: "Welcome to the Home Page".into(),
                size: (width as f32, height as f32),
                ..Default::default()
            },
            Err(e) => {
                println!("Error: {}", e);
                State {
                    get_path_error: e.to_string(),
                    ..Default::default()
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
                let command = match message {
                    // Message::InputCatalogPathChanged(catalog_path) => {
                    //     state.catalog_path = catalog_path;

                    //     Task::none()
                    // }
                    // Message::InputIcPathChanged(ic_path) => {
                    //     state.ic_path = ic_path;
                    //     Task::none()
                    // }
                    // Message::Loaded(state) => todo!(),
                    Message::GoToSelectCatalog => handle_file_selection(&mut state.catalog_path),
                    Message::GoToSeleceIc => handle_file_selection(&mut state.ic_path),
                    Message::StartUpdate => {
                        Task::perform(catalog_lib::process(), Message::ButtonClicked)
                    }
                    // Message::GoToHomePage => todo!(),
                    Message::ButtonClicked(()) => {
                        println!("Button Clicked");
                        Task::none()
                    }
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
                catalog_path,
                ic_path,
                get_path_error,
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
                            .height(size.1 / 10.0)
                            .color([0.5, 0.5, 0.5])
                            .align_x(Center), // .on_submit(Message::CreateTask),
                        row!(
                            text_input("请选择你的catalog cab文件?", catalog_path)
                                // .on_input(Message1::InputCatalogPathChanged)
                                .style(border_sytle)
                                .align_x(Center),
                            button(text("Catalog").align_x(Horizontal::Center))
                                .width(100)
                                .on_press(Message::GoToSelectCatalog),
                        )
                        .spacing(20),
                        row!(
                            text_input("请选择你的ic文件?", ic_path)
                                // .on_input(Message::InputIcPathChanged)
                                // .on_submit(Message::CreateTask)
                                .style(border_sytle)
                                .align_x(Center),
                            button(text("IC").align_x(Horizontal::Center))
                                .width(100)
                                .on_press(Message::GoToSeleceIc),
                        )
                        .spacing(20),
                        button(text("Start Update")).on_press(Message::StartUpdate),
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

fn handle_file_selection(path: &mut String) -> Task<Message> {
    match catalog_lib::open_file_dialog() {
        Ok(file_path) => {
            println!("{}", file_path);
            *path = file_path;
            Task::none()
        }
        Err(e) => {
            eprintln!("Error opening file dialog: {}", e);
            Task::none()
        }
    }
}

fn loading_message<'a>() -> Element<'a, Message> {
    center(text("Loading...").width(Fill).align_x(Center).size(50)).into()
}

/// 主函数
fn main() -> iced::Result {
    let (screen_width, screen_height) = catalog_lib::get_desktop_window_size();
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
        .window_size(((screen_width / 2) as f32, (screen_height / 2) as f32))
        .theme(Catalog::theme)
        .run_with(Catalog::new)
}
