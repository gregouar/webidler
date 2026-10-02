use leptos::{prelude::*, task::spawn_local};

use shared::{
    data::{
        pets::{PetButton, PetSpecs, PlayerPets},
        user::{UserCharacterId, UserUnlocks},
    },
    http::client::UpdateCharacterPetsRequest,
};

use crate::{
    assets::img_asset,
    components::{
        backend_client::BackendClient,
        data_context::DataContext,
        icons::header_icons::PetsIcon,
        settings::SettingsContext,
        ui::{
            card::{CardHeader, CardInset, MenuCard},
            list_row::MenuListRow,
            menu_panel::MenuPanel,
            toast::{ToastVariant, Toasts, show_toast},
            tooltip::{HelpTooltip, StaticTooltip, StaticTooltipPosition},
        },
    },
};
#[component]
pub fn PetSprite(
    pet_id: String,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(default = false)] flipped: bool,
) -> impl IntoView {
    let data: DataContext = expect_context();
    let settings: SettingsContext = expect_context();
    data.pets_specs
        .read_untracked()
        .get(&pet_id)
        .cloned()
        .map(|pet_specs| {
            let pet_name = pet_specs.name.clone();
            view! {
                <StaticTooltip position=StaticTooltipPosition::Top tooltip=move || pet_name.clone()>
                    <img
                        src=img_asset(&pet_specs.icon)
                        alt=pet_specs.name.clone()
                        class=format!(
                            "z-20 w-12 xl:w-20 max-w-none object-contain
                        {} {} {}",
                            if settings.uses_surface_effects() {
                                "xl:[filter:drop-shadow(0_0_3px_rgba(190,160,0,0.7))_drop-shadow(0_4px_12px_rgba(0,0,0,0.7))]"
                            } else {
                                ""
                            },
                            if flipped { "-scale-x-100" } else { "" },
                            class.unwrap_or(&""),
                        )
                    />
                </StaticTooltip>
            }
        })
}

#[component]
pub fn PetsPanel(
    open: RwSignal<bool>,
    character_id: Signal<UserCharacterId>,
    player_pets: RwSignal<PlayerPets>,
    user_unlocks: RwSignal<UserUnlocks>,
) -> impl IntoView {
    let data = expect_context::<DataContext>();
    let assigning = RwSignal::new(None::<PetButton>);
    let assign_open = RwSignal::new(false);
    let buttons = vec![
        (PetButton::Skill1, "Skill 1"),
        (PetButton::Skill2, "Skill 2"),
        (PetButton::Skill3, "Skill 3"),
        (PetButton::Skill4, "Skill 4"),
        (PetButton::LevelUp, "Level Up"),
        (PetButton::AutoPassive, "Auto Assign Passive"),
    ];
    let buttons = StoredValue::new(buttons);

    view! {
        <MenuPanel open w_full=false h_full=false class:items-center>
            <MenuCard class="w-full max-w-3xl mx-auto">
                <CardHeader title="Pets" on_close=move || open.set(false)>
                    <div class="ml-2 mr-auto">
                        <HelpTooltip text="Pets automate clicking the buttons they are assigned to." />
                    </div>
                </CardHeader>
                <CardInset class="min-h-0 overflow-y-auto">
                    <div class="grid grid-cols-2 gap-2 xl:grid-cols-3">
                        <For
                            each=move || buttons.get_value()
                            key=|(button, _)| *button
                            children=move |(button, label)| {
                                let assigned = move || { player_pets.read().get(&button).cloned() };
                                view! {
                                    <MenuListRow
                                        class="group min-h-40 overflow-hidden active:scale-95 active:brightness-75"
                                        on_click=move || {
                                            assigning.set(Some(button));
                                            assign_open.set(true);
                                        }
                                    >
                                        <div class="flex min-h-40 flex-col items-center gap-2 p-3">
                                            <div class="w-full pb-2 text-center font-display text-sm font-semibold tracking-wide text-amber-200">
                                                {label}
                                            </div>
                                            <div class="flex min-h-20 flex-1 items-center justify-center">
                                                {move || match assigned() {
                                                    Some(pet_id) => {
                                                        view! {
                                                            <div class="flex flex-col items-center gap-1">
                                                                // <img
                                                                // src=img_asset(&pet.icon)
                                                                // alt=pet.name.clone()
                                                                // class="h-20 w-20 object-contain drop-shadow-[0_4px_4px_rgba(0,0,0,0.5)]"
                                                                // />
                                                                <PetSprite pet_id />
                                                            // <span class="text-xs xl:text-sm font-medium text-zinc-300">
                                                            // {pet.name.clone()}
                                                            // </span>
                                                            </div>
                                                        }
                                                            .into_any()
                                                    }
                                                    None => {
                                                        view! {
                                                            <div class="text-lg xl:text-xl font-semibold text-zinc-500 transition-colors group-hover:text-amber-300/80">
                                                                <PetsIcon />
                                                            </div>
                                                        }
                                                            .into_any()
                                                    }
                                                }}
                                            </div>
                                        </div>
                                    </MenuListRow>
                                }
                            }
                        />
                    </div>
                </CardInset>
            </MenuCard>
        </MenuPanel>

        <MenuPanel open=assign_open w_full=false h_full=false class:items-center>
            <MenuCard class="w-full max-w-2xl mx-auto">
                <CardHeader
                    title="Assign Pet"
                    on_close=move || {
                        assigning.set(None);
                        assign_open.set(false);
                    }
                />
                <CardInset class="min-h-0 overflow-y-auto">
                    <div class="grid grid-cols-2 gap-2 xl:grid-cols-3 xl:gap-3">
                        {move || {
                            assigning
                                .get()
                                .and_then(|assigning_button| {
                                    player_pets
                                        .read()
                                        .contains_key(&assigning_button)
                                        .then(|| {
                                            view! {
                                                <MenuListRow
                                                    class="group min-h-32 overflow-hidden active:scale-95 active:brightness-75 transition-all duration-300"
                                                    on_click=move || {
                                                        update_assignment(
                                                            character_id.get_untracked(),
                                                            player_pets,
                                                            None,
                                                            assigning_button,
                                                        );
                                                        assigning.set(None);
                                                        assign_open.set(false);
                                                    }
                                                >
                                                    <div class="flex min-h-32 flex-col items-center justify-center gap-2 p-3">
                                                        // <span class="font-display text-sm font-semibold text-zinc-300">
                                                        // "Remove"
                                                        // </span>
                                                        "❌"
                                                    </div>
                                                </MenuListRow>
                                            }
                                        })
                                })
                        }}
                        <For
                            each=move || unlocked_pets(user_unlocks, data.pets_specs)
                            key=|(id, _)| id.clone()
                            children=move |(id, pet)| {
                                let assigned_id = id.clone();
                                let assigned_to = Memo::new(move |_| {
                                    player_pets
                                        .read()
                                        .iter()
                                        .find_map(|(button, pet_id)| {
                                            (pet_id == &assigned_id).then_some(*button)
                                        })
                                });
                                let clicked_id = id.clone();
                                let state_class = Signal::derive(move || {
                                    if assigned_to.get().is_some() {
                                        "grayscale brightness-50 hover:grayscale-[0.65] hover:brightness-75"
                                            .to_string()
                                    } else {
                                        "hover:brightness-110".to_string()
                                    }
                                });

                                view! {
                                    <MenuListRow
                                        class="group min-h-32 overflow-hidden active:scale-95 active:brightness-75 transition-all duration-300"
                                        state_class
                                        on_click=move || {
                                            if let Some(button) = assigning.get_untracked() {
                                                update_assignment(
                                                    character_id.get_untracked(),
                                                    player_pets,
                                                    Some(clicked_id.clone()),
                                                    button,
                                                );
                                                assigning.set(None);
                                                assign_open.set(false);
                                            }
                                        }
                                    >
                                        <div class="flex min-h-32 flex-col items-center justify-center gap-2 p-3">
                                            <img
                                                src=img_asset(&pet.icon)
                                                alt=pet.name.clone()
                                                class="h-20 w-20 object-contain drop-shadow-[0_4px_4px_rgba(0,0,0,0.5)] transition-transform duration-200 group-hover:scale-105"
                                            />
                                            <span class="font-display text-sm font-semibold text-amber-200">
                                                {pet.name}
                                            </span>
                                        // <span class="min-h-4 text-[10px] font-semibold uppercase tracking-wide text-zinc-400">
                                        // {move || {
                                        // assigned_to
                                        // .get()
                                        // .map(|button| {
                                        // format!("Assigned: {}", pet_button_label(button))
                                        // })
                                        // }}
                                        // </span>
                                        </div>
                                    </MenuListRow>
                                }
                            }
                        />
                    </div>
                </CardInset>
            </MenuCard>
        </MenuPanel>
    }
}

fn unlocked_pets(
    user_unlocks: RwSignal<UserUnlocks>,
    pets: RwSignal<std::collections::HashMap<String, PetSpecs>>,
) -> Vec<(String, PetSpecs)> {
    let mut pets = pets
        .read()
        .iter()
        .filter(|(id, _)| user_unlocks.read().pets.contains(*id))
        .map(|(id, pet)| (id.clone(), pet.clone()))
        .collect::<Vec<_>>();
    pets.sort_unstable_by(|(_, left), (_, right)| left.name.cmp(&right.name));
    pets
}

// fn pet_button_label(button: PetButton) -> String {
//     match button {
//         PetButton::Skill { index } => format!("Skill {}", index + 1),
//         PetButton::LevelUp => "Level Up".into(),
//         PetButton::AutoPassive => "Auto Assign Passive".into(),
//     }
// }

fn update_assignment(
    character_id: UserCharacterId,
    player_pets: RwSignal<PlayerPets>,
    pet: Option<String>,
    button: PetButton,
) {
    let backend = expect_context::<BackendClient>();
    let toaster = expect_context::<Toasts>();
    let previous = player_pets.get_untracked();

    let mut updated = previous.clone();
    if let Some(pet) = pet {
        updated.retain(|_, assigned_pet| assigned_pet != &pet);
        updated.insert(button, pet);
    } else {
        updated.remove(&button);
    }

    player_pets.set(updated.clone());
    spawn_local(async move {
        if let Err(error) = backend
            .post_update_character_pets(
                &character_id,
                &UpdateCharacterPetsRequest { pets: updated },
            )
            .await
        {
            player_pets.set(previous);
            show_toast(
                toaster,
                format!("Failed to assign pet: {error}"),
                ToastVariant::Error,
            );
        }
    });
}
