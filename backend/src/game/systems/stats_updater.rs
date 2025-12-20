use shared::data::{
    area::AreaThreat,
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
