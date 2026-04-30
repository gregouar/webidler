use leptos::{html::*, prelude::*};

use crate::{
    assets::img_asset,
    components::ui::{
        number::{Number, NumberInset},
        tooltip::{StaticTooltip, StaticTooltipPosition},
    },
};

#[component]
pub fn ResourceIcon(
    icon: &'static str,
    name: &'static str,
    description: impl Fn() -> String + Send + Sync + Clone + 'static,
) -> impl IntoView {
    let tooltip = move || {
        view! {
            <div class="flex flex-col xl:space-y-1 w-[20vw] whitespace-normal">
                <div class="font-semibold text-white">{name}</div>
                <div class="text-sm text-zinc-300">{description()}</div>
            </div>
        }
    };
    view! {
        <StaticTooltip tooltip=tooltip position=StaticTooltipPosition::Bottom>
            <img
                draggable="false"
                src=img_asset(icon)
                alt=name
                class="h-[2em] aspect-square drop-shadow-[0_2px_8px_rgba(0,0,0,0.95)] "
            />
        </StaticTooltip>
    }
}

#[component]
pub fn ResourceCounter(
    icon: &'static str,
    name: &'static str,
    description: &'static str,
    value: Signal<f64>,
    w_full: bool,
    text_color: &'static str,
    disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <div
            class="flex-1 text-shadow-md shadow-gray-950
            text-sm xl:text-xl 
            flex justify-center items-center space-x-1"
            class:saturate-10=disabled
        >
            <NumberInset>
                <div class=move || {
                    format!(
                        "font-number font-semibold text-right {} {}",
                        if w_full { "w-[8ch]" } else { "" },
                        if disabled.get() { "text-gray-300" } else { text_color },
                    )
                }>
                    <Number value=value />
                </div>
            </NumberInset>
            <ResourceIcon
                icon
                name
                description=move || {
                    if disabled.get() {
                        format!("{} are disabled in this area.", name)
                    } else {
                        description.to_string()
                    }
                }
            />
        </div>
    }
}

#[component]
pub fn GoldIcon() -> impl IntoView {
    view! {
        <ResourceIcon
            icon="ui/gold.webp"
            name="Gold"
            description=move || {
                "Used during Grind to buy level up for Skills. Total Gold collected during a Grind is also converted to Temple Donations to buy Blessings in Town."
                    .into()
            }
        />
    }
}
#[component]
pub fn GoldCounter(
    #[prop(into)] value: Signal<f64>,
    #[prop(default = false)] w_full: bool,
    #[prop(default= Signal::derive(|| false))] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <ResourceCounter
            text_color="text-amber-200"
            icon="ui/gold.webp"
            name="Gold"
            description="Used during Grind to buy level up for Skills. Total Gold collected during a Grind is also converted to Temple Donations to buy Blessings in Town."
            value
            w_full
            disabled
        />
    }
}

#[component]
pub fn GemsIcon() -> impl IntoView {
    view! {
        <ResourceIcon
            icon="ui/gems.webp"
            name="Gems"
            description=move || {
                "To exchange Items in the Market or craft Items at the Forge, in Town between Grinds. Obtained by killing Champion Monsters."
                    .into()
            }
        />
    }
}
#[component]
pub fn GemsCounter(
    value: Signal<f64>,
    #[prop(default = false)] w_full: bool,
    #[prop(default= Signal::derive(|| false))] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <ResourceCounter
            text_color="text-fuchsia-300"
            icon="ui/gems.webp"
            name="Gems"
            description="To exchange Items in the Market or craft Items at the Forge, in Town between Grinds. Obtained by killing Champion Monsters."
            value
            w_full
            disabled
        />
    }
}

#[component]
pub fn ShardsIcon() -> impl IntoView {
    view! {
        <ResourceIcon
            icon="ui/power_shard.webp"
            name="Power Shards"
            description=move || {
                "To permanently increase power of Passive Skills by Ascending them, in Town between Grinds. Obtained for every 10 new Area Level completed."
                    .into()
            }
        />
    }
}
#[component]
pub fn ShardsCounter(
    value: Signal<f64>,
    #[prop(default = false)] w_full: bool,
    #[prop(default= Signal::derive(|| false))] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <ResourceCounter
            text_color="text-cyan-300"
            icon="ui/power_shard.webp"
            name="Power Shards"
            description="To permanently increase power of Passive Skills by Ascending them, in Town between Grinds. Obtained for every 10 new Area Level completed."
            value
            w_full
            disabled
        />
    }
}
