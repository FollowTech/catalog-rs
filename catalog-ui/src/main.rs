use std::error::Error;

use catalog_lib::{error::CatalogError, get_catalog_and_ic_paths};
use iced::{
    alignment::Horizontal,
    application::Title,
    color,
    widget::{
        button, center, column,
        container::{self, Style},
        row, text, text_input, Container as WidgetContainer,
    },
    Alignment::Center,
    Border, Element,
    Length::Fill,
    Task, Theme,
};

mod trait_test;

#[derive(Debug)]
enum Catalog {
    Loading,
    Loaded(State),
}

#[derive(Debug, Default, Clone)]
struct State {
    title: String,
    catalog_path: String,
    ic_path: String,        // dirty: bool,
    get_path_error: String, // saving: bool,
}

#[derive(Debug, Clone)]
enum Message1 {
    Loaded(State),
    // Saved(Result<(), SaveError>),
    InputCatalogPathChanged(String),
    InputIcPathChanged(String),
    GoToSelectCatalog,
    GoToSeleceIc,
    StartUpdate,
    // CreateTask,
    // FilterChanged(Filter),
    // TaskMessage(usize, TaskMessage),
    // TabPressed { shift: bool },
    // ToggleFullscreen(window::Mode),
}

impl State {
    async fn load() -> State {
        let paths = get_catalog_and_ic_paths();
        match paths {
            Ok(paths) => State {
                catalog_path: paths.0,
                ic_path: paths.1,
                title: "Welcome to the Home Page".into(),
                ..Default::default()
            },
            Err(e) => State {
                get_path_error: e.to_string(),
                ..Default::default()
            },
        }
        // Ok(State {
        //     catalog_path: paths.0,
        //     ic_path: paths.1,
        //     title: "Welcome to the Home Page".into(),
        // })
    }
}

impl Catalog {
    fn new() -> (Self, Task<Message1>) {
        (
            Self::Loading,
            Task::perform(State::load(), Message1::Loaded),
        )
    }

    fn update(&mut self, message: Message1) -> Task<Message1> {
        match self {
            Catalog::Loading => {
                match message {
                    Message1::Loaded(state) => {
                        *self = Catalog::Loaded(State { ..state });
                    }
                    _ => {}
                }

                text_input::focus("new-task")
            }
            Catalog::Loaded(state) => {
                let command = match message {
                    Message1::InputCatalogPathChanged(catalog_path) => {
                        state.catalog_path = catalog_path;

                        Task::none()
                    }
                    Message1::InputIcPathChanged(ic_path) => {
                        state.ic_path = ic_path;
                        Task::none()
                    }
                    Message1::Loaded(state) => todo!(),
                    Message1::GoToSelectCatalog => todo!(),
                    Message1::GoToSeleceIc => todo!(),
                    Message1::StartUpdate => todo!(),
                };

                Task::batch(vec![command])
            }
        }
    }

    fn view(&self) -> Element<'_, Message1> {
        match self {
            Catalog::Loading => loading_message(),
            Catalog::Loaded(State {
                title,
                catalog_path,
                ic_path,
                get_path_error,
            }) => {
                let border_sytle = |theme: &Theme| {
                    let palette = theme.extended_palette();
                    Style {
                        background: Some(palette.background.weak.color.into()),
                        border: Border {
                            width: 4.0,
                            radius: 20.0.into(),
                            color: palette.background.strong.color,
                        },
                        ..Style::default()
                    }
                };

                WidgetContainer::new(
                    column!(
                        text("Welcome to the Home Page")
                            .width(Fill)
                            .size(100)
                            .color([0.5, 0.5, 0.5])
                            .align_x(Center), // .on_submit(Message::CreateTask),
                        row!(
                            text("请选择你的catalog cab文件?")
                                // .on_input(Message1::InputCatalogPathChanged)
                                // .padding(15)
                                .size(30)
                                .align_x(Center),
                            button(text("Catalog").align_x(Horizontal::Center))
                                .width(100)
                                .on_press(Message1::GoToSelectCatalog),
                        ),
                        row!(
                            text_input("请选择你的ic文件?", ic_path)
                                .on_input(Message1::InputIcPathChanged)
                                // .on_submit(Message::CreateTask)
                                .padding(15)
                                .size(30)
                                .align_x(Center),
                            button(text("IC").align_x(Horizontal::Center))
                                .width(100)
                                .on_press(Message1::GoToSeleceIc),
                        ),
                    )
                    .spacing(20)
                    .max_width(800),
                )
                .padding(20)
                .center_x(Fill)
                .center_y(Fill)
                .into()
            }
        }
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

fn loading_message<'a>() -> Element<'a, Message1> {
    center(text("Loading...").width(Fill).align_x(Center).size(50)).into()
}

/// 应用程序的状态
#[derive(Debug)]
struct AppState {
    current_page: Page,
    catalog_path_field: String,
    ic_path_field: String,
}

impl Default for AppState {
    fn default() -> Self {
        let de: AppState = match catalog_lib::get_catalog_and_ic_paths() {
            Ok((catlog_path, ic_path)) => Self {
                current_page: Page::default(),
                catalog_path_field: catlog_path,
                ic_path_field: ic_path,
            },
            Err(e) => {
                let hit = match e {
                    CatalogError::NoFilesFound(e) => e,
                    CatalogError::MultipleFilesFound(e1, e2) => format!("{}{}", e1, e2),
                    CatalogError::IoError(e) => "io error".into(),
                    e => unreachable!(),
                };
                Self {
                    current_page: Page::default(),
                    catalog_path_field: "请选择你的Cab".into(),
                    ic_path_field: "请选择你的ic".into(),
                }
            }
        };
        de
    }
}

/// 应用程序的消息
#[derive(Debug, Clone)]
enum Message {
    GoToHomePage,
    GoToSelectorPage,
    CatalogFieldChanged(String),
    IcFieldChanged(String),
    ButtonClicked,
}

#[derive(Debug, Default)]
/// 页面枚举
enum Page {
    #[default]
    HomePage,
    SelectorPage,
}

/// 主页视图
fn home_page(state: &AppState) -> Element<'_, Message> {
    WidgetContainer::new(
        column!(
            text("Welcome to the Home Page")
                .size(50)
                .color(color!(0x0000FF)),
            row!(
                text_input("Input 1", &state.catalog_path_field)
                    .on_input(Message::CatalogFieldChanged),
                button(text("Catalog").align_x(Horizontal::Center))
                    .width(100)
                    .on_press(Message::GoToSelectorPage),
            ),
            row!(
                text_input("Input 2", &state.ic_path_field).on_input(Message::IcFieldChanged),
                button(text("Ic").align_x(Center))
                    .width(100)
                    .on_press(Message::GoToSelectorPage),
            ),
            button(text("Go to Selector Page")).on_press(Message::GoToSelectorPage)
        )
        .spacing(20)
        .align_x(Horizontal::Center),
    )
    .padding(20)
    .center_x(Fill)
    .center_y(Fill)
    .into()
}

/// 选择页面视图
fn selector_page() -> Element<'static, Message> {
    let content = column!(
        text("Welcome to the Selector Page"),
        button(text("Go to Home Page")).on_press(Message::GoToHomePage),
    )
    .spacing(20);
    WidgetContainer::new(content)
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .center_y(Fill)
        .into()
}

/// 更新逻辑
fn update(state: &mut AppState, message: Message) {
    match message {
        Message::GoToHomePage => state.current_page = Page::HomePage,
        Message::GoToSelectorPage => state.current_page = Page::SelectorPage,
        Message::CatalogFieldChanged(input) => state.catalog_path_field = input,
        Message::IcFieldChanged(input) => state.ic_path_field = input,
        Message::ButtonClicked => {
            // 处理按钮点击事件
            println!("Button clicked!");
        }
    }
}

/// 视图逻辑
fn view(state: &AppState) -> Element<Message> {
    match state.current_page {
        Page::HomePage => home_page(state),
        Page::SelectorPage => selector_page(),
    }
}

fn theme(_state: &AppState) -> Theme {
    Theme::TokyoNight
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
        .run_with(Catalog::new)
}
