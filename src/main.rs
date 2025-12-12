use iced::{
    widget::{button, column, container, image, text},
    Application, Command, Element, Length, Settings, Theme,
};
use iced::image;

fn main() -> iced::Result {
    ImageViewer::run(Settings::default())
}

#[derive(Debug, Clone)]
enum Message {
    LoadImage(String),
    ImageLoaded(Result<iced::widget::image::Handle, String>),
}

struct ImageViewer {
    image_handle: Option<iced::widget::image::Handle>,
    error: Option<String>,
}

impl Application for ImageViewer {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            ImageViewer {
                image_handle: None,
                error: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Image Viewer")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadImage(path) => {
                return Command::perform(load_image(path), Message::ImageLoaded);
            }
            Message::ImageLoaded(result) => match result {
                Ok(handle) => {
                    self.image_handle = Some(handle);
                    self.error = None;
                }
                Err(err) => {
                    self.error = Some(err);
                    self.image_handle = None;
                }
            },
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let load_button = button("Load Image").on_press(Message::LoadImage("example.jpg".to_string())); // 例として固定

        let content = if let Some(handle) = &self.image_handle {
            column![
                load_button,
                image(handle.clone()).width(Length::Fill).height(Length::Fill),
            ]
        } else if let Some(err) = &self.error {
            column![
                load_button,
                text(format!("Error: {}", err)),
            ]
        } else {
            column![
                load_button,
                text("No image loaded"),
            ]
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

async fn load_image(path: String) -> Result<iced::widget::image::Handle, String> {
    let img = image::open(&path)
        .map_err(|e| format!("Failed to open image: {}", e))?;

    let rgba = img.to_rgba8();
    let width = rgba.width() as u32;
    let height = rgba.height() as u32;
    let data = rgba.into_raw();

    Ok(iced::widget::image::Handle::from_pixels(width, height, data))
}
