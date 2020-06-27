use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use tcod::line::*;

use crate::ai::Behavior;
use crate::constants::{HAMMER_DAMAGE};
use crate::map::{Surface};
use crate::types::*;
use crate::movement::{Reach, MoveMode, check_collision, MoveType, Movement};
use crate::messaging::*;


pub fn distance(pos1: Pos, pos2: Pos) -> i32 {
    //return (((pos1.x - pos2.x).pow(2) + (pos1.y - pos2.y).pow(2)) as f32).sqrt() as i32;
    let line = Line::new(pos1.to_tuple(), pos2.to_tuple()).into_iter();
    return line.count() as i32;
}

pub fn pos_mag(pos: Pos) -> i32 {
    return distance(Pos::new(0, 0), pos);
}

pub fn signedness(value: i32) -> i32 {
    if value == 0 {
        return 0;
    } else {
        return value.signum();
    }
}

pub fn in_direction_of(start: Pos, end: Pos) -> Pos {
    let dpos = sub_pos(end, start);
    let dx = signedness(dpos.x);
    let dy = signedness(dpos.y);

    return add_pos(start, Pos::new(dx, dy));
}

pub fn is_ordinal(delta: Pos) -> bool {
    return (delta.x == 0 && delta.y != 0) || 
           (delta.y == 0 && delta.x != 0);
}

pub fn push_attack(handle: EntityId,
                   target: EntityId,
                   delta_pos: Pos,
                   move_into: bool,
                   data: &mut GameData,
                   msg_log: &mut MsgLog) {
    let mut killed = false;
    let mut damage = 0;

    for _ in 0..pos_mag(delta_pos) {
        let pos = data.entities.pos[&handle];
        let other_pos = data.entities.pos[&target];
        let diff = other_pos - pos;

        let x_diff = signedness(diff.x);
        let y_diff = signedness(diff.y);

        let move_result = check_collision(other_pos, x_diff, y_diff, data);

        let past_pos = move_by(other_pos, Pos::new(x_diff, y_diff));

        if move_into {
            data.entities.move_to(handle, other_pos);

            let movement = Movement::new(past_pos, MoveType::Move, None);
            msg_log.log(Msg::Moved(target, movement, past_pos));
        }

        if move_result.no_collision() {
            // if not blocked, push the other entity, taking their space
            data.entities.set_pos(target, past_pos);

            let movement = Movement::new(past_pos, MoveType::Move, None);
            msg_log.log(Msg::Moved(target, movement, past_pos));
        } else {
            // otherwise crush them against the wall/entity
            data.entities.alive[&target] = false;
            data.entities.blocks[&target] = false;
            damage = data.entities.fighter[&target].hp;

            killed = true;

            // once we crush an entity, we lose the rest of the move
            break;
        }
    }

    msg_log.log(Msg::Pushed(handle, target));
    if killed {
        msg_log.log(Msg::Killed(handle, target, damage));
    } else {
        data.entities.messages[&target].push(Message::Attack(handle));
    }
}

// TODO 
// crush at a location
//pub fn crush_at(handle: EntityId, pos: Pos, entities: &mut Entities, msg_log: &mut MsgLog) {
//    if let 
//target: EntityId, 
//}

pub fn crush(handle: EntityId, target: EntityId, entities: &mut Entities, msg_log: &mut MsgLog) {
    let damage = entities.fighter.get(&target).map_or(0, |f| f.hp);
    if damage > 0 {
        entities.take_damage(target, damage);

        entities.alive[&target] = false;
        entities.blocks[&target] = false;

        msg_log.log(Msg::Killed(handle, target, damage));
    }
}

pub fn attack(entity: EntityId, target: EntityId, data: &mut GameData, msg_log: &mut MsgLog) {
    if data.using(entity, Item::Hammer) {
        data.entities.alive[&target] = false;
        data.entities.blocks[&target] = false;

        data.entities.take_damage(target, HAMMER_DAMAGE);
        data.entities.messages[&target].push(Message::Attack(entity));

        // NOTE assumes that this kills the enemy
        msg_log.log(Msg::Killed(entity, target, HAMMER_DAMAGE));

        let hit_pos = data.entities.pos[&target];
        // NOTE this creates rubble even if the player somehow is hit by a hammer...
        if data.map[hit_pos].surface == Surface::Floor {
            data.map[hit_pos].surface = Surface::Rubble;
        }
    } else if data.using(target, Item::Shield) {
        let pos = data.entities.pos[&entity];
        let other_pos = data.entities.pos[&target];
        let diff = other_pos - pos;

        let x_diff = diff.x.signum();
        let y_diff = diff.y.signum();

        let past_pos = move_by(other_pos, Pos::new(x_diff, y_diff));

        if !data.map.is_blocked_by_wall(other_pos, x_diff, y_diff).is_some() &&
           !data.has_blocking_entity(past_pos).is_some() {
            data.entities.set_pos(target, past_pos);
            data.entities.move_to(entity, other_pos);

            data.entities.messages[&target].push(Message::Attack(entity));
        }
    } else {
        let damage = data.entities.fighter.get(&entity).map_or(0, |f| f.power) -
                     data.entities.fighter.get(&target).map_or(0, |f| f.defense);
        if damage > 0 {
            data.entities.take_damage(target, damage);

            msg_log.log(Msg::Attack(entity, target, damage));
            // TODO consider moving this to the Attack msg
            if data.entities.fighter[&target].hp <= 0 {
                data.entities.alive[&target] = false;
                data.entities.blocks[&target] = false;

                msg_log.log(Msg::Killed(entity, target, damage));
            }

            data.entities.messages[&target].push(Message::Attack(entity));
        }
    }
}

pub fn stab(handle: EntityId, target: EntityId, entities: &mut Entities, msg_log: &mut MsgLog) {
    let damage = entities.fighter.get(&target).map_or(0, |f| f.hp);

    if damage != 0 {
        msg_log.log(Msg::Attack(handle, target, damage));

        entities.alive[&target] = false;
        entities.blocks[&target] = false;

        msg_log.log(Msg::Killed(handle, target, damage));

        entities.messages[&target].push(Message::Attack(handle));
    } else {
        panic!("Stabbed an enemy with no hp?");
    }
}

pub fn item_primary_at(entity_id: EntityId, entities: &mut Entities, index: usize) -> bool {
    let inv_len = entities.inventory[&entity_id].len();

    if inv_len <= index {
        return false;
    }

    let item_id = entities.inventory[&entity_id][index];
    let is_primary =
        entities.item[&item_id].class() == ItemClass::Primary;

    return is_primary;
}

pub fn add_pos(pos1: Pos, pos2: Pos) -> Pos {
    return Pos::new(pos1.x + pos2.x, pos1.y + pos2.y);
}

pub fn sub_pos(pos1: Pos, pos2: Pos) -> Pos {
    return Pos::new(pos1.x - pos2.x, pos1.y - pos2.y);
}

pub fn move_towards(start: Pos, end: Pos, num_blocks: usize) -> Pos {
    let line = Line::new(start.to_tuple(), end.to_tuple()).into_iter();

    return Pos::from(line.skip(num_blocks).next().unwrap_or(end.to_tuple()));
}

pub fn rand_from_pos(pos: Pos) -> f32 {
    return rand_from_x_y(pos.x, pos.y);
}

pub fn rand_from_x_y(x: i32, y: i32) -> f32 {
    let mut hasher = DefaultHasher::new();

    (x as u32).hash(&mut hasher);
    (y as u32).hash(&mut hasher);
 
    let result: u64 = hasher.finish();

    return ((result & 0xFFFFFFFF) as f32) / 4294967295.0;
}

pub fn lerp(first: f32, second: f32, scale: f32) -> f32 {
    return first + ((second - first) * scale);
}

pub fn lerp_color(color1: Color, color2: Color, scale: f32) -> Color {
    return Color {
        r: lerp(color1.r as f32, color2.r as f32, scale) as u8,
        g: lerp(color1.g as f32, color2.g as f32, scale) as u8,
        b: lerp(color1.b as f32, color2.b as f32, scale) as u8,
        a: lerp(color1.a as f32, color2.a as f32, scale) as u8,
    };
}

pub fn reach_by_mode(move_mode: MoveMode) -> Reach {
    match move_mode {
        MoveMode::Sneak => {
            Reach::Single(1)
        }

        MoveMode::Walk => {
            Reach::Single(1)
        }

        MoveMode::Run => {
            Reach::Single(2)
        }
    }
}

pub fn clamp<N: Ord>(val: N, min: N, max: N) -> N {
    if val < min {
        return min;
    } else if val > max {
        return max;
    } 

    return val;
}

pub fn clampf(val: f32, min: f32, max: f32) -> f32 {
    if val < min {
        return min;
    } else if val > max {
        return max;
    } 

    return val;
}

pub fn move_by(start: Pos, diff: Pos) -> Pos {
    return Pos::new(start.x + diff.x, start.y + diff.y);
}

pub fn move_y(pos: Pos, offset_y: i32) -> Pos {
    return Pos::new(pos.x, pos.y + offset_y);
}

pub fn move_x(pos: Pos, offset_x: i32) -> Pos {
    return Pos::new(pos.x + offset_x, pos.y);
}

pub fn next_pos(pos: Pos, delta_pos: Pos) -> Pos {
    let mut next_pos = add_pos(pos, delta_pos);

    if delta_pos.x != 0 {
        next_pos.x += delta_pos.x.signum();
    }

    if delta_pos.y != 0 {
        next_pos.y += delta_pos.y.signum();
    }

    return next_pos;
}

pub fn can_stab(data: &GameData, entity: EntityId, target: EntityId) -> bool {
    let entity_pos = data.entities.pos[&entity];
    let target_pos = data.entities.pos[&target];
    // NOTE this is not generic- uses EntityType::Enemy
    return data.entities.typ[&target] == EntityType::Enemy &&
           data.using(entity, Item::Dagger) &&
           data.clear_path(entity_pos, target_pos) &&
           !matches!(data.entities.behavior.get(&target), Some(Behavior::Attacking(_)));
}

pub fn dxy(start_pos: Pos, end_pos: Pos) -> (i32, i32) {
    return (end_pos.x - start_pos.x, end_pos.y - start_pos.y);
}

pub fn move_next_to(start_pos: Pos, end_pos: Pos) -> Pos {
    if distance(start_pos, end_pos) <= 1 {
        return start_pos;
    }

    let mut line = Line::new(start_pos.to_tuple(), end_pos.to_tuple()).into_iter();

    let mut second_to_last = line.next().unwrap();

    while let Some(pos) = line.next() {
        if pos != end_pos.to_tuple() {
            second_to_last = pos;
        }
    }

    return Pos::from(second_to_last);
}

#[test]
pub fn test_move_next_to() {
    assert_eq!(move_next_to(Pos::new(0, 0), Pos::new(5, 5)), Pos::new(4, 4));
    assert_eq!(move_next_to(Pos::new(0, 0), Pos::new(1, 1)), Pos::new(0, 0));
    assert_eq!(move_next_to(Pos::new(0, 0), Pos::new(-5, -5)), Pos::new(-4, -4));
    assert_eq!(move_next_to(Pos::new(0, 0), Pos::new(0, 5)), Pos::new(0, 4));
    assert_eq!(move_next_to(Pos::new(0, 0), Pos::new(5, 0)), Pos::new(4, 0));
}

#[test]
pub fn test_in_direction_of() {
    let start = Pos::new(1, 1);

    assert_eq!(in_direction_of(start, Pos::new(0, 0)), Pos::new(0, 0));
    assert_eq!(in_direction_of(start, Pos::new(10, 10)), Pos::new(2, 2));
    assert_eq!(in_direction_of(start, Pos::new(10, 1)), Pos::new(2, 1));
    assert_eq!(in_direction_of(start, Pos::new(1, 10)), Pos::new(1, 2));
    assert_eq!(in_direction_of(start, Pos::new(1, -10)), Pos::new(1, 0));
    assert_eq!(in_direction_of(start, Pos::new(-10, 1)), Pos::new(0, 1));
    assert_eq!(in_direction_of(start, Pos::new(-10, -10)), Pos::new(0, 0));
}
