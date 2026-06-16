use std::{collections::HashMap, sync::Arc};

use leptos::{html::*, prelude::*};
use leptos_use::{WatchDebouncedOptions, watch_debounced_with_options};
use shared::data::{
    player::PlayerBaseSkill,
    skill::{BaseSkillSpecs, SkillType},
    values::NonNegative,
};

use frontend::components::{
    backend_client::BackendClient,
    data_context::DataContext,
    events::{EventsContext, Key},
    shared::{skills::skill_specs_from_base, tooltips::skill_tooltip::SkillTooltip},
    ui::{
        buttons::MenuButton,
        card::{Card, CardInset, CardTitle},
    },
    utils::file_loader::{save_json, use_json_loader},
};

use crate::{header::HeaderMenu, utils::json_editor::JsonEditor};

type SkillsStore = HashMap<String, BaseSkillSpecs>;

#[component]
pub fn SkillsPage() -> impl IntoView {
    let events_context: EventsContext = expect_context();

    let backend: BackendClient = expect_context();
    let data_context: DataContext = expect_context();

    let _data_load = LocalResource::new({
        move || async move {
            let _ = data_context.load_data(backend).await;
        }
    });

    let skills_store = RwSignal::new(Default::default());
    let selected_skill = RwSignal::new(None::<String>);

    let (loaded_file, filename, on_file) = use_json_loader::<SkillsStore>();

    Effect::new(move || {
        loaded_file.with(|loaded_file| {
            if let Some(loaded_file) = loaded_file {
                selected_skill.set(None);
                skills_store.set(loaded_file.clone());
            }
        });
    });

    let save = move || {
        save_json(
            &skills_store.get(),
            &filename.get().unwrap_or("skills.json".into()),
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

    let add_skill = move || {
        let mut id = "new_skill".to_string();
        let mut counter = 2;
        while skills_store.read_untracked().contains_key(&id) {
            id = format!("new_skill_{counter}");
            counter += 1;
        }

        skills_store.write().insert(id.clone(), new_skill_specs());
        selected_skill.set(Some(id));
    };

    let duplicate_skill = move || {
        if let Some(selected_skill_id) = selected_skill.get_untracked()
            && let Some(skill_specs) = skills_store.read_untracked().get(&selected_skill_id)
        {
            let mut id = format!("{selected_skill_id}_copy");
            let mut counter = 2;
            while skills_store.read_untracked().contains_key(&id) {
                id = format!("{selected_skill_id}_copy_{counter}");
                counter += 1;
            }

            skills_store.write().insert(id.clone(), skill_specs.clone());
            selected_skill.set(Some(id));
        }
    };

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
                                    <CardTitle>"Skills"</CardTitle>
                                    <span class="text-shadow-md shadow-gray-950 text-gray-400 text-xs xl:text-base font-medium">
                                        "(" {move || skills_store.read().len()} ")"
                                    </span>
                                </div>

                                <div class="flex gap-2 ml-4">
                                    <MenuButton on:click=move |_| { load() }>"Load"</MenuButton>
                                    <MenuButton on:click=move |_| { save() }>"Save"</MenuButton>
                                </div>

                                <div class="flex-1" />

                                <div class="flex gap-2 ml-4">
                                    <MenuButton
                                        on:click=move |_| duplicate_skill()
                                        disabled=Signal::derive(move || {
                                            selected_skill.get().is_none()
                                        })
                                    >
                                        "Duplicate"
                                    </MenuButton>
                                    <MenuButton on:click=move |_| add_skill()>"Add"</MenuButton>
                                </div>
                            </div>
                            <CardInset class:flex-1 class:z-1>
                                <SkillsList skills_store selected_skill />
                            </CardInset>
                        </Card>
                    </div>

                    <Card class="h-full w-full">
                        <SkillEditor skills_store selected_skill />
                    </Card>

                    <Card class="h-full w-2xl">
                        <SkillPreview skills_store selected_skill />
                    </Card>
                </div>
            </div>
        </main>
    }
}

#[component]
fn SkillsList(
    skills_store: RwSignal<SkillsStore>,
    selected_skill: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2">
            <For
                each=move || {
                    let mut skills = skills_store.get().into_iter().collect::<Vec<_>>();
                    skills
                        .sort_by_key(|(skill_id, skill_specs)| {
                            (skill_specs.skill_type, skill_specs.name.clone(), skill_id.clone())
                        });
                    skills
                }
                key=|(skill_id, _)| skill_id.clone()
                let((skill_id, skill_specs))
            >
                <div
                    class={
                        let skill_id = skill_id.clone();
                        move || {
                            format!(
                                "flex justify-between gap-4 px-1 hover:bg-zinc-700 {}",
                                match selected_skill.get() {
                                    Some(selected_skill_id) if selected_skill_id == skill_id => {
                                        "ring-1 ring-amber-500"
                                    }
                                    _ => "",
                                },
                            )
                        }
                    }
                    on:click=move |_| selected_skill.set(Some(skill_id.clone()))
                >
                    <span class="text-left truncate">{skill_id.clone()}</span>
                    <span class=format!(
                        "font-semibold text-right {}",
                        skill_type_color(skill_specs.skill_type),
                    )>{skill_specs.name}</span>
                </div>
            </For>
        </div>
    }
}

#[component]
fn SkillEditor(
    skills_store: RwSignal<SkillsStore>,
    selected_skill: RwSignal<Option<String>>,
) -> impl IntoView {
    let skill_specs = RwSignal::new(new_skill_specs());

    Effect::new(move || {
        if let Some(selected_skill) = selected_skill.get()
            && let Some(selected_skill_specs) = skills_store.read().get(&selected_skill)
        {
            skill_specs.set(selected_skill_specs.clone());
        }
    });

    let _ = watch_debounced_with_options(
        move || skill_specs.get(),
        move |value, _, _| {
            if let Some(skill_id) = selected_skill.get_untracked()
                && skills_store
                    .read_untracked()
                    .get(&skill_id)
                    .map(|skill_specs| *skill_specs != *value)
                    .unwrap_or_default()
            {
                skills_store.write().insert(skill_id, value.clone());
            }
        },
        250.0,
        WatchDebouncedOptions::default().immediate(false),
    );

    let delete_skill = move || {
        if let Some(skill_id) = selected_skill.get_untracked() {
            skills_store.write().remove(&skill_id);
            selected_skill.set(None);
        }
    };

    view! {
        <div class="flex flex-col gap-2">
            <div class="flex justify-between">
                <span>{move || selected_skill.get()}</span>

                <MenuButton
                    class:ml-2
                    on:click=move |_| delete_skill()
                    disabled=Signal::derive(move || selected_skill.get().is_none())
                >
                    "x"
                </MenuButton>
            </div>
            <JsonEditor label="Skill" value=skill_specs h_size="h-160" />
        </div>
    }
}

#[component]
fn SkillPreview(
    skills_store: RwSignal<SkillsStore>,
    selected_skill: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <div class="flex h-full items-start justify-center overflow-auto p-2">
            {move || {
                selected_skill
                    .get()
                    .and_then(|skill_id| {
                        skills_store
                            .read()
                            .get(&skill_id)
                            .cloned()
                            .map(|base_skill_specs| {
                                let skill_specs = Arc::new(
                                    skill_specs_from_base(skill_id, &base_skill_specs),
                                );
                                let player_base_skill = Some(
                                    Arc::new(PlayerBaseSkill {
                                        next_upgrade_cost: base_skill_specs.upgrade_cost,
                                        base_skill_specs,
                                        item_slot: None,
                                        upgrade_level: 0,
                                    }),
                                );

                                view! {
                                    <SkillTooltip
                                        skill_specs=skill_specs
                                        player_base_skill=player_base_skill
                                    />
                                }
                            })
                    })
            }}
        </div>
    }
}

fn new_skill_specs() -> BaseSkillSpecs {
    BaseSkillSpecs {
        name: "New Skill".into(),
        icon: "skills/attack.svg".into(),
        description: String::new(),
        skill_type: SkillType::Other,
        cooldown: NonNegative::new(1.0),
        mana_cost: NonNegative::default(),
        upgrade_cost: 0.0,
        upgrade_effects: Default::default(),
        modifier_effects: Default::default(),
        targets: Default::default(),
        triggers: Default::default(),
        auto_use_conditions: Default::default(),
        ignore_stat_effects: Default::default(),
        required_item: Default::default(),
    }
}

fn skill_type_color(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "text-red-300",
        SkillType::Spell => "text-sky-300",
        SkillType::Curse => "text-purple-300",
        SkillType::Blessing => "text-amber-200",
        SkillType::Other => "text-slate-300",
    }
}
