use super::file_field::FileField;
use iced::widget::{column, text, text_input};
use iced::Element;
use std::path::PathBuf;
use steamworks::{PublishedFileId, QueryResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemInfoMessage {
    EditName(String),
    EditPreviewImage(String),
    EditTargetFolder(String),
    BrowsePreviewImage,
    BrowseTargetFolder,
    EditChangeNotes(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemInfoState {
    name: String,
    preview_image: FileField,
    target_folder: FileField,
    change_notes: String,
}

impl Default for ItemInfoState {
    fn default() -> Self {
        ItemInfoState {
            name: String::new(),
            preview_image: FileField::new(),
            target_folder: FileField::new(),
            change_notes: String::new(),
        }
    }
}

impl ItemInfoState {
    pub fn update(&mut self, message: ItemInfoMessage) {
        match message {
            ItemInfoMessage::EditName(new_name) => self.name = new_name,
            ItemInfoMessage::EditPreviewImage(new_path) => {
                self.preview_image = FileField::from(new_path)
            }
            ItemInfoMessage::EditTargetFolder(new_path) => {
                self.target_folder = FileField::from(new_path)
            }
            ItemInfoMessage::BrowsePreviewImage => {
                self.preview_image.select_file();
            }
            ItemInfoMessage::BrowseTargetFolder => {
                self.target_folder.select_dir();
            }
            ItemInfoMessage::EditChangeNotes(new_notes) => self.change_notes = new_notes,
        }
    }

    pub fn view(&self, file_id: Option<PublishedFileId>) -> Element<ItemInfoMessage> {
        column![
            if let Some(file_id) = file_id {
                text(format!("Updating item with ID: {}", file_id.0))
            } else {
                text("Creating new item:")
            },
            text_input("Name", &self.name, ItemInfoMessage::EditName,),
            self.preview_image.view(
                "Preview Image",
                if file_id.is_some() { "Optional" } else { "" },
                ItemInfoMessage::EditPreviewImage,
                ItemInfoMessage::BrowsePreviewImage,
            ),
            self.target_folder.view(
                "Target Folder",
                "",
                ItemInfoMessage::EditTargetFolder,
                ItemInfoMessage::BrowseTargetFolder,
            ),
            text_input(
                "Changenotes",
                &self.change_notes,
                ItemInfoMessage::EditChangeNotes
            )
        ]
        .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemInfo {
    pub name: String,
    pub preview_image: PathBuf,
    pub target_folder: PathBuf,
    pub change_notes: String,
}

impl From<ItemInfo> for ItemInfoState {
    fn from(value: ItemInfo) -> Self {
        ItemInfoState {
            name: value.name,
            preview_image: FileField::from(value.preview_image),
            target_folder: FileField::from(value.target_folder),
            change_notes: value.change_notes,
        }
    }
}

impl From<QueryResult> for ItemInfo {
    fn from(value: QueryResult) -> Self {
        ItemInfo {
            name: value.title,
            preview_image: PathBuf::new(),
            target_folder: PathBuf::new(),
            change_notes: String::new(),
        }
    }
}

impl TryFrom<ItemInfoState> for ItemInfo {
    type Error = String;

    fn try_from(value: ItemInfoState) -> Result<Self, Self::Error> {
        if value.name.is_empty() {
            return Err("Name cannot be empty.".to_string());
        }

        let preview_field_exists = value.preview_image.path.exists();
        let has_preview = preview_field_exists && value.preview_image.path.is_file();
        if !has_preview {
            if !value.preview_image.path.to_string_lossy().is_empty() {
                if !preview_field_exists {
                    return Err(format!(
                        "Preview image \"{}\" does not exist.",
                        value.preview_image.path.to_string_lossy()
                    ));
                } else {
                    return Err(format!(
                        "Preview image \"{}\" is not a file.",
                        value.preview_image.path.to_string_lossy()
                    ));
                }
            }
        }

        if !value.target_folder.path.exists() {
            if value.target_folder.path.to_string_lossy().is_empty() {
                return Err("Target folder cannot be empty.".to_string());
            } else {
                return Err(format!(
                    "Target folder \"{}\" does not exist.",
                    value.target_folder.path.to_string_lossy()
                ));
            }
        }

        Ok(ItemInfo {
            name: value.name,
            preview_image: value.preview_image.path,
            target_folder: value.target_folder.path,
            change_notes: value.change_notes,
        })
    }
}
