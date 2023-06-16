#![allow(non_snake_case)]

use std::str::FromStr;

use crate::behaviours::{Behaviour, ChangeTextsizeBehaviour};
use crate::environment::model::{Instance, UploadMedia};
use crate::environment::types::{AppEvent, FileEvent};
use crate::environment::{Environment, UploadMediaExt};
use crate::view_model::AttachmentMedia;
use navicula::Effect;

use super::{PostAction, PostKind, State, Visibility};

pub type ViewStore<'a> = navicula::ViewStore<'a, super::PostReducer>;

pub fn reduce<'a>(
    context: &'a impl navicula::types::MessageContext<PostAction, PostAction, ()>,
    action: PostAction,
    state: &'a mut State,
    environment: &'a Environment,
) -> Effect<'static, PostAction> {
    log::trace!("{action:?}");
    let window = context.window();

    match action {
        PostAction::Open(images) => {
            environment.platform.update_menu(window, |config| {
                config.enable_postwindow = true;
            });
            state.text = match state.kind {
                PostKind::Post => "".to_string(),
                PostKind::Reply(ref s) | PostKind::ReplyPrivate(ref s) => {
                    // have to ignore our own username for a mention
                    let compare = format!("@{}", state.account.username);
                    let f: Vec<_> = s
                        .mentions
                        .iter()
                        .filter(|e| *e != &compare)
                        .cloned()
                        .collect();
                    let mut others = f.join(" ");
                    if !f.is_empty() {
                        others.push(' ');
                    }
                    format!("@{} {others}", s.account.acct)
                }
            };
            if matches!(state.kind, PostKind::ReplyPrivate(_)) {
                state.visibility = Some(Visibility::Direct);
            }
            let instance = environment.model.instance();
            state.validity = validate_text(instance, &state.text);

            let imgs = if images.is_empty() {
                &state.image_paths
            } else {
                &images
            };

            if imgs.is_empty() {
                Effect::NONE
            } else {
                Effect::action(PostAction::DroppedPaths(imgs.clone()))
            }
        }
        PostAction::DroppedPaths(images) => Effect::future(
            async move {
                let mut m = Vec::new();
                for i in images {
                    if let Some(p) = crate::environment::platform::read_file_to_attachment(&i) {
                        m.push(p)
                    }
                }
                m
            },
            PostAction::DroppedMedia,
        ),
        PostAction::Close => {
            environment.platform.update_menu(window, |config| {
                config.enable_postwindow = false;
            });
            if state.is_window {
                window.close();
            }
            context.send_parent(PostAction::Close);
            Effect::NONE
        }
        PostAction::FileDialog => {
            let f = crate::environment::platform::open_file_dialog("~");
            Effect::action(PostAction::FileDialogDone(f))
        }
        PostAction::FileDialogDone(result) => {
            if let Some(image) = result {
                state.images.push(image.clone());
                let model = environment.model.clone();

                let cloned_image = image.clone();
                let cloned_image2 = image;
                Effect::future(
                    async move {
                        model
                            .upload_media(&cloned_image.path, cloned_image.description)
                            .await
                    },
                    move |result| PostAction::UploadMediaDone((cloned_image2, result)),
                )
            } else {
                Effect::NONE
            }
        }
        PostAction::UploadMediaDone((image, result)) => {
            if let Some(e) = handle_image_upload(image, result, &mut state.images) {
                state.error_message = Some(e);
            }
            Effect::NONE
        }
        PostAction::RemoveImage(index) => {
            state.images.remove(index);
            Effect::NONE
        }
        PostAction::ShowImageDisk(index) => {
            if let Some(img) = state.images.get(index) {
                crate::environment::platform::open_file(&img.path)
            }
            Effect::NONE
        }
        PostAction::UpdateVisibility(vis) => {
            let Ok(v) = Visibility::from_str(&vis) else {
                state.error_message = Some(format!("Invalid Visibility: {vis:?}"));
                return Effect::NONE
            };
            state.visibility = Some(v);
            Effect::NONE
        }
        PostAction::UpdateImageDescription(index, desc) => {
            let id = {
                let Some(entry) = state.images.get_mut(index) else { return Effect::NONE };
                entry.description = Some(desc.clone());
                entry.server_id.clone()
            };
            let Some(id) = id else { return Effect::NONE };
            let model = environment.model.clone();
            Effect::future(
                async move { model.update_media(id, Some(desc)).await },
                PostAction::UpdateImageDescriptionResult,
            )
        }
        PostAction::UpdateImageDescriptionResult(result) => {
            if let Err(e) = result {
                state.error_message = Some(format!("Could not change description: {e:?}"));
            }
            Effect::NONE
        }
        PostAction::UpdateText(text) => {
            let instance = environment.model.instance();
            state.validity = validate_text(instance, &text);
            state.text = text;
            Effect::NONE
        }
        PostAction::Post => {
            state.posting = true;
            let model = environment.model.clone();
            let reply_to = match state.kind {
                PostKind::Post => None,
                PostKind::Reply(ref i) => Some(i.id.0.clone()),
                PostKind::ReplyPrivate(ref i) => Some(i.id.0.clone()),
            };
            let media_ids: Vec<_> = state
                .images
                .iter()
                .flat_map(|e| e.server_id.clone())
                .collect();
            let media_ids = (!media_ids.is_empty()).then_some(media_ids);
            let visibility = state.visibility.as_ref().map(|e| e.into());
            let text = state.text.clone();
            Effect::future(
                async move {
                    model
                        .post_status(text, media_ids, reply_to, None, visibility)
                        .await
                },
                PostAction::PostResult,
            )
        }
        PostAction::PostResult(ref result) => {
            state.posting = false;
            match result {
                Ok(_) => {
                    context.send_parent(action);
                    Effect::action(PostAction::Close)
                }
                Err(e) => {
                    state.error_message = Some(e.clone());
                    Effect::NONE
                }
            }
        }
        PostAction::AppEvent(ref event) => match event {
            AppEvent::FileEvent(FileEvent::Hovering(valid)) => {
                state.dropping_file = *valid;
                Effect::NONE
            }
            AppEvent::FileEvent(FileEvent::Dropped(images)) => {
                let cloned = images.clone();
                Effect::future(
                    async move {
                        let mut m = Vec::new();
                        for i in cloned {
                            if let Some(p) =
                                crate::environment::platform::read_file_to_attachment(&i)
                            {
                                m.push(p)
                            }
                        }
                        m
                    },
                    PostAction::DroppedMedia,
                )
            }
            AppEvent::FileEvent(FileEvent::Cancelled) => {
                state.dropping_file = false;
                Effect::NONE
            }
            AppEvent::ClosingWindow => {
                environment.platform.update_menu(window, |config| {
                    config.enable_postwindow = false;
                });

                Effect::NONE
            }
            AppEvent::FocusChange(change) => {
                environment.platform.update_menu(window, |options| {
                    options.enable_postwindow = *change;
                });
                Effect::NONE
            }
            AppEvent::MenuEvent(ref menu_event) => {
                use crate::environment::types::MainMenuEvent;
                match menu_event {
                    MainMenuEvent::NewPost => {
                        context.send_parent(action);
                        Effect::NONE
                    }
                    MainMenuEvent::PostWindowSubmit => {
                        return Effect::action(PostAction::Post);
                    }
                    MainMenuEvent::PostWindowAttachFile => {
                        return Effect::action(PostAction::FileDialog);
                    }
                    MainMenuEvent::TextSizeIncrease
                    | MainMenuEvent::TextSizeDecrease
                    | MainMenuEvent::TextSizeReset => {
                        context.send_parent(action.clone());
                        // But also act ourselves
                        ChangeTextsizeBehaviour::handle(
                            window,
                            *menu_event,
                            &mut state.config,
                            environment,
                        )
                    }
                    _ => {
                        context.send_parent(action);
                        Effect::NONE
                    }
                }
            }
        },
        PostAction::DroppedMedia(m) => {
            state.dropping_file = false;
            let current = &mut state.images;
            current.extend(m.clone());
            let model = environment.model.clone();

            Effect::future(
                async move {
                    let mut results = Vec::new();
                    for image in m {
                        let i = model
                            .upload_media(&image.path, image.description.clone())
                            .await;
                        results.push((i, image));
                    }
                    results
                },
                PostAction::DroppedMediaUploaded,
            )
        }
        PostAction::DroppedMediaUploaded(m) => {
            for (result, image) in m {
                handle_image_upload(image, result, &mut state.images);
            }
            Effect::NONE
        }
        PostAction::ClearError => {
            state.error_message = None;
            Effect::NONE
        }
    }
}

fn validate_text(instance: Option<Instance>, text: &str) -> (bool, u32, u32) {
    let Some(instance) = instance else {
        return (false, 0, 500)
    };
    let config = instance.configuration;
    let mut current = text.chars().count() as u32;
    use linkify::LinkFinder;
    let finder = LinkFinder::new();
    let links: Vec<_> = finder.links(text).collect();
    for link in links {
        if let Some(s) = config.statuses.characters_reserved_per_url {
            current -= link.as_str().len() as u32;
            current += s;
        }
    }
    let m = config.statuses.max_characters;
    if current > m {
        (false, current, m)
    } else {
        (true, current, m)
    }
}

fn handle_image_upload(
    image: AttachmentMedia,
    result: Result<UploadMedia, String>,
    media: &mut Vec<AttachmentMedia>,
) -> Option<String> {
    match result {
        Ok(m) => {
            for n in media.iter_mut() {
                if n.path == image.path {
                    n.server_id = Some(m.id().to_string());
                    n.is_uploaded = true;
                    break;
                }
            }
            None
        }
        Err(error) => {
            let Some(index) = media.iter().position(|s| s.path == image.path) else {
                return None
            };
            media.remove(index);
            Some(error)
        }
    }
}
