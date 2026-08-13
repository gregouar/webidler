use leptos::{html::*, prelude::*};

use crate::{
    assets::img_asset,
    components::ui::{
        number::{Number, NumberInset, format_number},
        tooltip::{StaticTooltip, StaticTooltipPosition},
    },
};

pub type ResourceReward = (u64, f64, f64);

pub fn show_resource_reward(reward: RwSignal<ResourceReward>, gold: f64, gems: f64) {
    reward.update(|reward| {
        *reward = (reward.0.wrapping_add(1), gold, gems);
    });
}

#[component]
pub fn ResourceRewardOverlay(#[prop(into)] reward: Signal<ResourceReward>) -> impl IntoView {
    view! {
        <For each=move || vec![reward.get()] key=|reward| reward.0 let(reward)>
            <Show when=move || { reward.1 > 0.0 }>
                <div class="
                reward-float gold-text text-amber-400 text-lg xl:text-2xl text-shadow-md
                absolute left-1/2 transform -translate-y-1/2 -translate-x-1/2
                flex items-center gap-1 font-number"
                style="top: calc(50% - 1.5rem);">
                    <span>+{format_number(reward.1)}</span>
                    <img
                        draggable="false"
                        src=img_asset("ui/gold.webp")
                        alt="Gold"
                        class="h-[2em] aspect-square"
                    />
                </div>
            </Show>

            <Show when=move || { reward.2 > 0.0 }>
                <div class="
                reward-float gems-text text-fuchsia-400 text-lg xl:text-2xl text-shadow-md
                absolute left-1/2 transform -translate-y-1/2 -translate-x-1/2
                flex items-center gap-1 font-number"
                style="top: calc(50% + 1.5rem);">
                    <span>+{format_number(reward.2)}</span>
                    <img
                        draggable="false"
                        src=img_asset("ui/gems.webp")
                        alt="Gems"
                        class="h-[1.2em] aspect-square"
                    />
                </div>
            </Show>
        </For>
    }
}

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
    #[prop(default = Signal::derive(|| None))] disabled_description: Signal<Option<String>>,
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
                        if disabled.get() { "text-zinc-300" } else { text_color },
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
                        disabled_description
                            .get()
                            .unwrap_or_else(|| format!("{} are disabled in this area.", name))
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
    #[prop(default = Signal::derive(|| None))] disabled_description: Signal<Option<String>>,
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
            disabled_description
        />
    }
}
