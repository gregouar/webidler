use std::collections::{BTreeMap, BTreeSet};

use indexmap::IndexMap;
use leptos::prelude::*;

use shared::data::{
    area::AreaLevel,
    item::ItemCategory,
    item_affix::{AffixType, ItemAffixBlueprint, affix_group_id},
    market::AFFIX_FILTERS_AMOUNT,
    stat_effect::StatEffect,
};

use crate::components::{
    data_context::DataContext,
    shared::{
        stat_filters::format_stat_filter,
        tooltips::{effects_tooltip, item_tooltip::affix_type_str},
    },
    ui::{
        buttons::MenuButton,
        dropdown::{DropdownMenu, SearchableDropdownMenu},
    },
};

pub type AffixFilterSignals =
    [(RwSignal<Option<String>>, RwSignal<Option<AreaLevel>>); AFFIX_FILTERS_AMOUNT];

#[component]
pub fn AffixFilters(
    affix_filters: AffixFilterSignals,
    #[prop(into)] item_category: Signal<Option<ItemCategory>>,
) -> impl IntoView {
    let visible_affix_filters = Memo::new(move |_| {
        visible_filter_slots(&affix_filters.map(|(affix_id, _)| affix_id.get().is_some()))
    });
    let data_context: DataContext = expect_context();
    let available_affix_groups =
        Memo::new(move |_| affix_groups(data_context, item_category.get()));

    Effect::new(move |_| {
        let groups = available_affix_groups.read();
        for (affix_id, min_level) in affix_filters {
            let Some(selected_affix) = affix_id.get() else {
                min_level.set(None);
                continue;
            };
            let Some(group) = groups.get(&selected_affix) else {
                affix_id.set(None);
                min_level.set(None);
                continue;
            };
            if min_level
                .get()
                .is_some_and(|level| !group.levels.contains(&level))
            {
                min_level.set(None);
            }
        }
    });

    view! {
        <div class="flex flex-col gap-2 xl:gap-4 p-2 xl:p-4">
            {affix_filters
                .into_iter()
                .enumerate()
                .map(|(index, (affix_id, min_level))| {
                    view! {
                        <Show when=move || index < visible_affix_filters.get()>
                            <div class="flex w-full min-w-0 gap-2 xl:gap-4 items-center">
                                {move || {
                                    affix_id
                                        .read()
                                        .is_some()
                                        .then(|| {
                                            view! {
                                                <MenuButton
                                                    class:flex-none
                                                    on:click=move |_| {
                                                        affix_id.set(None);
                                                        min_level.set(None);
                                                    }
                                                >
                                                    "❌"
                                                </MenuButton>
                                            }
                                        })
                                }}
                                <span class=move || {
                                    if affix_id.read().is_none() {
                                        "flex-1 min-w-0 text-center"
                                    } else {
                                        "flex-1 min-w-0"
                                    }
                                }>
                                    {move || {
                                        let options = available_affix_groups
                                            .read()
                                            .iter()
                                            .map(|(group_id, group)| {
                                                (Some(group_id.clone()), group.label.clone())
                                            })
                                            .collect();
                                        view! {
                                            <SearchableDropdownMenu
                                                options
                                                chosen_option=affix_id
                                                missing_text="+ Add Affix Filter"
                                                width_class="w-full min-w-0"
                                            />
                                        }
                                    }}
                                </span>
                                {move || {
                                    affix_id
                                        .get()
                                        .and_then(|selected_affix| {
                                            available_affix_groups.read().get(&selected_affix).cloned()
                                        })
                                        .map(|group| {
                                            let options = std::iter::once((None, "Level".to_string()))
                                                .chain(
                                                    group
                                                        .levels
                                                        .into_iter()
                                                        .map(|level| (Some(level), level.to_string())),
                                                )
                                                .collect();
                                            view! {
                                                <DropdownMenu
                                                    options
                                                    chosen_option=min_level
                                                    missing_text="Level"
                                                    width_class="w-24"
                                                />
                                            }
                                        })
                                }}
                            </div>
                        </Show>
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}

#[derive(Clone, PartialEq)]
struct AffixGroupOption {
    label: String,
    levels: Vec<AreaLevel>,
}

fn affix_effects_label(
    affix: &ItemAffixBlueprint,
    affix_base_category: AffixBaseCategory,
    show_fixed_values: bool,
) -> String {
    let effects = affix
        .effects
        .iter()
        .map(|effect| {
            if show_fixed_values && effect.value.min == effect.value.max {
                effects_tooltip::format_stat(&StatEffect {
                    stat: effect.stat.clone(),
                    modifier: effect.modifier,
                    value: effect.value.min,
                    bypass_ignore: effect.bypass_ignore,
                })
            } else {
                format_stat_filter(&effect.stat, effect.modifier, effect.value.max < 0.0)
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    let meta = if let Some(category) = affix_base_category_str(affix_base_category) {
        format!("({category} · {})", affix_type_str(affix.affix_type))
    } else {
        format!("({})", affix_type_str(affix.affix_type))
    };

    format!(" {meta} {effects}")
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AffixBaseCategory {
    Item,
    Rune,
    Map,
    Other,
}

fn affix_base_category(affix_id: &str) -> AffixBaseCategory {
    let prefix = affix_id
        .split_once('_')
        .map(|(prefix, _)| prefix)
        .unwrap_or(affix_id);
    match prefix {
        "item" => AffixBaseCategory::Item,
        "rune" => AffixBaseCategory::Rune,
        "map" => AffixBaseCategory::Map,
        _ => AffixBaseCategory::Other,
    }
}

fn affix_base_category_str(affix_base_category: AffixBaseCategory) -> Option<&'static str> {
    match affix_base_category {
        AffixBaseCategory::Item => Some("Item"),
        AffixBaseCategory::Rune => Some("Rune"),
        AffixBaseCategory::Map => Some("Edict"),
        AffixBaseCategory::Other => None,
    }
}

pub fn visible_filter_slots(occupied: &[bool]) -> usize {
    let Some(last_occupied) = occupied.iter().rposition(|occupied| *occupied) else {
        return usize::from(!occupied.is_empty());
    };

    let occupied_slots = last_occupied + 1;
    if occupied_slots < occupied.len()
        && occupied[..occupied_slots].iter().all(|occupied| *occupied)
    {
        occupied_slots + 1
    } else {
        occupied_slots
    }
}

fn affix_groups(
    data_context: DataContext,
    category: Option<ItemCategory>,
) -> IndexMap<String, AffixGroupOption> {
    let item_affixes = data_context.item_affixes.read_untracked();
    let mut groups =
        BTreeMap::<(AffixBaseCategory, AffixType, String), Vec<&ItemAffixBlueprint>>::new();

    let intersected_categories = category
        .map(|category| {
            data_context
                .item_bases
                .read_untracked()
                .values()
                .filter(|item_base| item_base.categories.contains(&category))
                .fold(BTreeSet::new(), |mut acc, item_base| {
                    if acc.is_empty() {
                        acc.extend(item_base.categories.iter());
                    } else {
                        acc.retain(|category| item_base.categories.contains(category));
                    }
                    acc
                })
        })
        .unwrap_or_default();

    for (affix_id, affix) in item_affixes.iter().filter(|(_, affix)| {
        matches!(
            affix.affix_type,
            AffixType::Prefix | AffixType::Suffix | AffixType::Soul
        ) && (category.is_none()
            || (category == Some(ItemCategory::Remnant) && affix.affix_type == AffixType::Soul)
            || !intersected_categories.is_disjoint(&affix.restrictions))
    }) {
        groups
            .entry((
                affix_base_category(affix_id),
                affix.affix_type,
                affix_group_id(affix_id).to_string(),
            ))
            .or_default()
            .push(affix);
    }

    groups
        .into_iter()
        .map(|((affix_base_category, _, group_id), mut group)| {
            group.sort_by_key(|affix| affix.item_level);
            let label = affix_effects_label(
                group
                    .first()
                    .copied()
                    .expect("aggregated affix groups cannot be empty"),
                affix_base_category,
                group.len() == 1,
            );
            let mut levels = group
                .iter()
                .map(|affix| affix.item_level)
                .collect::<Vec<_>>();
            levels.sort_unstable();
            levels.dedup();

            (group_id, AffixGroupOption { label, levels })
        })
        .collect()
}
