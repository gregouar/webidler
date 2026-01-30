use shared::data::{
    area::AreaThreat,
    conditional_modifier::{Condition, ConditionalModifier},
    player::{CharacterSpecs, CharacterState},
    stat_effect::{
        EffectsMap, Modifier, StatConverterSource, StatConverterSpecs, StatEffect, StatType,
    },
};

// maybe AreaThreat should be some kind of more generic "Context"
pub fn stats_map_to_vec(effects: &EffectsMap, area_threat: &AreaThreat) -> Vec<StatEffect> {
    combine_effects(effects.into(), area_threat)
}

// maybe AreaThreat should be some kind of more generic "Context"
pub fn combine_effects(mut effects: Vec<StatEffect>, area_threat: &AreaThreat) -> Vec<StatEffect> {
    let to_add: Vec<_> = effects
        .iter()
        .flat_map(|effect| {
            if let StatType::StatConverter(StatConverterSpecs {
                source: StatConverterSource::ThreatLevel,
                target_stat,
                target_modifier,
                ..
            }) = &effect.stat
            {
                Some(StatEffect {
                    stat: *target_stat.clone(),
                    modifier: *target_modifier,
                    value: if target_stat.is_multiplicative() {
                        ((1.0 + effect.value * 0.01).powf(area_threat.threat_level as f64) - 1.0)
                            * 100.0
                    } else {
                        effect.value * area_threat.threat_level as f64
                    },
                    bypass_ignore: false,
                })
            } else {
                None
            }
        })
        .collect();

    effects.extend(to_add);
    sort_stat_effects(&mut effects);

    effects
}

pub fn sort_stat_effects(effects: &mut [StatEffect]) {
    effects.sort_by_key(|e| {
        (
            match e.stat {
                StatType::StatConverter(ref specs) => match specs.target_modifier {
                    Modifier::Flat => 1,
                    Modifier::Multiplier => 3,
                },
                _ => match e.modifier {
                    Modifier::Flat => 0,
                    Modifier::Multiplier => 2,
                },
            },
            e.stat.clone(),
        )
    });
}

pub fn compute_conditional_modifiers<'a>(
    character_specs: &CharacterSpecs,
    character_state: &CharacterState,
    conditional_modifiers: &'a [ConditionalModifier],
) -> Vec<&'a StatEffect> {
    conditional_modifiers
        .iter()
        .filter(|conditional_modifier| {
            conditional_modifier
                .conditions
                .iter()
                .all(|condition| check_condition(character_specs, character_state, condition))
        })
        .flat_map(|conditional_modifier| conditional_modifier.effects.iter())
        .collect()
}

pub fn check_condition(
    character_specs: &CharacterSpecs,
    character_state: &CharacterState,
    condition: &Condition,
) -> bool {
    match condition {
        Condition::HasStatus(stat_status_type) => character_state
            .statuses
            .iter()
            .any(|(status_specs, _)| (*stat_status_type).is_match(&status_specs.into())),
        Condition::MaximumLife => character_state.life >= character_specs.max_life * 0.99,
    }
}
