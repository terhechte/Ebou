use std::sync::Arc;

use cacao::button::Button;

use cacao::appkit::toolbar::{
    ItemIdentifier, Toolbar, ToolbarDelegate, ToolbarDisplayMode, ToolbarItem,
};

use crate::environment::types::{AppEvent, MainMenuEvent};
use crate::loc;

const ACCOUNT_BUTTON: &str = "AccountButton";
const SEGMENTED_CONTROL: &str = "SegmentedControl";
const NEW_TOOT_BUTTON: &str = "NewTootButton";
const RELOAD_BUTTON: &str = "ReloadButton";

#[derive(Debug)]
pub struct LoggedOutBar;

impl LoggedOutBar {
    pub fn new() -> Self {
        Self
    }

    fn item_identifiers(&self) -> Vec<ItemIdentifier> {
        vec![ItemIdentifier::Space]
    }
}

impl ToolbarDelegate for LoggedOutBar {
    const NAME: &'static str = "EbouToolbarLoggedout";

    fn did_load(&mut self, toolbar: Toolbar) {
        toolbar.set_display_mode(ToolbarDisplayMode::IconOnly);
    }

    fn allowed_item_identifiers(&self) -> Vec<ItemIdentifier> {
        self.item_identifiers()
    }

    fn default_item_identifiers(&self) -> Vec<ItemIdentifier> {
        self.item_identifiers()
    }

    fn item_for(&self, _identifier: &str) -> &ToolbarItem {
        std::unreachable!();
    }
}

#[derive(Debug)]
pub struct LoggedInToolbar {
    account_item: ToolbarItem,
    control_item: ToolbarItem,
    new_toot_item: ToolbarItem,
    reload_item: ToolbarItem,
}

impl LoggedInToolbar {
    pub fn new(
        account_url: String,
        selection: super::ToolbarSelection,
        has_notifications: bool,
        sender: Arc<dyn Fn(AppEvent) + Send + Sync>,
    ) -> Self {
        use cacao::appkit::segmentedcontrol::SegmentedControl;
        use cacao::foundation::{NSArray, NSURL};
        use cacao::image::{Image, SFSymbol};

        // Profile Button
        let url = if url::Url::parse(&account_url).is_ok() {
            NSURL::with_str(&account_url)
        } else {
            NSURL::with_str("https://terhech.de/ebou/images/logo2.png")
        };
        let image = Image::with_contents_of_url(url);
        let mut account_item = ToolbarItem::new(ACCOUNT_BUTTON);
        account_item.set_image(image);
        // let cloned = sender.clone();
        account_item.set_action(move |_| {
            // For now, for profile tap, we go to our toots
            // (*cloned)(AppEvent::MenuEvent(MainMenuEvent::YourToots));
        });

        let nicon = if has_notifications {
            SFSymbol::BellBadgeFill
        } else {
            SFSymbol::BellFill
        };

        // Segmented Control
        let images = NSArray::from(vec![
            &*Image::symbol(SFSymbol::ListAndFilm, loc!("Grouped Timelines")).0,
            &*Image::symbol(nicon, loc!("Notifications")).0,
            // &*Image::symbol(SFSymbol::MessageBadgeFilledFill, loc!("Messages")).0,
            &*Image::symbol(SFSymbol::MessageFill, loc!("Messages")).0,
            &*Image::symbol(SFSymbol::PersonCropCircle, loc!("More")).0,
        ]);
        let mut control = SegmentedControl::new(
            images,
            cacao::appkit::segmentedcontrol::TrackingMode::SelectOne,
        );
        control.select_segment(selection as u8 as u64);

        control.set_tooltip_segment(0, loc!("Timelines & Lists"));
        control.set_tooltip_segment(1, loc!("Notifications"));
        control.set_tooltip_segment(2, loc!("Direct Messages"));
        control.set_tooltip_segment(3, loc!("Followers, Classical Timelines & More"));

        let cloned = sender.clone();
        control.set_action(move |index| match index {
            0 => (*cloned)(AppEvent::MenuEvent(MainMenuEvent::Timeline)),
            1 => (*cloned)(AppEvent::MenuEvent(MainMenuEvent::Mentions)),
            2 => (*cloned)(AppEvent::MenuEvent(MainMenuEvent::Messages)),
            3 => (*cloned)(AppEvent::MenuEvent(MainMenuEvent::More)),
            _ => {
                log::error!("Invalid toolbar index {index}");
            }
        });
        let mut control_item = ToolbarItem::new(SEGMENTED_CONTROL);
        control_item.set_segmented_control(control);

        // New Toot
        let mut new_toot_item = ToolbarItem::new(NEW_TOOT_BUTTON);
        new_toot_item.set_title("New Toot");
        let mut button = Button::new("");
        button.set_image(Image::symbol(SFSymbol::SquareAndPencil, loc!("New Toot")));
        let cloned = sender.clone();
        button.set_action(move |_| {
            (*cloned)(AppEvent::MenuEvent(MainMenuEvent::NewPost));
        });
        new_toot_item.set_button(button);

        let mut reload_item = ToolbarItem::new(RELOAD_BUTTON);
        let mut button = Button::new("");
        button.set_image(Image::symbol(SFSymbol::ArrowClockwise, loc!("Reload")));
        let cloned = sender.clone();
        button.set_action(move |_| {
            (*cloned)(AppEvent::MenuEvent(MainMenuEvent::Reload));
        });
        reload_item.set_button(button);

        LoggedInToolbar {
            account_item,
            control_item,
            new_toot_item,
            reload_item,
        }
    }

    fn item_identifiers(&self) -> Vec<ItemIdentifier> {
        vec![
            ItemIdentifier::Custom(ACCOUNT_BUTTON),
            ItemIdentifier::Space,
            ItemIdentifier::Space,
            ItemIdentifier::Custom(SEGMENTED_CONTROL),
            ItemIdentifier::Space,
            ItemIdentifier::Space,
            ItemIdentifier::Custom(RELOAD_BUTTON),
            ItemIdentifier::FlexibleSpace,
            ItemIdentifier::Custom(NEW_TOOT_BUTTON),
        ]
    }
}

impl ToolbarDelegate for LoggedInToolbar {
    const NAME: &'static str = "EbouToolbarLoggedIn";

    fn did_load(&mut self, toolbar: Toolbar) {
        toolbar.set_display_mode(ToolbarDisplayMode::IconOnly);
    }

    fn allowed_item_identifiers(&self) -> Vec<ItemIdentifier> {
        self.item_identifiers()
    }

    fn default_item_identifiers(&self) -> Vec<ItemIdentifier> {
        self.item_identifiers()
    }

    fn item_for(&self, identifier: &str) -> &ToolbarItem {
        match identifier {
            ACCOUNT_BUTTON => &self.account_item,
            SEGMENTED_CONTROL => &self.control_item,
            NEW_TOOT_BUTTON => &self.new_toot_item,
            RELOAD_BUTTON => &self.reload_item,
            _ => {
                std::unreachable!();
            }
        }
    }
}
