use super::*;
use crate::game::native_entity_refs::tests::{bind_local_player, run_fresh_cases, with_game};
use std::sync::Arc;
use sunrise_client::motion::PlayerMotion;
use sunrise_core::random::{LegacyRandom, RandomSource as _};
use sunrise_world::loot::{LootData, PackOverlay};

#[test]
#[ignore = "daily"]
fn rolls_a_kind_that_is_never_spawned() {
    if run_fresh_cases(
        "game::native_entity_refs::death_loot::tests::rolls_a_kind_that_is_never_spawned",
        "SUNRISE_ROLL_ENTITY_LOOT_CHILD",
        &["kind_context"],
    ) {
        return;
    }
    with_game(|game| {
        let files = [(
            "loot_table/test:farm".into(),
            vec![br#"{
                "type":"minecraft:entity","pools":[
                  {"rolls":1,"condition":{"type":"minecraft:entity_properties","entity":"this",
                      "predicate":{"minecraft:entity_type":"minecraft:pig"}},
                   "entries":[{"type":"minecraft:item","name":"minecraft:leather"}]},
                  {"rolls":1,"condition":{"type":"minecraft:killed_by_player"},
                   "entries":[{"type":"minecraft:item","name":"minecraft:emerald"}]},
                  {"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:iron_ingot",
                      "modifier":{"type":"minecraft:enchanted_count_increase","enchantment":"minecraft:looting","count":1}}]},
                  {"rolls":1,"condition":{"type":"minecraft:damage_source_properties","predicate":{
                      "tags":[{"id":"minecraft:mob_attack","expected":true}],
                      "source_entity":{"minecraft:entity_type":"minecraft:zombie"}}},
                   "entries":[{"type":"minecraft:item","name":"minecraft:paper"}]}
                ]}"#
            .to_vec()],
        )];
        let loot = LootData::load(
            &game.options.resources,
            &PackOverlay::new(&files),
            game.res.item_defaults(),
        )
        .unwrap();
        game.server_loot.set(Some(Arc::new(loot)));
        game.player = Some(PlayerMotion::new(game.motion, Vec3::ZERO).unwrap());
        bind_local_player(game, 11, 123);
        let credited = game.player(123).unwrap();

        let zombie = game
            .res
            .mob_types()
            .get("minecraft:zombie")
            .unwrap()
            .clone();
        let mut killer = Mob::new(13, zombie, Vec3::ZERO, 0.0);
        let mut sword = ItemStack::new("minecraft:iron_sword", 1);
        sword.enchant("minecraft:looting", 3);
        killer.set_equipment_stack(0, Some(sword));
        game.mobs.push(killer);
        game.next_entity = game.next_entity.max(13);
        let killer = game.entity(13).unwrap();
        let level = game.level("minecraft:overworld").unwrap();
        let pig = Identifier::parse("minecraft:pig").unwrap();
        let farm = Identifier::parse("test:farm").unwrap();
        let mob_attack = Identifier::parse("minecraft:mob_attack").unwrap();
        let by_zombie = LootDamageSource::new(&mob_attack, Some(killer), Some(killer));
        let unarmed = LootDamageSource::new(&mob_attack, None, None);
        let at = [8.5, 64.0, 8.5];
        let next_entity = game.next_entity;
        let roll = |game: &mut Game<'_>, kind: &Identifier, source, credited| {
            game.level_roll_entity_loot(level, kind, Some(&farm), at, source, credited)
                .map(|stacks| {
                    stacks
                        .iter()
                        .map(|stack| (stack.item.to_string(), stack.count))
                        .collect::<Vec<_>>()
                })
        };
        let items = |list: &[(&str, u32)]| {
            Ok(list
                .iter()
                .map(|(item, count)| (item.to_string(), *count))
                .collect::<Vec<_>>())
        };
        // THIS_ENTITY is the kind; a killed_by_player pool needs the credited player;
        // Looting III on the attacker's sword adds 3.
        assert_eq!(
            roll(game, &pig, unarmed, None),
            items(&[("minecraft:leather", 1), ("minecraft:iron_ingot", 1)])
        );
        assert_eq!(
            roll(game, &pig, by_zombie, Some(credited)),
            items(&[
                ("minecraft:leather", 1),
                ("minecraft:emerald", 1),
                ("minecraft:iron_ingot", 4),
                ("minecraft:paper", 1),
            ])
        );
        let cow = Identifier::parse("minecraft:cow").unwrap();
        assert_eq!(
            roll(game, &cow, unarmed, None),
            items(&[("minecraft:iron_ingot", 1)])
        );
        // The kind's own table when none is named.
        assert!(
            game.level_roll_entity_loot(level, &pig, None, at, unarmed, None)
                .is_ok()
        );
        // Nothing joined the level or dropped.
        assert_eq!(game.next_entity, next_entity);
        assert_eq!(game.mobs.len(), 1);
        assert!(game.dropped.is_empty());
        assert!(game.orbs.is_empty());

        *game.world.random = LegacyRandom::new(741);
        let expected_next = game.world.random.clone().next_long();
        let stick = Identifier::parse("minecraft:stick").unwrap();
        let unknown_type = Identifier::parse("test:no_damage_type").unwrap();
        let unknown_table = Identifier::parse("test:no_table").unwrap();
        let stale = EntityRef::new(killer.epoch(), 9999);
        let mut refused = |kind, table, source| {
            game.level_roll_entity_loot(level, kind, table, at, source, None)
                .unwrap_err()
        };
        assert_eq!(
            refused(&stick, None, unarmed),
            DeathLootError::UnknownKind
        );
        assert_eq!(
            refused(&pig, Some(&unknown_table), unarmed),
            DeathLootError::UnknownTable
        );
        assert_eq!(
            refused(
                &pig,
                Some(&farm),
                LootDamageSource::new(&unknown_type, None, None)
            ),
            DeathLootError::UnknownDamageType
        );
        assert_eq!(
            refused(
                &pig,
                Some(&farm),
                LootDamageSource::new(&mob_attack, Some(stale), None)
            ),
            DeathLootError::StaleEntity
        );
        assert_eq!(game.world.random.next_long(), expected_next);

        game.options.execution_role = crate::game::execution_role::ExecutionRole::Client;
        assert_eq!(
            game.level_roll_entity_loot(level, &pig, Some(&farm), at, unarmed, None),
            Err(DeathLootError::Unavailable)
        );
    });
}
