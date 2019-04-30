#![allow(dead_code)]

extern crate tcod;
extern crate rand;
extern crate serde;
#[macro_use]extern crate serde_derive;
extern crate serde_json;
extern crate num;

mod types;
mod constants;
mod map;
mod ai;


#[allow(unused_imports)]use std::cmp;
#[allow(unused_imports)]use std::fs::File;
#[allow(unused_imports)]use std::io::BufReader;
#[allow(unused_imports)]use std::io::Read;

#[allow(unused_imports)]use tcod::map::{Map as FovMap};
#[allow(unused_imports)]use tcod::console::*;
#[allow(unused_imports)]use tcod::colors::*;
#[allow(unused_imports)]use tcod::input::Key;
#[allow(unused_imports)]use tcod::input::KeyCode::*;
#[allow(unused_imports)]use tcod::input::{self, Event, Mouse};
#[allow(unused_imports)]use tcod::AsNative;
#[allow(unused_imports)]use tcod::image;

use types::*;
use constants::*;
use map::*;
use ai::*;


fn handle_keys(game: &mut Game,
               key: Key,
               map: &mut Map,
               objects: &mut Vec<Object>,
               inventory: &mut Vec<Object>,
               messages: &mut Messages) -> PlayerAction {
    use PlayerAction::*;

    let player_alive = objects[PLAYER].alive;

    match (key, player_alive) {
        (Key { code: Up,      .. }, true)  |
        (Key { code: Number8, .. }, true)  |
        (Key { code: NumPad8, .. }, true) => {
            player_move_or_attack(0, -1, map, objects, messages);
            TookTurn
        }

        (Key { code: Down,    .. }, true) |
        (Key { code: Number2, .. }, true) |
        (Key { code: NumPad2, .. }, true) => {
            player_move_or_attack(0, 1, map, objects, messages);
            TookTurn
        }

        (Key { code: Left,    .. }, true) |
        (Key { code: Number4, .. }, true) |
        (Key { code: NumPad4, .. }, true) => {
            player_move_or_attack(-1, 0, map, objects, messages);
            TookTurn
        }

        (Key { code: Right,   .. }, true) |
        (Key { code: Number6, .. }, true) |
        (Key { code: NumPad6, .. }, true) => {
            player_move_or_attack(1, 0, map, objects, messages);
            TookTurn
        }

        (Key { code: Number9, .. }, true)  |
        (Key { code: NumPad9, .. }, true) => {
            player_move_or_attack(1, -1, map, objects, messages);
            TookTurn
        }

        (Key { code: Number3, .. }, true) |
        (Key { code: NumPad3, .. }, true) => {
            player_move_or_attack(1, 1, map, objects, messages);
            TookTurn
        }

        (Key { code: Number1, .. }, true) |
        (Key { code: NumPad1, .. }, true) => {
            player_move_or_attack(-1, 1, map, objects, messages);
            TookTurn
        }

        (Key { code: Number7, .. }, true) |
        (Key { code: NumPad7, .. }, true) => {
            player_move_or_attack(-1, -1, map, objects, messages);
            TookTurn
        }

        (Key { code: Number5, .. }, true) |
        (Key { code: NumPad5, .. }, true) => {
            objects[PLAYER].momentum = Some((0, 0));
            TookTurn
        }

        (Key { code: Enter, alt: true, .. }, _) => {
            let fullscreen = game.root.is_fullscreen();
            game.root.set_default_foreground(WHITE);
            game.root.set_fullscreen(!fullscreen);
            DidntTakeTurn
        },

        (Key {printable: 'g', .. }, true) => {
            let item_id = objects.iter().position(|object| {
                object.pos() == objects[PLAYER].pos() && object.item.is_some()
            });
            if let Some(item_id) = item_id {
                pick_item_up(item_id, objects, inventory, messages);
            }
            DidntTakeTurn
        }

        (Key {printable: 'i', .. }, true) => {
            let inventory_index =
                inventory_menu(inventory,
                               "Press the key next to an item to use it, or any other to cancel.\n",
                               &mut game.root);
            if let Some(inventory_index) = inventory_index {
                use_item(inventory_index, inventory, objects, messages);
            }
            DidntTakeTurn
        }

        (Key { code: Escape, .. }, _) => Exit,

        (Key {printable: 'v', .. }, true) => {
            for x in 0..MAP_WIDTH {
                for y in 0..MAP_HEIGHT {
                    map.0[x as usize][y as usize].explored = true;
                }
            }
            DidntTakeTurn
        }

        (_, _) => DidntTakeTurn,
    }
}

fn cast_heal(_inventory_id: usize, objects: &mut [Object], messages: &mut Messages) -> UseResult {
    if let Some(fighter) = objects[PLAYER].fighter {
        if fighter.hp == fighter.max_hp {
            messages.message("You are alrady at full health.", RED);
            return UseResult::Cancelled;
        }
        messages.message("Your wounds start to feel better!", LIGHT_VIOLET);
        objects[PLAYER].heal(HEAL_AMOUNT);
        return UseResult::UsedUp;
    }
    UseResult::Cancelled
}

fn menu<T: AsRef<str>>(header: &str, options: &[T], width: i32, root: &mut Root) -> Option<usize> {
    assert!(options.len() <= 26, "Cannot have a menu with more than 26 options");

    let header_height = root.get_height_rect(0, 0, width, SCREEN_HEIGHT, header);
    let height = options.len() as i32 + header_height;

    let mut window = Offscreen::new(width, height);

    window.set_default_foreground(WHITE);
    window.print_rect_ex(0, 0, width, height, BackgroundFlag::None, TextAlignment::Left, header);

    for (index, option_text) in options.iter().enumerate() {
        let menu_letter = (b'a' + index as u8) as char;
        let text = format!("({}) {}", menu_letter, option_text.as_ref());
        window.print_ex(0, header_height + index as i32,
                        BackgroundFlag::None, TextAlignment::Left, text);
    }

    let x = SCREEN_WIDTH / 2 - width / 2;
    let y = SCREEN_HEIGHT / 2 - height / 2;
    tcod::console::blit(&mut window, (0, 0), (width, height), root, (x, y), 1.0, 0.7);

    root.flush();
    let key = root.wait_for_keypress(true);

    if key.printable.is_alphabetic() {
        let index = key.printable.to_ascii_lowercase() as usize - 'a' as usize;
        if index < options.len() {
            Some(index)
        } else {
            None
        }
    } else {
        None
    }
}

fn inventory_menu(inventory: &[Object], header: &str, root: &mut Root) -> Option<usize> {
    let options = if inventory.len() == 0 {
        vec!["Inventory is empty.".into()]
    } else {
        inventory.iter().map(|item| { item.name.clone() }).collect()
    };

    let inventory_index = menu(header, &options, INVENTORY_WIDTH, root);

    if inventory.len() > 0 {
        inventory_index
    } else {
        None
    }
}

fn use_item(inventory_id: usize,
            inventory: &mut Vec<Object>,
            objects: &mut [Object],
            messages: &mut Messages) {
    use Item::*;

    if let Some(item) = inventory[inventory_id].item {
        let on_use = match item {
            Heal => cast_heal,
        };
        match on_use(inventory_id, objects, messages) {
            UseResult::UsedUp => {
                inventory.remove(inventory_id);
            }
            UseResult::Cancelled => {
                messages.message("Cancelled", WHITE);
            }
        }
    } else {
        messages.message(format!("The {} cannot be used.", inventory[inventory_id].name), WHITE);
    }
}

fn pick_item_up(object_id: usize,
                objects: &mut Vec<Object>,
                inventory: &mut Vec<Object>,
                messages: &mut Messages) {
    if inventory.len() >= 26 {
        messages.message(format!("Your inventory is full, cannot pick up {}", objects[object_id].name), RED);
    } else {
        let item = objects.swap_remove(object_id);
        messages.message(format!("You picked up a {}!", item.name), GREEN);
        inventory.push(item);
    }
}

fn player_move_or_attack(dx: i32, dy: i32, map: &Map, objects: &mut [Object], messages: &mut Messages) {
    let x = objects[PLAYER].x + dx;
    let y = objects[PLAYER].y + dy;

    let target_id = objects.iter().position(|object| {
        object.fighter.is_some() && object.pos() == (x, y)
    });

    match target_id {
        Some(target_id) => {
            let (player, target) = mut_two(PLAYER, target_id, objects);
             player.attack(target, messages);
        }

        None => {
            move_player_by(objects, map, dx, dy);
        }
    }
}

fn render_all(game: &mut Game,
              objects: &[Object],
              map: &mut Map,
              messages: &mut Messages,
              fov_recompute: bool,
              config: &Config) {
    if fov_recompute {
        let player = &objects[PLAYER];
        game.fov.compute_fov(player.x, player.y, TORCH_RADIOUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible = game.fov.is_in_fov(x, y);

            // Color based on TileType and visibility
            let color = match (map.0[x as usize][y as usize].tile_type, visible) {
                (TileType::Wall, true) => config.color_light_wall,
                (TileType::Wall, false) => config.color_dark_wall,

                (TileType::Empty, true) => config.color_light_ground,
                (TileType::Empty, false) => config.color_dark_ground,

                (TileType::Water, true) => config.color_light_water,
                (TileType::Water, false) => config.color_dark_water,

                (TileType::ShortWall, true) => config.color_light_wall,
                (TileType::ShortWall, false) => config.color_dark_wall,
            };

            let mut explored = map.0[x as usize][y as usize].explored;
            if visible {
                explored = true;
            }

            if explored {
                let tile_type = map.0[x as usize][y as usize].tile_type;
                match tile_type {
                    TileType::Empty | TileType::Water => {
                        game.console.set_char_background(x, y, color.color(), BackgroundFlag::Set);
                    }

                    TileType::ShortWall | TileType::Wall => {
                        if visible {
                            game.console.set_char_background(x, y, config.color_light_ground.color(), BackgroundFlag::Set);
                        } else {
                            game.console.set_char_background(x, y, config.color_dark_ground.color(), BackgroundFlag::Set);
                        }

                        let left = map[(x - 1, y)].tile_type == tile_type;
                        let right = map[(x + 1, y)].tile_type == tile_type;
                        let horiz = left || right;

                        let above = map[(x, y + 1)].tile_type == tile_type;
                        let below = map[(x, y - 1)].tile_type == tile_type;
                        let vert = above || below;

                        let chr;
                        if tile_type == TileType::Wall {
                            if horiz && vert {
                               chr = tcod::chars::SUBP_N;
                               game.console.set_char_background(x, y, color.color(), BackgroundFlag::Set);
                            } else if horiz {
                               chr = tcod::chars::SUBP_N;
                            } else if vert {
                               chr = tcod::chars::SUBP_E;
                            } else {
                               chr = tcod::chars::SUBP_E;
                            }
                        } else {
                            if horiz && vert {
                               chr = tcod::chars::CROSS
                            } else if horiz {
                               chr = tcod::chars::HLINE;
                            } else if vert {
                               chr = tcod::chars::VLINE;
                            } else {
                               chr = tcod::chars::VLINE;
                            }
                        };

                        game.console.set_default_foreground(color.color());
                        game.console.put_char(x, y, chr, BackgroundFlag::None);
                    }
                }
            }
            map.0[x as usize][y as usize].explored = explored;
        }
    }

    /* Draw objects */
    let mut to_draw: Vec<_> = objects.iter().filter(|o| game.fov.is_in_fov(o.x, o.y)).collect();
    to_draw.sort_by(|o1, o2| { o1.blocks.cmp(&o2.blocks) });

    for object in &to_draw {
        object.draw(&mut game.console);
    }

    game.panel.set_default_background(BLACK);
    game.panel.clear();

    let hp = objects[PLAYER].fighter.map_or(0, |f| f.hp);
    let max_hp = objects[PLAYER].fighter.map_or(0, |f| f.max_hp);
    render_bar(game, 1, 1, BAR_WIDTH, "HP", hp, max_hp, LIGHT_RED, DARK_RED);

    let mut y = MSG_HEIGHT as i32;
    for &(ref msg, color) in messages.0.iter().rev() {
        let msg_height = game.panel.get_height_rect(MSG_X, y, MSG_WIDTH, 0, msg);
        y -= msg_height;
        if y < 0 {
            break;
        }
        game.panel.set_default_foreground(color);
        game.panel.print_rect(MSG_X, y, MSG_WIDTH, 0, msg);
    }

    game.panel.set_default_foreground(LIGHT_GREY);
    game.panel.print_ex(1, 2, BackgroundFlag::None, TextAlignment::Left, format!("Turn Count: {}", game.turn_count));
    game.panel.print_ex(1, 3, BackgroundFlag::None, TextAlignment::Left, format!("{:?}", (objects[PLAYER].momentum.unwrap().0, objects[PLAYER].momentum.unwrap().1)));
    game.panel.print_ex(1, 3, BackgroundFlag::None, TextAlignment::Left, get_names_under_mouse(game.mouse, objects, &game.fov));

    /* print all special characters */
    //print_all_special_char(game);

    blit(&mut game.console, (0, 0), (SCREEN_WIDTH, SCREEN_HEIGHT), &mut game.root, (0, 0), 1.0, 1.0);

    blit(&mut game.panel, (0, 0), (SCREEN_WIDTH, SCREEN_HEIGHT), &mut game.root, (0, PANEL_Y), 1.0, 1.0);

}

// BLOCK3 solid block
// CHECKBOX_UNSET unfilled block
// CROSS cross
// DCROSS invert cross
// DHLINE invert horizontal line
// DTEEE,DTEEN,DTEES,DTEEW invert tees
// DNE,DNW,DSE,DSW corners
// DVLINE divider lines vertical
// TEEE,TEEN,TEES,TEEW tees
// RADIO_SET,RADIO_UNSET unfilled circle and circle with inner filled circle
// SUBP_DIAG,SUBP_E, SUBP_N,SUBP_NE,SUBP_NW,SUBP_SE,SUBP_SW half blocks, directional. maybe
// SW as well
// VLINE HLINE thin lines
// ARROW2_E,ARROW2_N,ARROW2_S,ARROW2_W solid arrow
// ARROW_E,ARROW_N,ARROW_S,ARROW_W  thin arrow
fn print_all_special_char(game: &mut Game) {
    use tcod::chars::*;
    let keys = vec!(ARROW2_E,ARROW2_N,ARROW2_S,ARROW2_W,ARROW_E,ARROW_N,ARROW_S,
                    ARROW_W,BLOCK1,BLOCK2,BLOCK3,BULLET,BULLET_INV,BULLET_SQUARE,
                    CENT,CHECKBOX_SET,CHECKBOX_UNSET,CLUB,COPYRIGHT,CROSS,CURRENCY,
                    DARROW_H,DARROW_V,DCROSS,DHLINE,DIAMOND,DIVISION,DNE,DNW,DSE,DSW,
                    DTEEE,DTEEN,DTEES,DTEEW,DVLINE,EXCLAM_DOUBLE,FEMALE,FUNCTION,
                    GRADE,HALF,HEART,HLINE,LIGHT,MALE,MULTIPLICATION,NE,NOTE,
                    NOTE_DOUBLE,NW,ONE_QUARTER,PILCROW,POUND,POW1,POW2,
                    POW3,RADIO_SET,RADIO_UNSET,RESERVED,SE,SECTION,
                    SMILIE,SMILIE_INV,SPADE,SUBP_DIAG,SUBP_E, SUBP_N,SUBP_NE,SUBP_NW,SUBP_SE,SUBP_SW
                    ,SW,TEEE,TEEN,TEES,TEEW,THREE_QUARTERS,UMLAUT,VLINE,YEN);
    let mut index = 0;

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            game.console.set_char_background(x, y, BLACK, BackgroundFlag::Set);
            game.console.put_char(x, y, ' ', BackgroundFlag::None);
        }
    }

    game.console.set_default_foreground(WHITE);
    for key in 0..256 {
        let index_x = 10 + (index % 32);
        let index_y = -10 + ((index / 32) * 2);

        let x = SCREEN_WIDTH/2 + index_x - 32 as i32;
        let y = SCREEN_HEIGHT/2 + index_y;

        game.console.put_char(x,
                              y,
                              key as u8 as char,
                              BackgroundFlag::None);
        if game.mouse.cx as i32 == x && game.mouse.cy as i32 == y {
            game.console.print_ex(x,
                                  y - 1,
                                  BackgroundFlag::None,
                                  TextAlignment::Left,
                                  format!("{}", key));
        }

        index += 1;
    }
}


fn get_names_under_mouse(mouse: Mouse, objects: &[Object], fov_map: &FovMap) -> String {
    let (x, y) = (mouse.cx as i32, mouse.cy as i32);

    let names = objects.iter()
                       .filter(|obj| { obj.pos() == (x, y) && fov_map.is_in_fov(obj.x, obj.y)})
                       .map(|obj| { format!("{}, Ai {:?}, Behavior {:?}", obj.name.clone(), obj.ai, obj.behavior) })
                       .collect::<Vec<_>>();

    names.join(", ")
}

fn render_bar(game: &mut Game,
              x: i32,
              y: i32,
              total_width: i32,
              name: &str,
              value: i32,
              maximum: i32,
              bar_color: Color,
              back_color: Color) {
    let bar_width = (value as f32 / maximum as f32 * total_width as f32) as i32;

    game.panel.set_default_background(back_color);
    game.panel.rect(x, y, total_width, 1, false, BackgroundFlag::Screen);

    game.panel.set_default_background(bar_color);
    if bar_width > 0 {
        game.panel.rect(x, y, bar_width, 1, false, BackgroundFlag::Screen);
    }

    game.panel.set_default_foreground(WHITE);
    game.panel.print_ex(x + total_width / 2,
                   y,
                   BackgroundFlag::None,
                   TextAlignment::Center,
                   &format!("{}: {}/{}", name, value, maximum));
}

pub fn make_player() -> Object {
    let mut player = Object::new(0, 0, '@', "player", WHITE, true);
    player.alive = true;
    player.fighter = Some(Fighter{max_hp: 50, hp: 50, defense: 2, power: 5, on_death: DeathCallback::Player });
    player.momentum = Some((0, 0));

    player
}

fn main() {
    let mut previous_player_position = (-1, -1);

    let mut messages = Messages::new();

    let mut inventory = vec![];

    let mut config: Config;
    {
        let mut file = File::open("config.json").expect("Could not open/parse config file config.json");
        let mut config_string = String::new();
        file.read_to_string(&mut config_string).unwrap();
        config = serde_json::from_str(&config_string).unwrap();
    }

    let mut player = make_player();

    let mut objects = vec!(player);

    let (mut map, position) = make_map(&mut objects, &config);
    let player_x = position.0;
    let player_y = position.1;
    objects[PLAYER].x = player_x;
    objects[PLAYER].y = player_y;

    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Rogue-like")
        .init();

    let mut game = Game::with_root(root);

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            game.fov.set(x, y,
                         !map.0[x as usize][y as usize].block_sight,
                         !map.0[x as usize][y as usize].blocked);
        }
    }

    messages.message("Welcome Stranger! Prepare to perish in the Desolation of Salt!", ORANGE);

    let mut key = Default::default();

    while !game.root.window_closed() {
        match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
            Some((_, Event::Mouse(m))) => game.mouse = m,
            Some((_, Event::Key(k))) => key = k,
            _ => key = Default::default(),
        }

        let fov_recompute = previous_player_position != (objects[PLAYER].x, objects[PLAYER].y);
        render_all(&mut game, 
                   &objects, &mut map, &mut messages,
                   fov_recompute,
                   &config);

        game.root.flush();

        for object in &objects {
            object.clear(&mut game.console);
        }

        previous_player_position = (objects[PLAYER].x, objects[PLAYER].y);
        let player_action = handle_keys(&mut game, key, &mut map, &mut objects, &mut inventory, &mut messages);
        match player_action {
          PlayerAction::Exit => {
            break;
          }

          PlayerAction::TookTurn => {
              game.turn_count += 1;
          }
          
          _ => {}
        }

        if objects[PLAYER].alive && player_action != PlayerAction::DidntTakeTurn {
            for id in 1..objects.len() {
                if objects[id].ai.is_some() {
                    ai_take_turn(id, &map, &mut objects, &game.fov, &mut messages);
                }
            }
        }

        // reload configuration
        match File::open("config.json") {
            Ok(mut file) => {
                let mut config_string = String::new();
                file.read_to_string(&mut config_string).expect("Could not read config file!");
                config = serde_json::from_str(&config_string).expect("Could not read JSON- config.json has a parsing error!");
            }
          _ => (),
        }
    }
}

