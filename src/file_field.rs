use super::err_dialog_types::error_dialog;
use iced::widget::{button, column, row, text, text_input};
use iced::Element;
use native_dialog::FileDialogBuilder;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileField {
    pub path: PathBuf,
}

impl FileField {
    pub fn new() -> Self {
        FileField {
            path: PathBuf::new(),
        }
    }

    pub fn view<'a, Message: Clone + 'a>(
        &self,
        label: &str,
        placeholder: &str,
        edit_msg: fn(String) -> Message,
        browse_msg: Message,
    ) -> Element<'a, Message> {
        column![
            text(label),
            row![
                text_input(placeholder, &self.path.to_string_lossy(), edit_msg),
                button("Browse",).on_press(browse_msg),
            ],
        ]
        .into()
    }

    pub fn select_file(&mut self) {
        let result = FileDialogBuilder::default()
            .add_filter("JPG Files", &["jpg", "jpeg"])
            .open_single_file()
            .show();

        if let Ok(pathbuf) = result {
            if let Some(pathbuf) = pathbuf {
                self.path = pathbuf;
            };
        } else {
            error_dialog(
                format!("Failed to select file. Error: {:?}", result.err().unwrap()).as_str(),
            );
        }
    }

    pub fn select_dir(&mut self) {
        let result = FileDialogBuilder::default().open_single_dir().show();

        if let Ok(pathbuf) = result {
            if let Some(pathbuf) = pathbuf {
                self.path = pathbuf;
            };
        } else {
            error_dialog(
                format!(
                    "Failed to select directory. Error: {:?}",
                    result.err().unwrap()
                )
                .as_str(),
            );
        }
    }
}

impl From<PathBuf> for FileField {
    fn from(path: PathBuf) -> Self {
        FileField { path }
    }
}

impl From<String> for FileField {
    fn from(path: String) -> Self {
        FileField {
            path: PathBuf::from(path),
        }
    }
}

impl From<&str> for FileField {
    fn from(path: &str) -> Self {
        FileField {
            path: PathBuf::from(path),
        }
    }
}
