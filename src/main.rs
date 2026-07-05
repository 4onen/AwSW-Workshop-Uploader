mod err_dialog_types;
mod file_field;
mod item_info;
mod my_steamworks;
use err_dialog_types::ErrorDialogUnwrapper;
use iced::widget::{button, column, row, text, text_input};
use iced::{Element, Settings, Task};
use item_info::{ItemInfo, ItemInfoMessage, ItemInfoState};
use my_steamworks::WorkshopClient;
use std::num::IntErrorKind;
use steamworks::{AppId, PublishedFileId, SteamError};

const APP_ID_STR: &str = include_str!("../steam_appid.txt");

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    SetExistingId(String),
    EditItemData(ItemInfoMessage),
    ReceiveFoundItemInfo(ItemInfo),
    ReceiveItemId(PublishedFileId),
    ReceiveSteamError(SteamError),
    Proceed,
    GoBack,
    TermsLinkPressed,
}

impl Message {
    fn receive_item_id(res: Result<(PublishedFileId, bool), SteamError>) -> Self {
        match res {
            Ok((id, _)) => Message::ReceiveItemId(id),
            Err(err) => Message::ReceiveSteamError(err),
        }
    }

    fn receive_item_info(res: Result<ItemInfo, SteamError>) -> Self {
        match res {
            Ok(item_info) => Message::ReceiveFoundItemInfo(item_info),
            Err(err) => Message::ReceiveSteamError(err),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
enum ModelState {
    Initial(String),
    ExistingIdSearching(PublishedFileId, Option<SteamError>),
    ItemForm(Option<PublishedFileId>, ItemInfoState),
    CreatingItem(ItemInfo),
    CreationError(ItemInfo, SteamError),
    SendingItem(PublishedFileId, ItemInfo),
    SendingError(PublishedFileId, ItemInfo, SteamError),
    Done(PublishedFileId),
}

struct Model {
    client: WorkshopClient,
    state: ModelState,
}

fn initial_view<'a>(existing_id: &str) -> Element<'a, Message> {
    let item_id = existing_id.parse::<u64>().map(PublishedFileId);

    let mut res = column![
        text("4onen's Steam Workshop Uploader"),
        if let Err(error) = &item_id {
            if *error.kind() == IntErrorKind::Empty {
                button("Create new").on_press(Message::Proceed)
            } else {
                button("Update existing")
            }
        } else {
            button("Update existing").on_press(Message::Proceed)
        },
        text_input("Existing item ID", existing_id)
            .on_input(Message::SetExistingId)
            .on_submit(Message::Proceed),
    ];

    if let Err(error) = item_id {
        if *error.kind() != IntErrorKind::Empty {
            res = res.push(text(format!("Invalid item ID: {}.", error)));
        }
    }

    res.into()
}

fn edit_item_view<'s, 'r>(
    item_info: &'s ItemInfoState,
    existing_id: Option<PublishedFileId>,
) -> Element<'r, Message> {
    let ready_info = ItemInfo::try_from(item_info.clone());

    let mut fwd_button = if existing_id.is_some() {
        button("Update")
    } else {
        button("Create")
    };

    if let Ok(_) = &ready_info {
        fwd_button = fwd_button.on_press(Message::Proceed);
    }

    column![
        item_info
            .view(existing_id)
            .map(move |message| Message::EditItemData(message)),
        column![
            text("By submitting this item, you agree to the Steam workshop"),
            button("Terms of Service").on_press(Message::TermsLinkPressed)
        ],
        row![button("Go back").on_press(Message::GoBack), fwd_button],
        match ready_info {
            Ok(_) => text(""),
            Err(error) => text(format!("{}", error)),
        },
    ]
    .into()
}

impl Model {
    fn update_to_create_item(&mut self, item_info: ItemInfo) -> Task<Message> {
        self.state = ModelState::CreatingItem(item_info);
        Task::perform(self.client.clone().create_item(), Message::receive_item_id)
    }

    fn update_to_send_item(
        &mut self,
        item_id: PublishedFileId,
        item_info: ItemInfo,
    ) -> Task<Message> {
        self.state = ModelState::SendingItem(item_id, item_info.clone());
        Task::perform(
            self.client.clone().send_item(item_id, item_info),
            Message::receive_item_id,
        )
    }
}

fn update(model: &mut Model, message: Message) -> Task<Message> {
    if std::mem::discriminant(&message) == std::mem::discriminant(&Message::TermsLinkPressed) {
        model.client.open_terms();
        return Task::none();
    }

    match model.state.clone() {
        ModelState::Initial(idstr) => match message {
            Message::SetExistingId(idstr) => {
                model.state = ModelState::Initial(idstr);
                Task::none()
            }
            Message::Proceed => match idstr.parse::<u64>().map(PublishedFileId) {
                Ok(item_id) => {
                    model.state = ModelState::ExistingIdSearching(item_id, None);
                    Task::perform(
                        model.client.clone().get_item_info(item_id),
                        Message::receive_item_info,
                    )
                }
                _ => {
                    model.state = ModelState::ItemForm(None, ItemInfoState::default());
                    Task::none()
                }
            },
            _ => Task::none(),
        },
        ModelState::ExistingIdSearching(item_id, _) => {
            match message {
                Message::GoBack => model.state = ModelState::Initial(item_id.0.to_string()),
                Message::ReceiveFoundItemInfo(item_info) => {
                    model.state = ModelState::ItemForm(Some(item_id), item_info.into())
                }
                Message::ReceiveSteamError(err) => {
                    model.state = ModelState::ExistingIdSearching(item_id, Some(err))
                }
                _ => (),
            };
            Task::none()
        }
        ModelState::ItemForm(maybe_id, mut item_info) => match message {
            Message::EditItemData(item_info_message) => {
                item_info.update(item_info_message);
                model.state = ModelState::ItemForm(maybe_id, item_info);
                Task::none()
            }
            Message::Proceed => match ItemInfo::try_from(item_info.clone()) {
                Ok(item_info) => match maybe_id {
                    Some(item_id) => model.update_to_send_item(item_id, item_info),
                    None => model.update_to_create_item(item_info),
                },
                Err(error) => {
                    println!("Error: {}", error);
                    Task::none()
                }
            },
            Message::GoBack => {
                model.state = ModelState::Initial(
                    maybe_id
                        .map(|id| id.0.to_string())
                        .unwrap_or(String::default()),
                );
                Task::none()
            }
            _ => Task::none(),
        },
        ModelState::CreatingItem(item_info) => match message {
            Message::ReceiveItemId(item_id) => model.update_to_send_item(item_id, item_info),
            Message::ReceiveSteamError(err) => {
                model.state = ModelState::CreationError(item_info, err);
                Task::none()
            }
            _ => Task::none(),
        },
        ModelState::CreationError(item_info, _err) => {
            match message {
                Message::GoBack => model.state = ModelState::ItemForm(None, item_info.into()),
                _ => (),
            };
            Task::none()
        }
        ModelState::SendingItem(item_id, item_info) => {
            match message {
                Message::ReceiveItemId(incoming_id) => {
                    if incoming_id != item_id {
                        println!(
                            "Not advancing due to non-matching ids. Expected {}, got {}.",
                            item_id.0, incoming_id.0,
                        );
                    } else {
                        model.state = ModelState::Done(item_id);
                    };
                }
                Message::ReceiveSteamError(err) => {
                    model.state = ModelState::SendingError(item_id, item_info, err);
                }
                _ => (),
            };
            Task::none()
        }
        ModelState::SendingError(item_id, item_info, _err) => {
            match message {
                Message::GoBack => {
                    model.state = ModelState::ItemForm(item_id.into(), item_info.into())
                }
                _ => (),
            };
            Task::none()
        }
        ModelState::Done(item_id) => {
            match message {
                Message::Proceed => {
                    let item_url = format!("steam://url/CommunityFilePage/{}", item_id.0);
                    model.client.open_url(item_url.as_str());
                }
                Message::GoBack => {
                    model.state = ModelState::Initial(String::default());
                }
                _ => (),
            };
            Task::none()
        }
    }
}
fn view<'r>(model: &'r Model) -> Element<'r, Message> {
    match &model.state {
            ModelState::Initial(existing_id) => initial_view(existing_id.as_str()),
            ModelState::ExistingIdSearching(item_id, None) => column![
                text(format!("Searching for item with ID {}...", item_id.0)),
                button("Cancel").on_press(Message::GoBack),
            ]
            .into(),
            ModelState::ExistingIdSearching(item_id, Some(e)) => column![
                text(format!(
                    "Search for item with ID {} failed.\nError: {:?}",
                    item_id.0, e
                )),
                button("Go Back").on_press(Message::GoBack),
            ]
            .into(),
            ModelState::ItemForm(item_id, item_state) => edit_item_view(item_state, *item_id),
            ModelState::CreatingItem(item_info) => {
                text(format!("Creating \"{}\" on Steam Workshop...", item_info.name)).into()
            }
            ModelState::CreationError(item_info, err) => column![text(format!(
                "Error creating a new entry on the workshop:\n{:?}\n\"{}\" was not uploaded.",
                err, item_info.name
            )),
            button("Go Back").on_press(Message::GoBack),
            ]
            .into(),
            ModelState::SendingItem(item_id, _item_info) => {
                text(format!("Sending item {} to Steam Workshop...", item_id.0)).into()
            }
            ModelState::SendingError(item_id, item_info, err) => column![text(format!(
                "Error uploading your item to the workshop:\n{:?}\n\"{}\" is created on the workshop with ID {}, but does not have your files in it.\nPlease resolve the issue and try uploading to this existing ID again.",
                err, item_info.name, item_id.0
            )),
            button("Go Back").on_press(Message::GoBack),
            ].into(),
            ModelState::Done(id) => column![
                text(format!("Item ID {} uploaded to workshop.", id.0)),
                button("Go to your item").on_press(Message::Proceed),
                button("Restart").on_press(Message::GoBack),
            ]
            .into(),
        }
}

fn main() -> iced::Result {
    let boot = move || {
        let client = APP_ID_STR
            .parse()
            .map(AppId)
            .map(WorkshopClient::init_app)
            .expect_or_dialog(
                "Failed to parse App ID. This build of the workshop uploader is corrupt.",
            )
            .expect_or_dialog("Failed to initialize Steam Workshop client.");
        let state = ModelState::Initial(String::new());
        (Model { client, state }, Task::none())
    };

    iced::application(boot, update, view)
        .title("4onen's Workshop Uploader")
        .settings(Settings {
            id: None,
            fonts: vec![],
            default_font: Default::default(),
            default_text_size: iced::Pixels(20.),
            antialiasing: false,
            vsync: true,
        })
        .window(iced::window::Settings {
            size: iced::Size {
                width: 300.,
                height: 400.,
            },
            maximized: false,
            fullscreen: false,
            position: iced::window::Position::Centered,
            min_size: None,
            max_size: None,
            visible: true,
            resizable: true,
            closeable: true,
            minimizable: true,
            decorations: true,
            transparent: false,
            blur: false,
            level: iced::window::Level::Normal,
            icon: None,
            platform_specific: Default::default(),
            exit_on_close_request: true,
        })
        .run()
}
