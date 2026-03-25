use std::sync::Arc;

use indexmap::IndexMap;

use leptos::{html::*, prelude::*};

use frontend::components::{
    events::{EventsContext, Key},
    shared::{item_card::ItemCard, tooltips::item_tooltip::name_color_rarity},
    ui::{
        buttons::MenuButton,
        card::{Card, CardInset, CardTitle},
    },
    utils::file_loader::{save_json, use_json_loader},
};

use leptos_use::{WatchDebouncedOptions, watch_debounced_with_options};
use shared::data::{
    item::{ItemBase, ItemModifiers, ItemSpecs},
    item_affix::{AffixEffect, AffixType, ItemAffix},
    stat_effect::StatEffect,
};

use crate::{header::HeaderMenu, utils::json_editor::JsonEditor};

type ItemsStore = IndexMap<String, ItemBase>;

#[component]
pub fn ItemsPage() -> impl IntoView {
    let events_context: EventsContext = expect_context();

    let items_store = RwSignal::new(Default::default());
    let selected_item = RwSignal::new(None);

    let (loaded_file, filename, on_file) = use_json_loader::<ItemsStore>();

    Effect::new(move || {
        loaded_file.with(|loaded_file| {
            if let Some(loaded_file) = loaded_file {
                selected_item.set(None);
                items_store.set(loaded_file.clone());
            }
        });
    });

    let save = move || {
        save_json(
            &items_store.get(),
            &filename.get().unwrap_or("items.json".into()),
        );
    };

    let file_input: NodeRef<Input> = NodeRef::new();

    let load = move || {
        if let Some(input) = file_input.get() {
            input.click();
        }
    };

    Effect::new({
        move || {
            if events_context.key_pressed(Key::Ctrl) {
                if events_context.key_pressed(Key::Character('s')) {
                    save();
                } else if events_context.key_pressed(Key::Character('o')) {
                    load();
                }
            }
        }
    });

    view! {
        <main class="my-0 mx-auto w-full text-center overflow-x-hidden flex flex-col min-h-screen">
            <input
                node_ref=file_input
                type="file"
                accept="application/json"
                on:change=on_file
                class="hidden"
            />
            <HeaderMenu />
            <div class="relative flex-1">
                <div class="absolute inset-0 flex p-1 xl:p-4 items-center gap-4">
                    <div class="w-6xl h-full">
                        <Card>
                            <div class="flex justify-between mx-4 items-center">
                                <div class="flex flex-row items-center gap-1 xl:gap-2">
                                    <CardTitle>"Items"</CardTitle>
                                </div>

                                <div class="flex gap-2 ml-4">
                                    <MenuButton on:click=move |_| { load() }>"Load"</MenuButton>
                                    <MenuButton on:click=move |_| { save() }>"Save"</MenuButton>
                                </div>

                                <div class="flex-1" />

                                <div class="flex gap-2 ml-4">
                                    <MenuButton on:click=move |_| {}>"Duplicate"</MenuButton>
                                    <MenuButton on:click=move |_| {}>"Add"</MenuButton>
                                </div>

                            </div>
                            <CardInset class:flex-1 class:z-1>
                                <ItemsList items_store selected_item />
                            </CardInset>
                        </Card>
                    </div>

                    <Card class="h-full w-full">
                        <ItemEditor items_store selected_item />
                    </Card>

                    <Card class="h-full w-lg">
                        <ItemPreview items_store selected_item />
                    </Card>

                </div>
            </div>
        </main>
    }
}

#[component]
fn ItemsList(
    items_store: RwSignal<ItemsStore>,
    selected_item: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2">
            <For
                each=move || items_store.get().into_iter()
                key=|(item_id, _)| item_id.clone()
                let((item_id, item_specs))
            >
                <div
                    class={
                        let item_id = item_id.clone();
                        move || {
                            format!(
                                "flex justify-between hover:bg-zinc-700 {}",
                                match selected_item.get() {
                                    Some(selected_item_id) if selected_item_id == item_id => {
                                        "ring-1 ring-amber-500"
                                    }
                                    _ => "",
                                },
                            )
                        }
                    }
                    on:click=move |_| selected_item.set(Some(item_id.clone()))
                >

                    <span>{item_id.clone()}</span>
                    <span class=format!(
                        "font-semibold {}",
                        name_color_rarity(item_specs.rarity),
                    )>{item_specs.name}</span>
                </div>
            </For>
        </div>
    }
}

#[component]
fn ItemEditor(
    items_store: RwSignal<ItemsStore>,
    selected_item: RwSignal<Option<String>>,
) -> impl IntoView {
    let item_base = RwSignal::new(Default::default());

    Effect::new(move || {
        if let Some(selected_item) = selected_item.get() {
            if let Some(selected_item_specs) = items_store.read().get(&selected_item) {
                item_base.set(selected_item_specs.clone());
            }
        }
    });

    let _ = watch_debounced_with_options(
        move || item_base.get(),
        move |value, _, _| {
            if let Some(item_id) = selected_item.get_untracked()
                && items_store
                    .read_untracked()
                    .get(&item_id)
                    .map(|item_base| *item_base != *value)
                    .unwrap_or_default()
            {
                items_store.write().insert(item_id, value.clone());
            }
        },
        250.0,
        WatchDebouncedOptions::default().immediate(false),
    );

    view! {
        <div class="flex flex-col gap-2">
            <div class="flex justify-between">
                <span>{move || selected_item.get()}</span>

                <MenuButton class:ml-2 on:click=move |_| {}>
                    "❌"
                </MenuButton>
            </div>
            <JsonEditor label="Item" value=item_base h_size="h-196" />
        </div>
    }
}

#[component]
fn ItemPreview(
    items_store: RwSignal<ItemsStore>,
    selected_item: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        {move || {
            selected_item
                .get()
                .and_then(|selected_item| items_store.read().get(&selected_item).cloned())
                .map(|item_base| {
                    let unique_effects = item_base
                        .affixes
                        .iter()
                        .map(|affix| ItemAffix {
                            name: "Unique".to_string(),
                            family: "unique".to_string(),
                            tags: Default::default(),
                            affix_type: AffixType::Unique,
                            tier: 0,
                            effects: Vec::from([
                                AffixEffect {
                                    scope: affix.scope,
                                    stat_effect: StatEffect {
                                        stat: affix.stat.clone(),
                                        modifier: affix.modifier,
                                        value: affix.value.max,
                                        bypass_ignore: false,
                                    },
                                },
                            ]),
                            item_level: 0,
                        })
                        .collect();
                    let item_specs = Arc::new(ItemSpecs {
                        required_level: item_base.min_area_level,
                        weapon_specs: item_base.weapon_specs.clone(),
                        armor_specs: item_base.armor_specs.clone(),
                        modifiers: ItemModifiers {
                            base_item_id: selected_item.get().unwrap_or_default(),
                            name: item_base.name.clone(),
                            rarity: item_base.rarity,
                            level: item_base.min_area_level,
                            affixes: unique_effects,
                            quality: 0.0,
                        },
                        base: item_base,
                        old_game: false,
                    });

                    view! { <ItemCard item_specs /> }
                })
        }}
    }
}
