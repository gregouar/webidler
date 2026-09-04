use leptos::{html::*, prelude::*};
use shared::data::temple::BenedictionEffect;

use crate::components::{
    chat::chat_context::ChatContext,
    data_context::DataContext,
    events::{EventsContext, Key},
    shared::{
        inventory::InventoryEquipFilter,
        resources::{GemsCounter, GoldCounter, ShardsCounter},
    },
    town::TownContext,
    ui::{
        buttons::MenuButton,
        fullscreen::FullscreenButton,
        header::BaseHeaderMenu,
        tutorial_popup::{TutorialPopup, TutorialPopupPosition},
        wiki::WikiButton,
    },
};

#[component]
pub fn HeaderMenu() -> impl IntoView {
    let town_context: TownContext = expect_context();
    let data_context: DataContext = expect_context();
    let chat_context: ChatContext = expect_context();
    let events_context: EventsContext = expect_context();

    let show_ascension_tutorial = Signal::derive(move || {
        town_context.character.read().resource_shards >= 1.0
            && town_context
                .passives_tree_ascension
                .read()
                .ascended_nodes
                .values()
                .all(|level| *level == 0)
            && !town_context.open_ascend.get()
    });
    let show_temple_tutorial = Signal::derive(move || {
        town_context.character.read().resource_gold >= 100.0
            && !has_bought_extra_skill_slot(town_context)
            && !town_context.open_temple.get()
            && !town_context.open_ascend.get()
            && !show_ascension_tutorial.get()
    });
    let show_skill_mastery_tutorial = Signal::derive(move || {
        has_first_unspent_mastery_point(town_context, data_context)
            && !town_context.open_skill_masteries.get()
            && !town_context.open_temple.get()
            && !town_context.open_ascend.get()
            && !show_ascension_tutorial.get()
            && !show_temple_tutorial.get()
    });

    let gold = Signal::derive(move || town_context.character.read().resource_gold);
    let gems = Signal::derive(move || town_context.character.read().resource_gems);
    let shards = Signal::derive(move || town_context.character.read().resource_shards);

    let navigate_quit = {
        let navigate = leptos_router::hooks::use_navigate();
        move |_| {
            navigate("/user-dashboard", Default::default());
        }
    };

    let disable_panels = Signal::derive(move || town_context.character.read().max_area_level == 0);
    let disable_trade = Signal::derive(move || town_context.character.read().is_ssf);

    let open_inventory = move || {
        town_context
            .open_inventory
            .set(!town_context.open_inventory.get_untracked());
        town_context.open_ascend.set(false);
        town_context.open_temple.set(false);
        town_context.open_skill_masteries.set(false);
        town_context.open_skill_mastery_details.set(false);
        town_context.equip_filter.set(InventoryEquipFilter::Slot);
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('i')) {
            open_inventory()
        }
    });

    let open_stash = move || {
        town_context
            .open_stash
            .set(!town_context.open_stash.get_untracked());
        town_context.open_ascend.set(false);
        town_context.open_market.set(false);
        town_context.open_forge.set(false);
        town_context.open_temple.set(false);
        town_context.open_skill_masteries.set(false);
        town_context.open_skill_mastery_details.set(false);
        town_context.open_inventory.set(false);
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('s')) {
            open_stash()
        }
    });

    let open_market = move || {
        town_context
            .open_market
            .set(!town_context.open_market.get_untracked());
        town_context.open_ascend.set(false);
        town_context.open_forge.set(false);
        town_context.open_temple.set(false);
        town_context.open_skill_masteries.set(false);
        town_context.open_skill_mastery_details.set(false);
        town_context.open_inventory.set(false);
        town_context.open_stash.set(false);
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('m')) {
            open_market()
        }
    });

    let open_forge = move || {
        town_context
            .open_forge
            .set(!town_context.open_forge.get_untracked());
        town_context.open_market.set(false);
        town_context.open_ascend.set(false);
        town_context.open_temple.set(false);
        town_context.open_skill_masteries.set(false);
        town_context.open_skill_mastery_details.set(false);
        town_context.open_inventory.set(false);
        town_context.open_stash.set(false);
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('r')) {
            open_forge()
        }
    });

    let open_ascend = move || {
        town_context
            .open_ascend
            .set(!town_context.open_ascend.get_untracked());
        town_context.open_market.set(false);
        town_context.open_forge.set(false);
        town_context.open_temple.set(false);
        town_context.open_skill_masteries.set(false);
        town_context.open_skill_mastery_details.set(false);
        town_context.open_inventory.set(false);
        town_context.open_stash.set(false);
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('p')) {
            open_ascend()
        }
    });

    let open_temple = move || {
        town_context
            .open_temple
            .set(!town_context.open_temple.get_untracked());
        town_context.open_ascend.set(false);
        town_context.open_market.set(false);
        town_context.open_forge.set(false);
        town_context.open_inventory.set(false);
        town_context.open_stash.set(false);
        town_context.open_skill_masteries.set(false);
        town_context.open_skill_mastery_details.set(false);
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('t')) {
            open_temple()
        }
    });

    let open_skill_masteries = move || {
        town_context
            .open_skill_masteries
            .set(!town_context.open_skill_masteries.get_untracked());
        town_context.open_ascend.set(false);
        town_context.open_market.set(false);
        town_context.open_forge.set(false);
        town_context.open_inventory.set(false);
        town_context.open_skill_mastery_details.set(false);
        town_context.open_stash.set(false);
        town_context.open_temple.set(false);
    };

    view! {
        <BaseHeaderMenu>
            <div class="flex justify-start space-x-1 xl:space-x-2">
                <FullscreenButton />
                <MenuButton on:click=move |_| {
                    town_context.open_settings.set(!town_context.open_settings.get_untracked())
                }>"⚙"</MenuButton>
                <MenuButton
                    class:hidden
                    class:xl:inline
                    on:click=move |_| {
                        chat_context.opened.set(!chat_context.opened.get_untracked())
                    }
                >
                    "🗪"
                </MenuButton>
                <WikiButton />
            </div>
            <div class="flex-1 flex justify-around items-center">
                <GoldCounter value=gold w_full=true />
                <GemsCounter value=gems w_full=true />
                <ShardsCounter value=shards w_full=true />
            </div>
            <div class="flex justify-end space-x-1 xl:space-x-2">
                <MenuButton on:click=move |_| open_inventory() disabled=disable_panels>
                    <span class="inline xl:hidden">"Inv."</span>
                    <span class="hidden xl:inline font-variant:small-caps">"Inventory"</span>
                </MenuButton>
                <MenuButton on:click=move |_| open_stash() disabled=disable_panels>
                    "Stash"
                </MenuButton>
                {move || {
                    (!disable_trade.get())
                        .then(|| {
                            view! {
                                <MenuButton on:click=move |_| open_market() disabled=disable_panels>
                                    "Market"
                                    {move || {
                                        (town_context.market_stash.read().resource_gems > 0.0)
                                            .then_some(" [!]")
                                    }}

                                </MenuButton>
                            }
                        })
                }}
                <MenuButton on:click=move |_| open_forge() disabled=disable_panels>
                    "Forge"
                </MenuButton>
                <TutorialPopup
                    show=show_ascension_tutorial
                    position=TutorialPopupPosition::BelowRight
                    message="Spend a Power Shard to permanently Ascend a passive node."
                >
                    <MenuButton on:click=move |_| open_ascend() disabled=disable_panels>
                        <span class="inline xl:hidden">"Pas"</span>
                        <span class="hidden xl:inline font-variant:small-caps">"Passives"</span>
                    </MenuButton>
                </TutorialPopup>
                <TutorialPopup
                    show=show_temple_tutorial
                    position=TutorialPopupPosition::BelowRight
                    message="Buy a Skill Slot to use another skill during Grinds."
                >
                    <MenuButton on:click=move |_| open_temple() disabled=disable_panels>
                        "Temple"
                    </MenuButton>
                </TutorialPopup>
                <TutorialPopup
                    show=show_skill_mastery_tutorial
                    position=TutorialPopupPosition::BelowRight
                    message="Open Skills to spend your Skill Mastery Point on an upgrade."
                >
                    <MenuButton
                        on:click=move |_| open_skill_masteries()
                        disabled=move || {
                            disable_panels.get()
                                || town_context.player_skill_masteries.read().masteries.is_empty()
                        }
                    >
                        "Skills"
                    </MenuButton>
                </TutorialPopup>
                <MenuButton on:click=navigate_quit>"Back"</MenuButton>
            </div>
        </BaseHeaderMenu>
    }
}

fn has_bought_extra_skill_slot(town_context: TownContext) -> bool {
    let benedictions_specs = town_context.benedictions_specs.read();
    town_context
        .player_benedictions
        .read()
        .categories
        .iter()
        .any(|(category_id, player_category)| {
            let Some(category_specs) = benedictions_specs.get(category_id) else {
                return false;
            };

            player_category
                .purchased_benedictions
                .iter()
                .any(|(benediction_id, level)| {
                    *level > 0
                        && category_specs.benedictions.get(benediction_id).is_some_and(
                            |benediction| benediction.effect == BenedictionEffect::SkillSlots,
                        )
                })
        })
}

fn has_first_unspent_mastery_point(town_context: TownContext, data_context: DataContext) -> bool {
    let mastery_specs = data_context.skill_mastery_specs.read();
    let skill_masteries = town_context.player_skill_masteries.read();
    let no_points_spent = skill_masteries.masteries.values().all(|mastery| {
        mastery
            .upgrades_bought
            .values()
            .all(|upgrade_level| *upgrade_level == 0)
    });

    no_points_spent
        && skill_masteries.masteries.iter().any(|(skill_id, mastery)| {
            mastery_specs
                .get(skill_id)
                .is_some_and(|specs| mastery.level(specs.max_level) >= 1)
        })
}
