use std::{collections::HashMap, iter, time::Duration};

use shared::{
    constants::{DEFAULT_MAX_LEVEL, PLAYER_LIFE_PER_LEVEL, SKILL_BASE_COST},
    data::{
        area::{AreaLevel, AreaThreat},
        chance::{Chance, ChanceRange},
        character::{CharacterAttrs, CharacterId, CharacterSize, CharacterStatic},
        character_status::StatusSpecs,
        item::{SkillRange, SkillShape},
        item_affix::AffixEffectScope,
        modifier::{ModifiableValue, Modifier},
        passive::{self, PassivesTreeSpecs, PassivesTreeState},
        player::{PlayerBaseSpecs, PlayerInventory, PlayerSpecs, PlayerState},
        skill::{
            DamageType, RestoreModifier, RestoreType, SkillEffect, SkillEffectType, SkillType,
        },
        stat_effect::{EffectsMap, StatConverterSource, StatConverterSpecs, StatEffect, StatType},
        trigger::{EventTrigger, HitTrigger, TriggerTarget, TriggeredEffect},
        values::{AtLeastOne, NonNegative},
    },
};
use strum::IntoEnumIterator;

use crate::game::{
    data::{DataInit, event::EventsQueue},
    systems::{stats_updater, statuses_controller},
};

use super::{characters_updater, skills_updater};

pub fn base_player_character_attrs(level: u8) -> CharacterAttrs {
    CharacterAttrs {
        max_life: AtLeastOne::new(100.0 + PLAYER_LIFE_PER_LEVEL * (level.saturating_sub(1)) as f64)
            .into(),
        life_regen: 10.0.into(),
        max_mana: NonNegative::new(100.0).into(),
        mana_regen: 10.0.into(),
        ..Default::default()
    }
}

pub fn init_player_base_specs(
    character_name: String,
    character_portrait: String,
    max_area_level: AreaLevel,
    effects: EffectsMap,
) -> PlayerBaseSpecs {
    PlayerBaseSpecs {
        max_area_level,
        character_static: CharacterStatic {
            name: character_name,
            portrait: character_portrait,
            size: CharacterSize::Small,
            position_x: 0,
            position_y: 0,
        },
        character_attrs: base_player_character_attrs(1),
        effects,
        max_skills: 4,
        buy_skill_cost: SKILL_BASE_COST,
        skills: Default::default(),
        level: 1,
        experience_needed: 20.0,
        movement_cooldown: 3.0.into(),
        gold_find: 100.0.into(),
        threat_gain: 100.0.into(),
        max_level: DEFAULT_MAX_LEVEL,
    }
}

pub fn update_player_state(
    events_queue: &mut EventsQueue,
    elapsed_time: Duration,
    player_specs: &PlayerSpecs,
    player_state: &mut PlayerState,
    area_threat: &AreaThreat,
) {
    if !player_state.character_state.is_alive {
        return;
    }

    characters_updater::update_character_state(
        events_queue,
        elapsed_time,
        CharacterId::Player,
        &player_specs.character_specs,
        &mut player_state.character_state,
        area_threat,
    );
}

pub fn reset_player(player_state: &mut PlayerState) {
    characters_updater::reset_character(&mut player_state.character_state);
}

// I hate the fact player state influences player specs... But I couldn't figure out a way
// to have it working with the dynamic statuses.
pub fn update_player_specs(
    player_base_specs: &PlayerBaseSpecs,
    player_specs: &PlayerSpecs,
    player_state: &PlayerState,
    player_inventory: &PlayerInventory,
    passives_tree_specs: &PassivesTreeSpecs,
    passives_tree_state: &PassivesTreeState,
    area_threat: &AreaThreat,
) -> PlayerSpecs {
    let effects_map = EffectsMap::combine_all(
        player_inventory
            .equipped_items()
            .map(|(_, i)| {
                i.modifiers
                    .aggregate_effects(AffixEffectScope::Global, false)
            })
            .chain(iter::once(passive::generate_effects_map_from_passives(
                passives_tree_specs,
                &passives_tree_state.ascension,
                &passives_tree_state.purchased_nodes,
            )))
            .chain(iter::once(
                statuses_controller::generate_effects_map_from_statuses(
                    &player_state.character_state.statuses,
                ),
            ))
            .chain(iter::once(
                stats_updater::compute_conditional_modifiers(
                    area_threat,
                    &player_specs.character_specs.character_attrs,
                    &player_state.character_state,
                    &player_specs.character_specs.conditional_modifiers,
                )
                .into(),
            ))
            .chain(iter::once(player_base_specs.effects.clone())),
    );

    let mut effects: Vec<_> = (&effects_map).into();

    let mut player_specs = compute_player_specs(player_base_specs, player_inventory, &mut effects);

    player_specs.character_specs.triggers.extend(
        passives_tree_state
            .purchased_nodes
            .iter()
            .filter_map(|node_id| passives_tree_specs.nodes.get(node_id))
            .flat_map(|node| node.triggers.iter())
            .chain(
                player_inventory
                    .equipped_items()
                    .flat_map(|(_, item_specs)| item_specs.base.triggers.iter()),
            )
            .chain(
                player_specs
                    .character_specs
                    .skills_specs
                    .iter()
                    .flat_map(|skill_specs| skill_specs.triggers.iter()),
            )
            .map(|trigger_specs| trigger_specs.triggered_effect.clone()),
    );

    for trigger_specs in player_specs.character_specs.triggers.iter_mut() {
        for trigger_effect in trigger_specs.effects.iter_mut() {
            skills_updater::compute_skill_specs_effect(
                &trigger_specs.trigger_id,
                trigger_specs.skill_type,
                trigger_effect,
                effects.iter(),
            );
        }
    }

    player_specs.character_specs.triggers.extend(
        player_state
            .character_state
            .statuses
            .iter()
            .filter_map(|(status_specs, _)| match status_specs {
                StatusSpecs::Trigger(trigger_specs) => Some(trigger_specs.as_ref()),
                _ => None,
            })
            .map(|trigger_specs| trigger_specs.triggered_effect.clone()),
    );

    player_specs.character_specs.effects = effects.into();

    player_specs
}

fn compute_player_specs(
    player_base_specs: &PlayerBaseSpecs,
    player_inventory: &PlayerInventory,
    effects: &mut Vec<StatEffect>,
) -> PlayerSpecs {
    let mut player_specs = PlayerSpecs::init(player_base_specs);

    let (total_armor, total_block) = player_inventory
        .equipped_items()
        .map(|(_, item)| item)
        .filter_map(|item| item.armor_specs.as_ref())
        .map(|spec| (spec.armor, spec.block))
        .fold((0.0, 0.0), |(a_sum, b_sum), (a, b)| {
            (a_sum + *a, b_sum + b.get())
        });

    player_specs
        .character_specs
        .character_attrs
        .armor
        .entry(DamageType::Physical)
        .or_default()
        .apply_modifier(total_armor, Modifier::Flat);
    player_specs
        .character_specs
        .character_attrs
        .block
        .entry(SkillType::Attack)
        .or_default()
        .value
        .apply_modifier(total_block as f64, Modifier::Flat);

    let (character_specs, converted_effects) =
        characters_updater::update_character_specs(&player_specs.character_specs, effects);
    player_specs.character_specs = character_specs;
    effects.extend(converted_effects);

    let ModifiablePlayerSpecs {
        movement_cooldown,
        gold_find,
        threat_gain,
        restore_on_hit,
    } = modify_player_specs(
        ModifiablePlayerSpecs {
            movement_cooldown: player_specs.movement_cooldown.into(),
            gold_find: player_specs.gold_find.into(),
            threat_gain: player_specs.threat_gain.into(),
            restore_on_hit: Default::default(),
        },
        effects,
    );

    player_specs.movement_cooldown = *movement_cooldown;
    player_specs.gold_find = *gold_find;
    player_specs.threat_gain = *threat_gain;

    for ((restore_type, skill_type), value) in restore_on_hit.into_iter() {
        player_specs.character_specs.triggers.push(TriggeredEffect {
            trigger_id: format!("restore_on_hit_{:?}_{:?}", restore_type, skill_type),
            trigger: EventTrigger::OnHit(HitTrigger {
                skill_type: Some(skill_type),
                range: None,
                is_crit: None,
                is_blocked: None,
                is_hurt: Some(true),
                is_triggered: Some(false),
                damage_type: None,
            }),
            target: TriggerTarget::Source,
            skill_range: SkillRange::Any,
            skill_type,
            skill_shape: SkillShape::Single,
            modifiers: Vec::new(),
            effects: vec![SkillEffect {
                success_chance: Chance::new_sure(),
                effect_type: SkillEffectType::Restore {
                    restore_type,
                    value: ChanceRange {
                        min: value,
                        max: value,
                        lucky_chance: Default::default(),
                    },
                    modifier: RestoreModifier::Flat,
                },
                ignore_stat_effects: Default::default(),
                conditional_modifiers: Default::default(),
            }],
            owner: Some(CharacterId::Player),
        });
    }

    player_specs.character_specs.skills_specs = player_base_specs
        .skills
        .iter()
        .map(|(skill_id, player_base_skill)| {
            skills_updater::update_skill_specs(
                skill_id.to_string(),
                &player_base_skill.base_skill_specs,
                player_base_skill.upgrade_level,
                effects,
                &player_specs.character_specs.character_attrs,
                Some(player_inventory),
            )
        })
        .collect();

    player_specs
}

#[derive(Clone, Default)]
pub struct ModifiablePlayerSpecs {
    pub movement_cooldown: ModifiableValue<AtLeastOne>,
    pub gold_find: ModifiableValue<NonNegative>,
    pub threat_gain: ModifiableValue<NonNegative>,

    pub restore_on_hit: HashMap<(RestoreType, SkillType), ModifiableValue<f64>>,
}

fn modify_player_specs(
    mut modifiable_player_specs: ModifiablePlayerSpecs,
    effects: &[StatEffect],
) -> ModifiablePlayerSpecs {
    for effect in effects.iter() {
        match effect.stat {
            StatType::MovementSpeed => modifiable_player_specs
                .movement_cooldown
                .apply_negative_effect(effect),
            StatType::GoldFind => modifiable_player_specs.gold_find.apply_effect(effect),
            StatType::ThreatGain => modifiable_player_specs.threat_gain.apply_effect(effect),
            // TODO: Move to character specs
            StatType::RestoreOnHit {
                restore_type,
                skill_type,
            } => {
                let skill_types = match skill_type {
                    Some(skill_type) => vec![skill_type],
                    None => SkillType::iter().collect(),
                };

                for skill_type in skill_types {
                    modifiable_player_specs
                        .restore_on_hit
                        .entry((restore_type, skill_type))
                        .or_default()
                        .apply_effect(effect);
                }
            }
            // /!\ No magic _ to be sure we don't forget when adding new Stats
            // Handled by character
            StatType::Life
            | StatType::LifeRegen
            | StatType::Mana
            | StatType::ManaRegen
            | StatType::Armor(_)
            | StatType::TakeFromManaBeforeLife
            | StatType::TakeFromLifeBeforeMana
            | StatType::Block(_)
            | StatType::BlockDamageTaken
            | StatType::Evade(_)
            | StatType::EvadeDamageTaken
            | StatType::DamageResistance { .. }
            | StatType::StatusResistance { .. }
            | StatType::StatConverter(StatConverterSpecs {
                source:
                    StatConverterSource::MaxLife
                    | StatConverterSource::LifeRegen
                    | StatConverterSource::MaxMana
                    | StatConverterSource::ManaRegen
                    | StatConverterSource::Block(_),
                ..
            })
            | StatType::StatConditionalModifier { .. } => {}
            // Delegate to skills
            StatType::ManaCost { .. }
            | StatType::Damage { .. }
            | StatType::Restore { .. }
            | StatType::CritChance(_)
            | StatType::CritDamage(_)
            | StatType::StatusDuration { .. }
            | StatType::StatusPower { .. }
            | StatType::Speed(_)
            | StatType::Lucky { .. }
            | StatType::StatConverter(StatConverterSpecs {
                source: StatConverterSource::CritDamage | StatConverterSource::Damage { .. }, // | StatConverterSource::DamageOverTime { .. }
                ..
            })
            | StatType::SuccessChance { .. }
            | StatType::SkillLevel(_)
            | StatType::SkillTargetModifier { .. }
            | StatType::SkillConditionalModifier { .. } => {}
            // Other
            StatType::ItemRarity
            | StatType::ItemLevel
            | StatType::GemsFind
            | StatType::PowerLevel
            | StatType::Description(_)
            | StatType::Description2(_) => {}
        }
    }

    modifiable_player_specs
}
