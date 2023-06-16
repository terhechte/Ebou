use dioxus::prelude::*;

mod labels;
pub use labels::*;

mod stacks;
pub use stacks::*;

mod spinner;
pub use spinner::*;

mod errors;
pub use errors::*;

mod buttons;
pub use buttons::*;

mod textcontent;
pub use textcontent::*;

mod status;
pub use status::*;

mod sidebar_text;
pub use sidebar_text::*;

mod splitview;
pub use splitview::SplitViewComponent;

mod segmented_control;
pub use segmented_control::*;

mod hideable_view;
pub use hideable_view::*;

#[inline_props]
pub fn FormattedTime<'a>(
    cx: Scope<'a>,
    human_time: &'a str,
    full_time: &'a str,
    align: TextAlign,
    vertical_alignment: Option<VerticalTextAlign>,
) -> Element<'a> {
    let vertical_alignment = vertical_alignment.unwrap_or_default();
    cx.render(rsx!(
        Label {
            class: "time",
            style: TextStyle::Tertiary,
            alignment: *align,
            vertical_alignment: vertical_alignment,
            force_singleline: true,
            title: "{full_time}",
            "{human_time}"
        }
    ))
}
