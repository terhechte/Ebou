use dioxus::prelude::ScopeState;
use dioxus_desktop::{
    tao::{event::WindowEvent, menu::MenuBar, platform::macos::WindowExtMacOS},
    use_wry_event_handler, LogicalSize, WindowBuilder,
};
use std::{cell::RefCell, string::ToString, sync::Arc};

use crate::{environment::storage::UiTab, environment::types::AppEvent, loc};

use super::{
    super::toolbar::{LoggedInToolbar, LoggedOutBar},
    super::types::{ActionFromEvent, MainMenuConfig, MainMenuEvent},
};
use cacao::{
    appkit::{toolbar::Toolbar, window::WindowToolbarStyle},
    foundation::NSUInteger,
};
use dioxus_desktop::tao::platform::macos::WindowBuilderExtMacOS;
use dioxus_desktop::wry::webview::WebviewExtMacOS;
pub use navicula::types::AppWindow;

pub fn default_window() -> WindowBuilder {
    let builder = WindowBuilder::new();
    let s = LogicalSize::new(1200., 775.);

    let menu = mainmenu(MainMenuConfig::default());

    let builder = builder
        .with_title("Ebou")
        .with_theme(Some(dioxus_desktop::tao::window::Theme::Dark))
        .with_menu(menu)
        .with_inner_size(s);

    builder
        .with_automatic_window_tabbing(false)
        .with_title_hidden(true)
}

use std::rc::Rc;

#[derive(Default)]
enum ToolbarType {
    #[default]
    NoneYet,
    LoggedOut(Toolbar<LoggedOutBar>),
    LoggedIn(Toolbar<LoggedInToolbar>),
}

type ToolbarHandlerUpdateCell = Rc<RefCell<Option<Arc<dyn Fn(AppEvent) + Send + Sync>>>>;

#[derive(Clone, Default)]
pub struct Platform {
    /// The current Menu Configuration
    content: Rc<RefCell<MainMenuConfig>>,
    /// The current toolbar
    toolbar: Rc<RefCell<ToolbarType>>,
    /// The handler for the current toolbar. We save it here,
    /// so that further toolbar updates don't need to access it
    toolbar_handler: ToolbarHandlerUpdateCell,
}

impl Platform {
    pub fn update_menu(&self, window: &AppWindow, mutator: impl Fn(&mut MainMenuConfig)) {
        let mut config = self.content.take();
        mutator(&mut config);
        self.content.replace(config);
        let new_menu = mainmenu(config);
        window.set_menu(Some(new_menu))
    }

    pub fn handle_menu_events<A: ActionFromEvent + 'static>(
        &self,
        cx: &ScopeState,
        updater: Arc<dyn Fn(A) + Send + Sync>,
    ) {
        // let updater = cx.schedule_update();
        use_wry_event_handler(cx, move |event, _target| match event {
            dioxus_desktop::tao::event::Event::WindowEvent {
                event: WindowEvent::Focused(a),
                ..
            } => {
                let Some(converted) = A::make_focus_event(*a) else { return};
                updater(converted);
            }
            dioxus_desktop::tao::event::Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                let Some(converted) = A::make_close_window_event() else { return};
                updater(converted);
            }
            dioxus_desktop::tao::event::Event::MenuEvent { menu_id, .. } => {
                let Some(event) = MainMenuEvent::resolve(menu_id) else {
                    return
                };
                let Some(converted) = A::make_menu_event(event) else { return };
                updater(converted);
            }
            _ => (),
        });
    }

    pub fn setup_toolbar(&self, window: &AppWindow) {
        use objc::runtime::Object;
        let toolbar = Toolbar::new("com.ebou.EbouToolbar.LoggedOut", LoggedOutBar::new());
        let webview = window.webview.clone();
        unsafe {
            let native_window: *mut Object = webview.window().ns_window() as *mut Object;
            let _: () = msg_send![native_window, setToolbar:&*toolbar.objc];
            let style: NSUInteger = WindowToolbarStyle::Unified.into();
            let _: () = msg_send![native_window, setToolbarStyle: style];
        }
        *self.toolbar.borrow_mut() = ToolbarType::LoggedOut(toolbar);
    }

    pub fn set_toolbar_handler(&self, handler: std::sync::Arc<dyn Fn(AppEvent) + Send + Sync>) {
        *self.toolbar_handler.borrow_mut() = Some(handler);
    }

    pub fn update_toolbar(
        &self,
        account: &str,
        window: &AppWindow,
        tab: &UiTab,
        has_notifications: bool,
    ) {
        log::trace!("update_toolbar {tab:?}");
        let Some(handler) = self.toolbar_handler.borrow().clone() else {
            log::error!("No toolbar handler set in setup_toolbar");
            return;
        };
        use objc::runtime::Object;
        let toolbar = Toolbar::new(
            "com.ebou.EbouToolbar.LoggedIn",
            LoggedInToolbar::new(
                account.to_string(),
                super::tab_index(tab),
                has_notifications,
                handler,
            ),
        );
        let webview = window.webview.clone();
        unsafe {
            let native_window: *mut Object = webview.window().ns_window() as *mut Object;
            let _: () = msg_send![native_window, setToolbar:&*toolbar.objc];
        }
        *self.toolbar.borrow_mut() = ToolbarType::LoggedIn(toolbar);
    }

    pub fn loggedout_toolbar(&self, window: &AppWindow) {
        use objc::runtime::Object;
        let toolbar = Toolbar::new("com.ebou.EbouToolbar", LoggedOutBar::new());
        let webview = window.webview.clone();
        unsafe {
            let native_window: *mut Object = webview.window().ns_window() as *mut Object;
            let _: () = msg_send![native_window, setToolbar:&*toolbar.objc];
        }
        *self.toolbar.borrow_mut() = ToolbarType::LoggedOut(toolbar);
    }
}

fn mainmenu(config: MainMenuConfig) -> MenuBar {
    use dioxus_desktop::tao::{
        accelerator::Accelerator,
        keyboard::KeyCode,
        keyboard::ModifiersState,
        menu::{AboutMetadata, MenuBar as Menu, MenuItem, MenuItemAttributes},
    };
    let mut menu_bar_menu = Menu::new();
    let about_metadata = AboutMetadata {
        version: Some("0.1.0".to_string()),
        authors: Some(vec!["Benedikt Terhechte".to_string()]),
        website: Some("https://terhech.de".to_string()),
        ..Default::default()
    };

    let mut app_menu = Menu::new();

    app_menu.add_native_item(MenuItem::About(loc!("Ebou").to_string(), about_metadata));
    app_menu.add_native_item(MenuItem::Separator);
    let acc = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::Comma);
    app_menu.add_item(
        MenuItemAttributes::new(loc!("Settings"))
            .with_id(MainMenuEvent::Settings.menu_id())
            .with_accelerators(&acc),
    );
    app_menu.add_native_item(MenuItem::Separator);
    app_menu.add_native_item(MenuItem::Services);
    app_menu.add_native_item(MenuItem::Separator);
    app_menu.add_native_item(MenuItem::Hide);
    app_menu.add_native_item(MenuItem::HideOthers);
    app_menu.add_native_item(MenuItem::ShowAll);
    app_menu.add_native_item(MenuItem::Separator);
    app_menu.add_native_item(MenuItem::Quit);
    menu_bar_menu.add_submenu(loc!("Ebou"), true, app_menu);

    // edit
    let mut file_menu = Menu::new();
    let acc = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyN);
    file_menu.add_item(
        MenuItemAttributes::new(loc!("New Toot"))
            .with_enabled(config.logged_in)
            .with_id(MainMenuEvent::NewPost.menu_id())
            .with_accelerators(&acc),
    );
    let acc = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyD);
    file_menu.add_item(
        MenuItemAttributes::new(loc!("Send Toot"))
            .with_id(MainMenuEvent::PostWindowSubmit.menu_id())
            .with_enabled(config.enable_postwindow && config.logged_in)
            .with_accelerators(&acc),
    );
    // let acc = Accelerator::new(
    //     Some(ModifiersState::SUPER | ModifiersState::SHIFT),
    //     KeyCode::KeyA,
    // );
    // file_menu.add_item(
    //     MenuItemAttributes::new("Attach File")
    //         .with_id(MainMenuEvent::PostWindowAttachFile.menu_id())
    //         .with_enabled(config.enable_postwindow && config.logged_in)
    //         .with_accelerators(&acc),
    // );
    let acc = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyR);
    file_menu.add_item(
        MenuItemAttributes::new(loc!("Reload"))
            .with_id(MainMenuEvent::Reload.menu_id())
            .with_enabled(config.logged_in)
            .with_accelerators(&acc),
    );
    file_menu.add_item(
        MenuItemAttributes::new(loc!("Logout"))
            .with_enabled(config.logged_in)
            .with_id(MainMenuEvent::Logout.menu_id()),
    );
    file_menu.add_native_item(MenuItem::Separator);
    file_menu.add_native_item(MenuItem::CloseWindow);
    menu_bar_menu.add_submenu(loc!("File"), true, file_menu);

    let mut edit_menu = Menu::new();
    edit_menu.add_native_item(MenuItem::Undo);
    edit_menu.add_native_item(MenuItem::Redo);
    edit_menu.add_native_item(MenuItem::Separator);
    edit_menu.add_native_item(MenuItem::Cut);
    edit_menu.add_native_item(MenuItem::Copy);
    edit_menu.add_native_item(MenuItem::Paste);
    edit_menu.add_native_item(MenuItem::Separator);
    edit_menu.add_native_item(MenuItem::SelectAll);
    menu_bar_menu.add_submenu(loc!("Edit"), true, edit_menu);

    let mut view_menu = Menu::new();

    let acc = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::Digit1);
    view_menu.add_item(
        MenuItemAttributes::new(loc!("Timeline"))
            .with_enabled(config.logged_in)
            .with_id(MainMenuEvent::Timeline.menu_id())
            .with_accelerators(&acc),
    );

    let acc = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::Digit2);
    view_menu.add_item(
        MenuItemAttributes::new(loc!("Mentions"))
            .with_enabled(config.logged_in)
            .with_id(MainMenuEvent::Mentions.menu_id())
            .with_accelerators(&acc),
    );

    let acc = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::Digit3);
    view_menu.add_item(
        MenuItemAttributes::new(loc!("Messages"))
            .with_enabled(config.logged_in)
            .with_id(MainMenuEvent::Messages.menu_id())
            .with_accelerators(&acc),
    );

    let acc = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::Digit4);
    view_menu.add_item(
        MenuItemAttributes::new(loc!("More"))
            .with_enabled(config.logged_in)
            .with_id(MainMenuEvent::More.menu_id())
            .with_accelerators(&acc),
    );

    // let acc = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::Digit0);
    // view_menu.add_item(
    //     MenuItemAttributes::new(loc!("Your Toots"))
    //         .with_enabled(config.logged_in)
    //         .with_id(MainMenuEvent::YourToots.menu_id())
    //         .with_accelerators(&acc),
    // );

    view_menu.add_native_item(MenuItem::Separator);

    let acc = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::BracketRight);
    view_menu.add_item(
        MenuItemAttributes::new(loc!("Go to Top"))
            .with_enabled(config.enable_scroll && config.logged_in)
            .with_id(MainMenuEvent::ScrollUp.menu_id())
            .with_accelerators(&acc),
    );
    let acc = Accelerator::new(Some(ModifiersState::SUPER), KeyCode::BracketLeft);
    view_menu.add_item(
        MenuItemAttributes::new(loc!("Go to Bottom"))
            .with_id(MainMenuEvent::ScrollDown.menu_id())
            .with_enabled(config.enable_scroll && config.logged_in)
            .with_accelerators(&acc),
    );

    view_menu.add_native_item(MenuItem::Separator);

    let acc = Accelerator::new(
        Some(ModifiersState::SUPER | ModifiersState::SHIFT),
        KeyCode::Period,
    );
    view_menu.add_item(
        MenuItemAttributes::new(loc!("Zoom In"))
            .with_id(MainMenuEvent::TextSizeIncrease.menu_id())
            .with_accelerators(&acc),
    );
    let acc = Accelerator::new(
        Some(ModifiersState::SUPER | ModifiersState::SHIFT),
        KeyCode::Comma,
    );
    view_menu.add_item(
        MenuItemAttributes::new(loc!("Zoom Out"))
            .with_id(MainMenuEvent::TextSizeDecrease.menu_id())
            .with_accelerators(&acc),
    );
    let acc = Accelerator::new(
        Some(ModifiersState::SUPER | ModifiersState::SHIFT),
        KeyCode::Digit0,
    );
    view_menu.add_item(
        MenuItemAttributes::new(loc!("Reset Zoom"))
            .with_id(MainMenuEvent::TextSizeReset.menu_id())
            .with_accelerators(&acc),
    );

    view_menu.add_native_item(MenuItem::Separator);
    view_menu.add_native_item(MenuItem::EnterFullScreen);
    menu_bar_menu.add_submenu(loc!("View"), true, view_menu);

    let mut window_menu = Menu::new();
    window_menu.add_native_item(MenuItem::Minimize);
    window_menu.add_native_item(MenuItem::Zoom);
    menu_bar_menu.add_submenu(loc!("Window"), true, window_menu);

    let mut help_menu = Menu::new();
    help_menu
        .add_item(MenuItemAttributes::new("Ebou Help").with_id(MainMenuEvent::EbouHelp.menu_id()));
    menu_bar_menu.add_submenu(loc!("Help"), true, help_menu);

    menu_bar_menu
}

pub fn apply_window_background(window: &AppWindow) {
    let webview = window.webview.clone();
    let native_window = webview.window();

    use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
    apply_vibrancy(
        native_window,
        // NSVisualEffectMaterial::UnderWindowBackground,
        NSVisualEffectMaterial::Sidebar,
        // NSVisualEffectMaterial::Sidebar,
        None,
        None,
    )
    .unwrap();

    // Tell the webview to be transparent
    // https://stackoverflow.com/questions/27211561/transparent-background-wkwebview-nsview
    unsafe {
        let native_webview = webview.webview();
        use cacao::foundation::NSNumber;
        use cacao::foundation::NSString;
        use cacao::foundation::NO;
        let n = NSNumber::bool(true);
        let s = NSString::new("drawsTransparentBackground");
        let _: () = msg_send![native_webview, setValue:n forKey:s];
        let _: () = msg_send![native_webview, setHidden: NO];
    }
}

pub fn show_emoji_popup(window: &AppWindow) {
    let webview = window.webview.webview();
    unsafe {
        use cocoa::appkit::NSApp;
        let app = NSApp();
        let _: () = msg_send![app, orderFrontCharacterPalette: webview];
    }
}
/*
    typedef CF_ENUM(CFIndex, CFDateFormatterStyle) {	// date and time format styles
    kCFDateFormatterNoStyle = 0,
    kCFDateFormatterShortStyle = 1,
    kCFDateFormatterMediumStyle = 2,
    kCFDateFormatterLongStyle = 3,
    kCFDateFormatterFullStyle = 4
};
     */
use cocoa::base::id;
use std::ffi::c_char;
pub trait NSDateFormatter: Sized {}

impl NSDateFormatter for id {}

use chrono::{DateTime, Utc};
pub fn format_datetime(datetime: &DateTime<Utc>) -> (String, String) {
    unsafe {
        let tt = datetime.timestamp() as f64;
        let date: id = msg_send![class!(NSDate), dateWithTimeIntervalSince1970: tt];

        let duration = Utc::now().signed_duration_since(*datetime);
        let human = if duration.num_hours() <= 24 {
            let f1: id = msg_send![class!(NSDateFormatter), localizedStringFromDate:date dateStyle:0 timeStyle: 1];

            convert_string(f1)
        } else if duration.num_days() <= 6 {
            datetime.format("%A").to_string()
        } else {
            let f3: id = msg_send![class!(NSDateFormatter), localizedStringFromDate:date dateStyle:1 timeStyle: 0];

            convert_string(f3)
        };

        let ffull: id = msg_send![class!(NSDateFormatter), localizedStringFromDate:date dateStyle:2 timeStyle: 2];
        let dfull = convert_string(ffull);

        (human, dfull)
    }
}

unsafe fn convert_string(nsstring: id) -> String {
    let bytes: *const c_char = msg_send![&*nsstring, UTF8String];
    let bytes = bytes as *const u8;
    let len: usize = msg_send![&*nsstring, lengthOfBytesUsingEncoding: 4];

    let bytes = std::slice::from_raw_parts(bytes, len);
    let str = std::str::from_utf8(bytes).unwrap();
    str.to_string()
}
