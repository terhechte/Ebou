use super::conversation_helpers::{build_conversation, Conversation};
use crate::environment::Environment;
use crate::view_model::StatusId;
use crate::PublicAction;
use navicula::Effect;

#[derive(Clone, Default)]
pub struct State {
    pub conversation_id: StatusId,
    pub conversation: Option<Conversation>,
    pub is_loading: bool,
    pub error: Option<String>,
}

pub type ViewStore<'a> = navicula::ViewStore<'a, super::ConversationReducer>;

#[derive(Clone)]
pub enum Action {
    Initial,
    LoadConversation,
    LoadedConversation(Result<Conversation, String>),
    ApplyConversation,
    Public(PublicAction),
    SelectConversation(StatusId),
    Close,
}

impl std::fmt::Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initial => write!(f, "Initial"),
            Self::LoadConversation => write!(f, "LoadConversation"),
            Self::ApplyConversation => write!(f, "ApplyConversation"),
            Self::LoadedConversation(_arg0) => f.debug_tuple("LoadedConversation").finish(),
            Self::Public(arg0) => f.debug_tuple("Public").field(arg0).finish(),
            Self::SelectConversation(arg0) => {
                f.debug_tuple("SelectConversation").field(arg0).finish()
            }
            Self::Close => write!(f, "Close"),
        }
    }
}

impl State {
    pub fn new(id: StatusId) -> Self {
        Self {
            conversation_id: id,
            ..Default::default()
        }
    }
}

pub fn reduce<'a>(
    context: &'a impl navicula::types::MessageContext<Action, PublicAction, ()>,
    action: Action,
    state: &'a mut State,
    environment: &'a Environment,
) -> Effect<'static, Action> {
    log::trace!("{action:?}");
    match action {
        Action::Initial => {
            // FIXME: The uniqueness of the subscription is invalid. If two Conversations are open,
            // they'll have the same id. But we also can't use the `conversation_id` as that can
            // change via `SelectConversation`. We also can't use the number of known conversations,
            // as this can lead to keys being used twice (if a user opens and closes another conv).
            // So really, instead, the subscription should - like an ecs - handle its own ids

            // TMP: We hack something
            let id = environment.storage.with(|s| {
                format!(
                    "conversation-{}-{:?}",
                    s.conversations.len(),
                    state.conversation_id
                )
            });
            Effect::merge2(
                Effect::action(Action::LoadConversation),
                // If the storage changes, reload our conversation. the user might have bookmarked something.
                // FIXME: Fine-grained subscriptions
                environment
                    .storage
                    .subscribe(id, context, |_| Action::ApplyConversation),
            )
        }
        Action::SelectConversation(a) => {
            state.conversation_id = a;
            Effect::action(Action::LoadConversation)
        }
        Action::LoadConversation => {
            state.is_loading = true;
            let model = environment.model.clone();
            let id = state.conversation_id.0.clone();
            Effect::future(
                async move { build_conversation(&model, id).await },
                Action::LoadedConversation,
            )
        }
        Action::LoadedConversation(result) => {
            state.is_loading = false;
            let Ok(selected_conv) = result else {
                return Effect::NONE
            };
            environment.storage.with_mutation(|mut storage| {
                storage
                    .conversations
                    .insert(selected_conv.status(), selected_conv);
            });
            Effect::action(Action::ApplyConversation)
        }
        Action::ApplyConversation => {
            environment.storage.with(|storage| {
                state.conversation = storage.conversation(&state.conversation_id).cloned();
            });

            let Some(id) = state.conversation.as_ref().map(|e| e.status().dom_id()) else {
                return Effect::NONE;
            };

            // Scroll to the selected item
            return Effect::ui(format!(
                r#"
            window.setTimeout(() => {{
                document.getElementById("conv-{id}").scrollIntoView({{ behavior: "smooth", block: "nearest" }});
            }}, 250);
            "#
            ));
        }
        Action::Close => {
            environment.storage.with_mutation(|mut storage| {
                storage.conversations.remove(&state.conversation_id);
            });
            context.send_parent(PublicAction::Close);
            Effect::NONE
        }
        Action::Public(action) => {
            // : Can this be right? Shouldn't this be sent to the children..?
            context.send_parent(action);
            Effect::NONE
        }
    }
}
