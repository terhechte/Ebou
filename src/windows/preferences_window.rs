#![allow(unused)]
use crate::components::loggedin::Action;
use crate::environment::types::{AppEvent, TimelineDirection};
use crate::environment::{Environment, OpenWindowState};
use crate::loc;
use crate::widgets::*;
use dioxus::prelude::*;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PreferencesChange {
    Direction,
    PostWindow,
}

#[derive(Clone)]
pub struct PreferencesWindowState {}

impl PreferencesWindowState {
    pub fn new() -> Self {
        Self {}
    }
}

impl OpenWindowState for PreferencesWindowState {
    type Action = PreferencesChange;
    fn window<'a, 'b>(
        &'a self,
        cx: Scope<'b>,
        environment: &'a Environment,
        receiver: flume::Receiver<AppEvent>,
        parent_handler: Rc<dyn Fn(Self::Action)>,
    ) -> Element<'b>
    where
        'a: 'b,
    {
        let Ok(current) = environment.repository.config() else {
            return render! {
                h3 {
                    "An Error Occurred"
                }
            }
        };
        let direction = current.direction;

        let inline_postwindow = current.post_window_inline;

        let e1 = environment.clone();
        let e2 = environment.clone();

        let p1 = parent_handler.clone();
        let p2 = parent_handler.clone();

        cx.render(rsx!(div {
            class: "settings-container",
            VStack {
                class: "gap-3",
                TimelineSetting {
                    direction: direction,
                    onchange: move |direction| {
                        let Ok(mut current) = e1.repository.config() else {
                            return
                        };
                        current.direction = direction;
                        e1.repository.set_config(&current);
                        p1(PreferencesChange::Direction);
                    }
                }
                PostInlineSetting {
                    open_inline: inline_postwindow,
                    onchange: move |v| {
                        let Ok(mut current) = e2.repository.config() else {
                            return
                        };
                        current.post_window_inline = v;
                        e2.repository.set_config(&current);
                        p2(PreferencesChange::PostWindow);
                    }
                }
            }
        }))
    }
}

#[inline_props]
fn TimelineSetting<'a>(
    cx: Scope<'a>,
    direction: TimelineDirection,
    onchange: EventHandler<'a, TimelineDirection>,
) -> Element<'a> {
    let stop = if direction == &TimelineDirection::NewestTop {
        "true"
    } else {
        "false"
    };
    let sbottom = if direction == &TimelineDirection::NewestBottom {
        "true"
    } else {
        "false"
    };

    render! {
        HStack {
            class: "justify-content-between align-items-center",
            Label {
                style: TextStyle::Secondary,
                loc!("Account Timeline Direction")
            }
            select {
                onchange: move |evt| {
                    let Ok(d) = TimelineDirection::from_str(&evt.value) else {
                        return
                    };
                    onchange.call(d);
                },
                option {
                    value: "bottom",
                    selected: sbottom,
                    loc!("Newest at the Bottom (Default)")
                }
                option {
                    value: "top",
                    selected: stop,
                    loc!("Newest at the Top")
                }
            }
        }
    }
}

#[inline_props]
fn PostInlineSetting<'a>(
    cx: Scope<'a>,
    open_inline: bool,
    onchange: EventHandler<'a, bool>,
) -> Element<'a> {
    render! {
        HStack {
            class: "justify-content-between align-items-center",
            input {
                r#type: "checkbox",
                id: "open_inline",
                checked: "{open_inline}",
                oninput: move |evt| {
                    onchange.call(evt.value.parse().unwrap());
                },
            }
            label {
                class: "label-secondary",
                r#for: "open_inline",
                loc!("Always open New Toot Window inline")
            }
        }
    }
}

impl FromStr for TimelineDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "top" => Ok(TimelineDirection::NewestTop),
            "bottom" => Ok(TimelineDirection::NewestBottom),
            _ => Err("Unknown Direction".to_string()),
        }
    }
}
