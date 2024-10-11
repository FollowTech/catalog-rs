use catalog_lib::error::CatalogError;
use iced::{
    alignment::Horizontal,
    color,
    font::{Family, Stretch, Style, Weight},
    widget::{
        button, column, container, row, text, text_input, vertical_space,
        Container as WidgetContainer,
    },
    window::Settings,
    Element, Font,
    Length::{self, Fill},
    Size, Theme,
};
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
fn home_page<'a>(state: &'a AppState) -> Element<'a, Message> {
    container(
        column!(
            vertical_space(),
            text("Welcome to the Home Page")
                .font(Font {
                    family: Family::Monospace,
                    weight: Weight::ExtraBold,
                    stretch: Stretch::SemiExpanded,
                    style: Style::Oblique
                })
                .size(50)
                .color(color!(0x0000FF)),
            vertical_space(),
            row!(
                text_input("Input 1", &state.catalog_path_field)
                    .on_input(|catalog_path| Message::CatalogFieldChanged(catalog_path)),
                vertical_space(),
                button(text("Catalog").align_x(Horizontal::Center))
                    .width(100)
                    .on_press(Message::GoToSelectorPage),
            )
            .spacing(10),
            row!(
                text_input("Input 2", &state.ic_path_field)
                    .on_input(|ic| Message::IcFieldChanged(ic)),
                vertical_space(),
                button(text("Ic"))
                    .width(100)
                    .on_press(Message::GoToSelectorPage),
            )
            .spacing(10),
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
    iced::application("Catalog", update, view)
        .window(Settings {
            size: Size::new((screen_width / 2) as f32, (screen_height / 2) as f32),
            ..Settings::default()
        })
        .theme(theme)
        .run()
}
