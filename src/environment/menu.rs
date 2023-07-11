use dioxus::prelude::*;
use navicula::logic::Drops;
use navicula::{Reducer, ViewStore};

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use std::any::Any;

use crate::environment::platform::AppWindow;

type MenuEventId = u32;

type ContextMenuEventIdMapHandler =
    Arc<RwLock<HashMap<String, Arc<dyn Fn(MenuEventId) + Send + Sync>>>>;
type ContextMenuEventIdMapAction =
    Arc<RwLock<HashMap<String, HashMap<MenuEventId, Box<dyn Any + Send + Sync>>>>>;

lazy_static::lazy_static! {
    /// Action to handler
    static ref CTX_STATE_A: ContextMenuEventIdMapHandler = Arc::default();

    /// Action to MenuId to Action
    static ref CTX_STATE: ContextMenuEventIdMapAction = Arc::default();
}

pub trait ScopeExt {
    fn window(&self) -> &AppWindow;
}

type Payload = Box<dyn Any + Send + Sync>;

#[allow(unused)]
enum ContextMenuKind {
    Checkbox {
        title: String,
        checked: bool,
        payload: Payload,
    },
    Item {
        title: String,
        payload: Payload,
    },
    Submenu {
        title: String,
        children: Vec<ContextMenuItem>,
    },
    Separator,
}

pub struct ContextMenuItem {
    // Only one field to hide the actual enum
    kind: ContextMenuKind,
}

pub struct ContextMenu<A> {
    title: String,
    enabled: bool,
    children: Vec<ContextMenuItem>,
    _m: std::marker::PhantomData<A>,
}

impl<A> ContextMenu<A> {
    pub fn new(title: impl AsRef<str>, enabled: bool, children: Vec<ContextMenuItem>) -> Self {
        Self {
            title: title.as_ref().to_string(),
            enabled,
            children,
            _m: std::marker::PhantomData,
        }
    }
}

#[allow(unused)]
impl ContextMenuItem {
    pub fn item<T: Send + Sync + 'static>(title: impl AsRef<str>, payload: T) -> Self {
        Self {
            kind: ContextMenuKind::Item {
                title: title.as_ref().to_string(),
                payload: Box::new(payload),
            },
        }
    }
    pub fn checkbox<T: Send + Sync + 'static>(
        title: impl AsRef<str>,
        checked: bool,
        payload: T,
    ) -> Self {
        Self {
            kind: ContextMenuKind::Checkbox {
                title: title.as_ref().to_string(),
                checked,
                payload: Box::new(payload),
            },
        }
    }
    pub fn submenu(title: impl AsRef<str>, children: Vec<ContextMenuItem>) -> Self {
        Self {
            kind: ContextMenuKind::Submenu {
                title: title.as_ref().to_string(),
                children,
            },
        }
    }

    pub fn separator() -> Self {
        Self {
            kind: ContextMenuKind::Separator,
        }
    }
}

#[cfg(not(target_os = "ios"))]
impl ContextMenuItem {
    fn build(self, into: &mut muda::Submenu, actions: &mut HashMap<MenuEventId, Payload>) {
        use muda::{CheckMenuItem, MenuItem, PredefinedMenuItem, Submenu};
        match self.kind {
            ContextMenuKind::Checkbox {
                title,
                checked,
                payload,
            } => {
                let item = CheckMenuItem::new(title, true, checked, None);
                actions.insert(item.id(), payload);
                into.append(&item);
            }
            ContextMenuKind::Item { title, payload } => {
                let item = MenuItem::new(title, true, None);
                actions.insert(item.id(), payload);
                into.append(&item);
            }
            ContextMenuKind::Submenu { title, children } => {
                let mut sub_menu = Submenu::new(title, true);
                for child in children {
                    child.build(&mut sub_menu, actions);
                }
                into.append(&sub_menu);
            }
            ContextMenuKind::Separator => {
                into.append(&PredefinedMenuItem::separator());
            }
        }
    }
}

pub trait ViewStoreContextMenu<'a> {
    type Action;
    fn context_menu<T>(
        &self,
        cx: Scope<'a, T>,
        event: &'a MouseData,
        menu: ContextMenu<Self::Action>,
    );
}

impl<'a, R: Reducer> ViewStoreContextMenu<'a> for ViewStore<'a, R> {
    type Action = R::Action;
    fn context_menu<T>(
        &self,
        cx: Scope<'a, T>,
        event: &'a MouseData,
        menu: ContextMenu<Self::Action>,
    ) {
        let window = AppWindow::retrieve(cx);
        let sender = self.sender();
        //context_menu(cx, sender, window, event, menu)
        context_menu(cx, Arc::new(move |a| sender.send(a)), window, event, menu)
    }
}

#[cfg(not(target_os = "ios"))]
pub fn context_menu<A: Clone + std::fmt::Debug + Send + 'static, T>(
    cx: Scope<T>,
    //sender: ActionSender<A>,
    sender: Arc<dyn Fn(A) + Send + Sync>,
    window: AppWindow,
    event: &MouseData,
    menu: ContextMenu<A>,
) {
    let id = cx.scope_id().0;
    let action_key = std::any::type_name::<A>().to_string();
    let action_key = format!("{}-{}", id, action_key);

    // Setup the menu handler

    crate::environment::menu::setup_menu_handler::<A>(
        id,
        Some(Arc::new(move |ev| {
            if let Some(action) = resolve_current_action(id, ev) {
                // sender.send(action);
                sender(action);
            }
        })),
    );

    // remove the handler on context drop
    let cloned = id;
    Drops::action(cx, move || {
        crate::environment::menu::setup_menu_handler::<A>(cloned, None);
    });

    // Show the menu
    show_context_menu(window, event, menu, action_key);
}

#[cfg(target_os = "ios")]
pub fn context_menu<A: Clone + std::fmt::Debug + Send + 'static, T>(
    cx: Scope<T>,
    //sender: ActionSender<A>,
    sender: Arc<dyn Fn(A) + Send + Sync>,
    window: AppWindow,
    event: &MouseData,
    menu: ContextMenu<A>,
) {
}

#[cfg(not(target_os = "ios"))]
fn show_context_menu<A>(
    window: AppWindow,
    event: &MouseData,
    menu: ContextMenu<A>,
    action_key: String,
) {
    use muda::{ContextMenu, Submenu};

    let mut context_menu = Submenu::new(menu.title, menu.enabled);
    let mut actions = HashMap::new();
    for child in menu.children {
        child.build(&mut context_menu, &mut actions);
    }

    if let Ok(mut t) = CTX_STATE.write() {
        let Some(entry) = t.get_mut(&action_key) else {
                println!("setup_menu_handler was not called for action {action_key}. No handler registered");
                return;
            };
        *entry = actions;
    }

    #[cfg(target_os = "macos")]
    {
        use dioxus_desktop::wry::webview::WebviewExtMacOS;
        use objc::runtime::Object;
        let scale_factor = window.scale_factor();
        let vx = window.webview.webview();
        let _pos = event.client_coordinates();

        let xp = unsafe {
            use cocoa::appkit::NSApp;
            use cocoa::foundation::NSPoint;
            let app = NSApp();
            let o: *mut Object = msg_send![app, currentEvent];
            let mut p: NSPoint = msg_send![o, locationInWindow];
            p.x += 5.;
            p.y -= 12.;
            p
        };

        context_menu.show_context_menu_for_nsview(vx as _, xp.x * scale_factor, xp.y * scale_factor)
    }
    #[cfg(target_os = "windows")]
    unsafe {
        use dioxus_desktop::wry::webview::WebviewExtWindows;
        use windows::Win32::Foundation::HWND;
        let mut hwnd: HWND = HWND(0);
        let controller = window.webview.controller();
        if let Err(e) = controller.ParentWindow(&mut hwnd as *mut HWND) {
            log::error!("Could not target window: {e:?}");
        }
        // just like on macOS, on windows the pos is completely wrong. Have to take the current cursor pos
        let pos = {
            use winapi::shared::windef::POINT;
            use winapi::um::winuser::GetCursorPos;
            let mut point = POINT { x: 0, y: 0 };
            if GetCursorPos(&mut point as *mut _) != 0 {
                (point.x as f64, point.y as f64)
            } else {
                // fallback
                let pos = event.client_coordinates();
                (pos.x, pos.y)
            }
        };
        context_menu.show_context_menu_for_hwnd(hwnd.0, pos.0, pos.1);
    }
}

#[cfg(target_os = "ios")]
fn show_context_menu<A>(
    window: AppWindow,
    event: &MouseData,
    menu: ContextMenu<A>,
    action_key: String,
) {
}

#[cfg(not(target_os = "ios"))]
pub fn setup_menu_handler<A>(
    id: usize,
    schedule_update: Option<Arc<dyn Fn(MenuEventId) + Send + Sync>>,
) {
    let action_key = format!("{}-{}", id, std::any::type_name::<A>());
    // If the `HashMap` is still empty, set up the event handler, otherwise
    // insert into the hashmap
    let Some(mut m) = CTX_STATE.write().ok() else {
            panic!("Could not get a write handle into the map");
        };

    if m.is_empty() {
        use muda::MenuEvent;
        MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
            // iterate over all actions and call them. only those with a matching
            // event will trigger. this is a bit expensive, but only happens on
            // menu events.
            let Ok(r) = CTX_STATE_A.read() else {
                    return;
                };
            for (_, v) in r.iter() {
                (v)(event.id)
            }
        }));
    }

    // register
    m.insert(action_key.clone(), HashMap::new());
    let Ok(mut actions) = CTX_STATE_A.write() else {
            panic!("Could not get a write handle into the action map");
        };
    if let Some(n) = schedule_update {
        actions.insert(action_key, n);
    } else {
        actions.remove(&action_key);
    }
}

#[cfg(target_os = "ios")]
pub fn setup_menu_handler<A>(
    id: usize,
    schedule_update: Option<Arc<dyn Fn(MenuEventId) + Send + Sync>>,
) {
}

pub fn resolve_current_action<Action: std::fmt::Debug + Clone + 'static>(
    id: usize,
    menu_event_id: MenuEventId,
) -> Option<Action> {
    let action_key = format!("{}-{}", id, std::any::type_name::<Action>());
    let Some(mut s) = CTX_STATE.write().ok() else {
        return None;
    };
    let Some(m) = s.get_mut(&action_key) else {
        return None;
    };
    let Some(mx) = m.remove(&menu_event_id) else {
        return None;
    };

    let Ok(value) = mx.downcast::<Action>() else {
        return None;
    };
    Some(*value)
}
