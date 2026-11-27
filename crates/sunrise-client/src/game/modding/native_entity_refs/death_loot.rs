//! A server-side roll of `LivingEntity.dropFromLootTable`, collected instead of spawned, for a
//! kind created and never spawned.

#[cfg(test)]
#[path = "death_loot_tests.rs"]
mod tests;

use crate::game::{Game, commands::Target, creatures::from_loot, native_server::PlayerRead};
use mod_api::{DeathLootError, EntityRef, Identifier, LevelRef, LootDamageSource, PlayerRef};
use sunrise_client::mobs::{Mob, drops};
use sunrise_core::geometry::Vec3;
use sunrise_items::items::ItemStack;
use sunrise_world::{
    attribute_map::ValueKind,
    loot::{EntityLootParams, LootDamage},
};

impl Game<'_> {
    /// `LevelRef::roll_entity_loot`: `LivingEntity.dropFromLootTable` for a `kind` created at
    /// `origin` and never spawned (`EntityType.create` without `finalizeSpawn`), collected. The
    /// mob is built off the books: it takes no entity id, joins no list and fires nothing.
    pub(in crate::game) fn level_roll_entity_loot(
        &mut self,
        level: LevelRef,
        kind: &Identifier,
        table: Option<&Identifier>,
        origin: [f64; 3],
        source: LootDamageSource<'_>,
        last_damage_player: Option<PlayerRef>,
    ) -> Result<Vec<ItemStack>, DeathLootError> {
        if !self.is_logical_server() {
            return Err(DeathLootError::Unavailable);
        }
        if !self.level_handle_current(level) || self.dim.key() != level.dimension() {
            return Err(DeathLootError::StaleEntity);
        }
        let mob_type = self
            .res
            .mob_types()
            .get(&kind.to_string())
            .cloned()
            .ok_or(DeathLootError::UnknownKind)?;
        let damage_type = source.damage_type.to_string();
        if self.health_resources.damage.id(&damage_type).is_none() {
            return Err(DeathLootError::UnknownDamageType);
        }
        let mob = Mob::new(0, mob_type, Vec3::new(origin[0], origin[1], origin[2]), 0.0);
        let table = match table {
            Some(table) => table.to_string(),
            None => drops::death_loot_table(&mob, true).ok_or(DeathLootError::UnknownTable)?,
        };
        let loot = self.server_loot.get().ok_or(DeathLootError::Unavailable)?;
        if !loot.has_table(&table) {
            return Err(DeathLootError::UnknownTable);
        }

        let resolve = |handle: Option<EntityRef>| {
            handle
                .map(|e| {
                    self.native_entity_target(e)
                        .ok_or(DeathLootError::StaleEntity)
                })
                .transpose()
        };
        let attacker = resolve(source.causing)?;
        let direct = resolve(source.direct)?;
        let credited = last_damage_player
            .map(|player| {
                let read = self
                    .player_handle_read(player)
                    .ok_or(DeathLootError::StaleEntity)?;
                Ok(match read {
                    PlayerRead::Acting { body: Some(_), .. } => Target::Player,
                    PlayerRead::Guest(guest) => Target::Guest(guest.conn),
                    PlayerRead::Acting { body: None, .. } => {
                        return Err(DeathLootError::StaleEntity);
                    }
                })
            })
            .transpose()?;

        let this = drops::loot_entity(&mob, None);
        let source_entity = attacker.map(|t| self.target_loot_entity(t));
        let direct_entity = if direct == attacker {
            source_entity.clone()
        } else {
            direct.map(|t| self.target_loot_entity(t))
        };
        let (player, luck) = match credited {
            Some(t) => {
                let luck = self
                    .native_target_attribute(t, "minecraft:luck", ValueKind::Computed)
                    .unwrap_or(0.0) as f32;
                (Some(self.target_loot_entity(t)), luck)
            }
            None => (None, 0.0),
        };
        let params = EntityLootParams {
            this,
            damage: Some(LootDamage {
                kind: damage_type,
                source: source_entity,
                direct: direct_entity,
            }),
            player,
            luck,
            attacker: None,
        };
        let world_seed = self.level.seed;
        let stacks = self.caller_loot(Some(origin), None).with(|| {
            match loot.random_sequence(&table) {
                Some(sequence) => {
                    let random = self.rand_sequences.get(sequence, world_seed);
                    loot.roll_entity(&table, random, &params)
                }
                None => loot.roll_entity(&table, &mut self.world.random, &params),
            }
        });
        Ok(stacks
            .iter()
            .filter(|stack| !stack.is_empty())
            .map(from_loot)
            .collect())
    }
}
