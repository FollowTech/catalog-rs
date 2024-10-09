use iced::{
    futures::io::Window,
    theme,
    widget::{Button, Column, Container as WidgetContainer, Text, TextInput},
    window::Settings,
    Alignment, Element, Length, Size, Theme,
};

/// 应用程序的状态
#[derive(Debug, Default)]
struct AppState {
    current_page: Page,
    input1: String,
    input2: String,
}

/// 应用程序的消息
#[derive(Debug, Clone)]
enum Message {
    GoToHomePage,
    GoToSelectorPage,
    Input1Changed(String),
    Input2Changed(String),
    ButtonClicked,
}

#[derive(Debug)]
/// 页面枚举
enum Page {
    HomePage,
    SelectorPage,
}

impl Default for Page {
    fn default() -> Self {
        Page::HomePage
    }
}

/// 主页视图
fn home_page<'a>(state: &'a AppState) -> Element<'a, Message> {
    let content = Column::new()
        .push(Text::new("Welcome to the Home Page"))
        .push(
            TextInput::new("Input 1", &state.input1)
                .on_input(|catalog| Message::Input1Changed(catalog)),
        )
        .push(TextInput::new("Input 2", &state.input2).on_input(|ic| Message::Input2Changed(ic)))
        .push(Button::new(Text::new("Go to Selector Page")).on_press(Message::GoToSelectorPage));

    WidgetContainer::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// 选择页面视图
fn selector_page() -> Element<'static, Message> {
    let content = Column::new()
        .spacing(20)
        .push(Text::new("Welcome to the Selector Page"))
        .push(Button::new(Text::new("Go to Home Page")).on_press(Message::GoToHomePage));

    WidgetContainer::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// 更新逻辑
fn update(state: &mut AppState, message: Message) {
    match message {
        Message::GoToHomePage => state.current_page = Page::HomePage,
        Message::GoToSelectorPage => state.current_page = Page::SelectorPage,
        Message::Input1Changed(input) => state.input1 = input,
        Message::Input2Changed(input) => state.input2 = input,
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

use catalog_lib::error::CatalogError;

/// 主函数
fn main() -> iced::Result {
    iced::application("Catalog", update, view)
        .window(Settings::default())
        .theme(theme)
        .run()
}
