use std::fs::File;
use std::io::BufReader;
use std::collections::HashSet;

use rand::prelude::*;

use pathfinding::directed::astar::astar;

use wfc_image::*;
use image;
use image::GenericImageView;

use roguelike_core::constants::*;
use roguelike_core::map::*;
use roguelike_core::types::*;
use roguelike_core::utils::*;

use crate::generation::*;
use crate::game::*;
use crate::make_map::*;


#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Debug)]
pub enum StructureType {
    Single,
    Line,
    Path,
    Complex,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Structure {
    pub blocks: Vec<Pos>,
    pub typ: StructureType,
}

impl Structure {
    pub fn new() -> Structure {
        return Structure { blocks: Vec::new(), typ: StructureType::Single };
    }

    pub fn add_block(&mut self, block: Pos) {
        self.blocks.push(block);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Debug)]
pub enum ProcCmd {
    Island(usize), // radius
    Entities(EntityType, usize, usize),
    Items(Item, usize, usize),
    Grass(usize, usize),
    Rubble(usize),
    ShortWall(usize),
}

pub fn generate_bare_map(width: u32, height: u32, template_file: &str, rng: &mut SmallRng) -> Map {
    let mut new_map = Map::from_dims(width, height);

    let file = File::open(template_file).unwrap();
    let reader = BufReader::new(file);
    let seed_image = image::load(reader, image::ImageFormat::Png).unwrap();
    let orientations = [Orientation::Original,
                        Orientation::Clockwise90,
                        Orientation::Clockwise180,
                        Orientation::Clockwise270,
                        Orientation::DiagonallyFlipped,
                        Orientation::DiagonallyFlippedClockwise90,
                        Orientation::DiagonallyFlippedClockwise180,
                        Orientation::DiagonallyFlippedClockwise270];
    let map_image = 
        wfc_image::generate_image_with_rng(&seed_image,
                                           core::num::NonZeroU32::new(3).unwrap(),
                                           wfc_image::Size::new(width, height),
                                           &orientations, 
                                           wfc_image::wrap::WrapNone,
                                           ForbidNothing,
                                           wfc_image::retry::NumTimes(3),
                                           rng).unwrap();
    map_image.save("wfc_map.png").unwrap();

    for x in 0..width {
        for y in 0..height {
            let pixel = map_image.get_pixel(x, y);
            if pixel.0[0] == 0 {
                let pos = Pos::new(x as i32, y as i32);
                new_map[pos] = Tile::wall_with(MAP_WALL as char);
            }
         }
    }

    return new_map;
}

pub fn saturate_map(game: &mut Game) -> Pos {
    // find structures-
    // find blocks that are next to exactly one block (search through all tiles, and
    // don't accept tiles that are already accepted).
    //
    // place grass in open areas and perhaps in very enclosed areas
    // place rubble near blocks
    //
    // place goal and exit, and pathing between them, knocking out tiles that
    // block the player from completing the level.

    handle_diagonal_full_tile_walls(game);

    let mut structures = find_structures(&game.data.map);
    println!("{} singles", structures.iter().filter(|s| s.typ == StructureType::Single).count());
    println!("{} lines", structures.iter().filter(|s| s.typ == StructureType::Line).count());
    println!("{} Ls", structures.iter().filter(|s| s.typ == StructureType::Path).count());
    println!("{} complex", structures.iter().filter(|s| s.typ == StructureType::Complex).count());

    let mut to_remove: Vec<usize> = Vec::new();
    for (index, structure) in structures.iter().enumerate() {
        if structure.typ == StructureType::Single {
            if game.rng.gen_range(0.0, 1.0) > 0.1 {
                make_column(&mut game.data.entities, &game.config, structure.blocks[0], &mut game.msg_log);
                to_remove.push(index);
            }
        } else if structure.typ == StructureType::Line { 
            if structure.blocks.len() > 5 {
                let index = game.rng.gen_range(0, structure.blocks.len());
                let block = structure.blocks[index];
                game.data.map[block] = Tile::empty();
                game.data.map[block].surface = Surface::Rubble;
            }
        }

        if structure.typ == StructureType::Line {
           if game.rng.gen_range(0.0, 1.0) < 0.5 {
               let wall_type;
               if game.rng.gen_range(0.0, 1.0) < 0.5 {
                   wall_type = Wall::ShortWall;
               } else {
                   wall_type = Wall::TallWall;
               }

               let diff = sub_pos(structure.blocks[0], structure.blocks[1]);
               for pos in structure.blocks.iter() {
                   if diff.x != 0 {
                       game.data.map[*pos].bottom_wall = wall_type;
                   } else {
                       game.data.map[*pos].left_wall = wall_type;
                   }

                   game.data.map[*pos].blocked = false;
                   game.data.map[*pos].chr = ' ' as u8;
               }
           }
        }
    }

    to_remove.sort();
    to_remove.reverse();
    for index in to_remove.iter() {
        for block in structures[*index].blocks.iter() {
            game.data.map[*block] = Tile::empty();
        }
        structures.swap_remove(*index);
    }

    clear_island(game);

    place_grass(game);

    place_vaults(game);

    let player_id = game.data.find_player().unwrap();
    let player_pos = find_available_tile(game).unwrap();
    game.data.entities.pos[&player_id] = player_pos;

    place_key_and_goal(game, player_pos);

    place_monsters(game);

    clear_island(game);

    return player_pos;
}

fn handle_diagonal_full_tile_walls(game: &mut Game) {
    let (width, height) = game.data.map.size();

    // ensure that diagonal full tile walls do not occur.
    for y in 0..(height - 1) {
        for x in 0..(width - 1) {
            if game.data.map[(x, y)].blocked         && 
               game.data.map[(x + 1, y + 1)].blocked &&
               !game.data.map[(x + 1, y)].blocked    && 
               !game.data.map[(x, y + 1)].blocked {
                   game.data.map[(x + 1, y)] = Tile::wall();
            } else if game.data.map[(x + 1, y)].blocked  && 
                      game.data.map[(x, y + 1)].blocked  &&
                      !game.data.map[(x, y)].blocked &&
                      !game.data.map[(x + 1, y + 1)].blocked {
                   game.data.map[(x, y)] = Tile::wall();
            }
        }
    }
}

fn place_monsters(game: &mut Game) {
    let mut potential_pos = game.data.map.get_empty_pos();

    // add gols
    for _ in 0..5 {
        let len = potential_pos.len();

        if len == 0 {
            break;
        }

        let index = game.rng.gen_range(0, len);
        let pos = potential_pos[index];

        make_gol(&mut game.data.entities, &game.config, pos, &mut game.msg_log);

        potential_pos.remove(index);
    }

    for _ in 0..5 {
        let len = potential_pos.len();
        if len == 0 {
            break;
        }

        let index = game.rng.gen_range(0, len);
        let pos = potential_pos[index];

        make_elf(&mut game.data.entities, &game.config, pos, &mut game.msg_log);

        potential_pos.remove(index);
    }
}

fn place_vaults(game: &mut Game) {
    if game.rng.gen_range(0.0, 1.0) < 0.99 {
        let vault_index = game.rng.gen_range(0, game.vaults.len());

        let (width, height) = game.data.map.size();
        let offset = Pos::new(game.rng.gen_range(0, width), game.rng.gen_range(0, height));

        let vault = &game.vaults[vault_index];
        if offset.x + vault.data.map.size().0  < width &&
           offset.y + vault.data.map.size().1 < height {
               dbg!();
            place_vault(&mut game.data, vault, offset);
        }
    }
}

pub fn place_vault(data: &mut GameData, vault: &Vault, offset: Pos) {
    let (width, height) = vault.data.map.size();

    for x in 0..width {
        for y in 0..height {
            let pos = add_pos(offset, Pos::new(x, y));
            data.map[pos] = vault.data.map[(x, y)];
        }
    }

    let mut entities = vault.data.entities.clone();
    for id in vault.data.entities.ids.iter() {
        entities.pos[id] = 
            add_pos(offset, entities.pos[id]);
    }

    data.entities.merge(&entities);
}

fn place_grass(game: &mut Game) {
    let (width, height) = game.data.map.size();

    let mut potential_grass_pos = Vec::new();
    for x in 0..width {
        for y in 0..height {
            let pos = Pos::new(x, y);

            if !game.data.map[pos].blocked {
                let count = game.data.map.floodfill(pos, 3).len();
                if count > 28 && count < 35 {
                    potential_grass_pos.push(pos);
                }
            }
        }
    }
    potential_grass_pos.shuffle(&mut game.rng);
    let num_grass_to_place = game.rng.gen_range(4, 8);
    let num_grass_to_place = std::cmp::min(num_grass_to_place, potential_grass_pos.len());
    for pos_index in 0..num_grass_to_place {
        let pos = potential_grass_pos[pos_index];
        game.data.map[pos].surface = Surface::Grass;

        for _ in 0..4 {
            let offset_pos = Pos::new(pos.x + game.rng.gen_range(0, 3),
                                      pos.y + game.rng.gen_range(0, 3));
            if game.data.map.is_within_bounds(offset_pos) &&
               !game.data.map[offset_pos].blocked {
                game.data.map[offset_pos].surface = Surface::Grass;
            }
        }

    }
}

fn find_available_tile(game: &mut Game) -> Option<Pos> {
    let mut avail_pos = None;

    let (width, height) = game.data.map.size();
    let mut index = 1.0;
    for x in 0..width {
        for y in 0..height {
            let pos = Pos::new(x, y);

            if !game.data.map[pos].blocked && game.data.has_blocking_entity(pos).is_none() {
                if game.rng.gen_range(0.0, 1.0) < (1.0 / index) {
                    avail_pos = Some(pos);
                }

                index += 1.0;
            }
        }
    }

    return avail_pos;
}

fn place_key_and_goal(game: &mut Game, player_pos: Pos) {
    // place goal and key
    let key_pos = find_available_tile(game).unwrap();
    game.data.map[key_pos] = Tile::empty();
    make_key(&mut game.data.entities, &game.config, key_pos, &mut game.msg_log);

    let goal_pos = find_available_tile(game).unwrap();
    game.data.map[goal_pos] = Tile::empty();
    make_exit(&mut game.data.entities, &game.config, goal_pos, &mut game.msg_log);

    fn blocked_tile_cost(pos: Pos, map: &Map) -> i32 {
        if map[pos].blocked {
            return 15;
        } 

        return 0;
    }

    // clear a path to the key
    let key_path = 
        astar(&player_pos,
              |&pos| game.data.map.neighbors(pos).iter().map(|p| (*p, 1)).collect::<Vec<(Pos, i32)>>(),
              |&pos| blocked_tile_cost(pos, &game.data.map) + distance(player_pos, pos) as i32,
              |&pos| pos == key_pos);

    if let Some((results, _cost)) = key_path {
        for pos in results {
            if game.data.map[pos].blocked {
                game.data.map[pos] = Tile::empty();
            }
        }
    }

    // clear a path to the goal
    let goal_path = 
        astar(&player_pos,
              |&pos| game.data.map.neighbors(pos).iter().map(|p| (*p, 1)).collect::<Vec<(Pos, i32)>>(),
              |&pos| blocked_tile_cost(pos, &game.data.map) + distance(player_pos, pos) as i32,
              |&pos| pos == goal_pos);

    if let Some((results, _cost)) = goal_path {
        for pos in results {
            if game.data.map[pos].blocked {
                game.data.map[pos] = Tile::empty();
            }
        }
    }
}

fn clear_island(game: &mut Game) {
    fn dist(pos1: Pos, pos2: Pos) -> f32 {
        return (((pos1.x - pos2.x).pow(2) + (pos1.y - pos2.y).pow(2)) as f32).sqrt();
    }

    let (width, height) = game.data.map.size();
    let x_mid = width / 2;
    let y_mid = height / 2;
    let mid_pos = Pos::new(x_mid, y_mid);

    for y in 0..height {
        for x in 0..width {
            let pos = Pos::new(x, y);

            if dist(pos, mid_pos) >= ISLAND_DISTANCE as f32 {
                game.data.map[pos] = Tile::water();
                game.data.map[pos].chr = MAP_WATER;

                for entity_id in game.data.has_entities(pos).clone() {
                    game.data.remove_entity(entity_id);
                }
            }
        }
    }
}


fn process_block(block: Pos, structure: &mut Structure, map: &Map, seen: &mut HashSet<Pos>) {
    let adjacent = adjacent_blocks(block, map, seen);

    let mut needs_processing = false;
    if adjacent.len() == 1 {
        needs_processing = true;
        if structure.typ == StructureType::Line && structure.blocks.len() > 1 {
            let len = structure.blocks.len();
            if sub_pos(structure.blocks[len - 2], structure.blocks[len - 1]) !=
               sub_pos(structure.blocks[len - 1], adjacent[0]) {
               structure.typ = StructureType::Path;
            }
        }

    } else if adjacent.len() > 1 {
        needs_processing = true;

        // this structure must be complex- if there are multiple adj, they are new
        // meaning we split in at least two directions
        structure.typ = StructureType::Complex;
    }

    if needs_processing {
        for adj in adjacent.iter() {
            structure.add_block(*adj);
            seen.insert(*adj);
        }

        for adj in adjacent.iter() {
            process_block(*adj, structure, map, seen);
        }
    }
}

fn adjacent_blocks(block: Pos, map: &Map, seen: &HashSet<Pos>) -> Vec<Pos> {
    let mut result = Vec::new();

    let adjacents = [move_x(block, 1), move_y(block, 1), move_x(block, -1), move_y(block, -1)];
    for adj in adjacents.iter() {
        if map.is_within_bounds(*adj) && map[*adj].blocked && !seen.contains(&adj) {
            result.push(*adj);
        }
    }

    return result;
}

#[test]
fn test_adjacent_blocks() {
    let mut map = Map::from_dims(5, 5);
    let mid = Pos::new(2, 2);
    map[(2, 2)] = Tile::wall();

    map[(1, 2)] = Tile::wall();
    map[(2, 1)] = Tile::wall();
    map[(3, 2)] = Tile::wall();
    map[(2, 3)] = Tile::wall();

    let mut seen = HashSet::new();

    assert_eq!(4, adjacent_blocks(Pos::new(2, 2), &map, &seen).len());
    assert_eq!(2, adjacent_blocks(Pos::new(1, 1), &map, &seen).len());
    assert_eq!(1, adjacent_blocks(Pos::new(2, 1), &map, &seen).len());
    seen.insert(Pos::new(1, 2));
    assert_eq!(3, adjacent_blocks(Pos::new(2, 2), &map, &seen).len());
}

fn find_structures(map: &Map) -> Vec<Structure> {
    let (width, height) = map.size();
    let mut blocks = Vec::new();
    for y in 0..height {
        for x in 0..width {
            if map[(x, y)].blocked {
                blocks.push(Pos::new(x, y));
            }
        }
    }

    let mut structures = Vec::new();
    let mut seen: HashSet<Pos> = HashSet::new();
    for block in blocks {
        if !seen.contains(&block) {
            let mut structure = Structure::new();

            let adjacent = adjacent_blocks(block, &map, &seen);

            if adjacent.len() != 2 {
                structure.add_block(block);
                seen.insert(block);

                if adjacent.len() == 1 {
                    // found start of a structure (line, L, or complex)- process structure
                    structure.typ = StructureType::Line;
                    process_block(block, &mut structure, map, &mut seen);
                } else if adjacent.len() > 2 {
                    // found part of a complex structure- process all peices
                    structure.typ = StructureType::Complex;

                    for adj in adjacent.iter() {
                        seen.insert(*adj);
                    }

                    for adj in adjacent {
                        process_block(adj, &mut structure, map, &mut seen);
                    }
                }

                structures.push(structure);
            }
            // else we are in the middle of a line, so we will pick it up later
        }
    }

    return structures;
}

#[test]
fn test_find_simple_structures() {
    let mut map = Map::from_dims(5, 5);

    // find a single line
    map[(0, 2)] = Tile::wall();
    map[(1, 2)] = Tile::wall();
    map[(2, 2)] = Tile::wall();
    let structures = find_structures(&map);
    assert_eq!(1, structures.len());
    assert_eq!(StructureType::Line, structures[0].typ);
    assert_eq!(3, structures[0].blocks.len());

    // add a lone block and check that it is found along with the line
    map[(0, 0)] = Tile::wall();
    let structures = find_structures(&map);
    assert_eq!(2, structures.len());
    assert!(structures.iter().find(|s| s.typ == StructureType::Single).is_some());
    assert!(structures.iter().find(|s| s.typ == StructureType::Line).is_some());

    // add a vertical line and check that all structures are found
    map[(4, 0)] = Tile::wall();
    map[(4, 1)] = Tile::wall();
    map[(4, 2)] = Tile::wall();
    map[(4, 3)] = Tile::wall();
    let structures = find_structures(&map);
    assert_eq!(3, structures.len());
    assert!(structures.iter().find(|s| s.typ == StructureType::Single).is_some());
    assert!(structures.iter().filter(|s| s.typ == StructureType::Line).count() == 2);
}

#[test]
fn test_find_complex_structures() {
    let mut map = Map::from_dims(5, 5);

    // lay down an L
    map[(0, 2)] = Tile::wall();
    map[(1, 2)] = Tile::wall();
    map[(2, 2)] = Tile::wall();
    map[(2, 3)] = Tile::wall();
    let structures = find_structures(&map);
    assert_eq!(1, structures.len());
    assert_eq!(StructureType::Path, structures[0].typ);
    assert_eq!(4, structures[0].blocks.len());

    // turn it into a 'complex' structure and check that it is discovered
    map[(2, 1)] = Tile::wall();
    let structures = find_structures(&map);
    assert_eq!(1, structures.len());
    assert_eq!(StructureType::Complex, structures[0].typ);
    assert_eq!(5, structures[0].blocks.len());
}
