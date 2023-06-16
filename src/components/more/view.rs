use super::{
    reducer::{Action, ViewStore},
    MoreReducer,
};
use crate::{
    components::{
        component_stack::{RootTimelineKind, StackReducer},
        sidebar::MoreSelection,
    },
    widgets::*,
};
use dioxus::prelude::*;
use navicula::reducer::ChildReducer;

#[inline_props]
pub fn MoreViewComponent<'a>(cx: Scope<'a>, store: ViewStore<'a>) -> Element<'a> {
    log::trace!(" {:?}", store.selection);
    render! {
        StatusesPageComponent {
            title: "classic timeline",
            store: store,
            provider: store.providers.classic_timeline.clone(),
            hidden: store.selection != MoreSelection::Classic
        }

        StatusesPageComponent {
            title: "yours",
            store: store,
            provider: store.providers.account.clone(),
            hidden: store.selection != MoreSelection::Yours
        }

        StatusesPageComponent {
            title: "bookmarks",
            store: store,
            provider: store.providers.bookmarks.clone(),
            hidden: store.selection != MoreSelection::Bookmarks
        }

        StatusesPageComponent {
            title: "favorites",
            store: store,
            provider: store.providers.favorites.clone(),
            hidden: store.selection != MoreSelection::Favorites
        }

        StatusesPageComponent {
            title: "federated",
            store: store,
            provider: store.providers.public.clone(),
            hidden: store.selection != MoreSelection::Federated
        }

        StatusesPageComponent {
            title: "local",
            store: store,
            provider: store.providers.local.clone(),
            hidden: store.selection != MoreSelection::Local
        }

        StatusesPageComponent {
            title: "follows",
            store: store,
            provider: store.providers.follows.clone(),
            hidden: store.selection != MoreSelection::Followers
        }

        StatusesPageComponent {
            title: "following",
            store: store,
            provider: store.providers.following.clone(),
            hidden: store.selection != MoreSelection::Following
        }
    }
}

#[derive(Props)]
struct StatusesPageComponentProps<'a> {
    #[allow(unused)]
    title: &'a str,
    hidden: bool,
    store: &'a ViewStore<'a>,
    #[props(!optional)]
    provider: Option<RootTimelineKind>,
}

fn StatusesPageComponent<'a>(cx: Scope<'a, StatusesPageComponentProps<'a>>) -> Element<'a> {
    let store = cx.props.store;
    let Some(ref provider) = cx.props.provider else {
        return render!(div {})
    };

    use crate::components::component_stack::{Stack, State};

    render!(HideableView {
        hidden: cx.props.hidden,
        Stack {
            store: store.host_with(
                cx,
                provider,
                State::new
            )
        }
    })
}

impl ChildReducer<MoreReducer> for StackReducer {
    fn to_child(
        message: <MoreReducer as navicula::Reducer>::Message,
    ) -> Option<<Self as navicula::Reducer>::Action> {
        match message {
            super::reducer::Message::AppEvent(a) => {
                Some(crate::components::component_stack::Action::AppEvent(a))
            }
            super::reducer::Message::Selection(_, _) => None,
        }
    }

    fn from_child(
        message: <Self as navicula::Reducer>::DelegateMessage,
    ) -> Option<<MoreReducer as navicula::Reducer>::Action> {
        match message {
            crate::components::component_stack::DelegateMessage::PublicAction(a) => {
                Some(Action::Conversation(a))
            }
            crate::components::component_stack::DelegateMessage::ConversationAction(a) => {
                Some(Action::Conversation(a))
            }
        }
    }
}
