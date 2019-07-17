#![allow(dead_code)]

extern crate tcod;
extern crate rand;
extern crate serde;
#[macro_use]extern crate serde_derive;
extern crate serde_json;
extern crate num;
extern crate timer;
extern crate chrono;
extern crate rodio;

mod engine;
mod constants;
mod input;
mod game;
#[cfg(test)]
mod tests;


#[allow(unused_imports)]use std::cmp;
#[allow(unused_imports)]use std::fs::File;
#[allow(unused_imports)]use std::io::BufReader;
#[allow(unused_imports)]use std::io::Read;
#[allow(unused_imports)]use std::sync::mpsc::channel;

#[allow(unused_imports)]use tcod::map::{Map as FovMap};
#[allow(unused_imports)]use tcod::console::*;
#[allow(unused_imports)]use tcod::colors::*;
#[allow(unused_imports)]use tcod::input::Key;
#[allow(unused_imports)]use tcod::input::KeyCode::*;
#[allow(unused_imports)]use tcod::input::{Event, Mouse};
#[allow(unused_imports)]use tcod::AsNative;
#[allow(unused_imports)]use tcod::image;

use timer::*;

use rodio::Source;

use engine::types::*;
use constants::*;
use engine::display::*;
use engine::map::*;
use engine::ai::*;
use input::*;
use game::*;


pub fn setup_fov(fov: &mut FovMap, map: &Map) {
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            fov.set(x,
                    y,
                    !map.0[x as usize][y as usize].block_sight,
                    !map.0[x as usize][y as usize].blocked);
        }
    }
}

fn step_animation(objects: &mut [Object], map: &Map, animation: &mut Animation) -> bool {
    match animation {
        Animation::Thrown(obj_id, line) => {
            match line.step() {
                Some(next) => {
                    if map.0[next.0 as usize][next.1 as usize].block_sight {
                        true
                    } else {
                        objects[*obj_id].x = next.0;
                        objects[*obj_id].y = next.1;
                        false
                    }
                },

                None => {
                    true
                },
            }
        }
    }
}

/// Check whether the exit condition for the game is met.
fn exit_condition_met(inventory: &[Object], map: &Map, objects: &[Object]) -> bool {
    // loop over objects in inventory, and check whether any
    // are the goal object.
    let has_goal =
        inventory.iter().any(|obj| obj.item.map_or(false, |item| item == Item::Goal));

    let player_pos = (objects[PLAYER].x, objects[PLAYER].y);
    let on_exit_tile = map[player_pos].tile_type == TileType::Exit;

    let exit_condition = has_goal && on_exit_tile;

    return exit_condition;
}

/// Play a sound file.
/// This implementation is inefficient, but simple.
pub fn play_sound(file_name: &str) {
    let device = rodio::default_output_device().unwrap();
    let file = File::open(file_name).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    rodio::play_raw(&device, source.convert_samples());
}

pub fn run_game<F>(mut step: F)
  where F: FnMut() -> bool {
    // start game tick timer
    let timer = Timer::new();
    let (tick_sender, tick_receiver) = channel();
    let _guard = 
        timer.schedule_repeating(chrono::Duration::milliseconds(TIME_BETWEEN_FRAMES_MS), move || {
            tick_sender.send(0).unwrap();
        });

    /* main game loop */
    let mut running = true;
    while running {
        /* fps limiting */
        tick_receiver.recv().unwrap();

        running = step();
    }
}

pub fn step_game(game: &mut Game,
                 config: &mut Config,
                 previous_player_position: &mut (i32, i32),
                 map: &mut Map,
                 objects: &mut Vec<Object>,
                 messages: &mut Messages,
                 inventory: &mut Vec<Object>) -> bool {
    /* Handle Inputs */
    let mut key = Default::default();

    match tcod::input::check_for_event(tcod::input::MOUSE | tcod::input::KEY_PRESS) {
        Some((_, Event::Mouse(m))) => game.mouse = m,
        Some((_, Event::Key(k))) => key = k,
        _ => {
            key = Default::default();
            game.mouse.lbutton_pressed = false;
            game.mouse.rbutton_pressed = false;
        },
    }

    /* Display */
    let fov_recompute = *previous_player_position != (objects[PLAYER].x, objects[PLAYER].y);
    render_all(game, 
               objects,
               map,
               messages,
               fov_recompute,
               config);

    game.root.flush();

    for object in objects.iter() {
        object.clear(&mut game.console);
    }

    for clearable in game.needs_clear.iter() {
        game.console.put_char(clearable.0, clearable.1, ' ', BackgroundFlag::None);
    }
    game.needs_clear.clear();

    /* Player Action and Animations */
    // If there is an animation playing, let it finish
    let player_action;
    if game.animations.len() > 0 {
        let mut finished_ixs = Vec::new();
        let mut ix = 0; 
        for mut animation in game.animations.iter_mut() {
          let finished = step_animation(objects, map, &mut animation);
          if finished {
              finished_ixs.push(ix)
          }
          ix += 1;
        }
        finished_ixs.sort_unstable();
        for ix in finished_ixs.iter().rev() {
            game.animations.swap_remove(*ix);
        }
        player_action = PlayerAction::DidntTakeTurn;
    } else {
        *previous_player_position = (objects[PLAYER].x, objects[PLAYER].y);
        player_action = handle_input(game, key, map, objects, inventory, config, messages);
        match player_action {
          PlayerAction::Exit => {
            return false;
          }

          PlayerAction::TookTurn => {
              game.turn_count += 1;
          }
          
          _ => {}
        }
    }

    /* Check Exit Condition */
    if exit_condition_met(&inventory, map, objects) {
        std::process::exit(0);
    }

    /* AI */
    if objects[PLAYER].alive && player_action != PlayerAction::DidntTakeTurn {
        for id in 1..objects.len() {
            if objects[id].ai.is_some() {
                ai_take_turn(id, map, objects, &game.fov, &mut game.animations);
            }
        }
    }

    // reload configuration
    match File::open("config.json") {
        Ok(mut file) => {
            let mut config_string = String::new();
            file.read_to_string(&mut config_string).expect("Could not read config file!");
            *config = serde_json::from_str(&config_string).expect("Could not read JSON- config.json has a parsing error!");
        }
      _ => (),
    }

  return !game.root.window_closed();
}

fn main() {
    let mut previous_player_position = (-1, -1);

    let mut messages = Messages::new();

    let mut inventory = vec![Object::make_stone(0, 0)];

    let mut config: Config;
    {
        let mut file = File::open("config.json").expect("Could not open/parse config file config.json");
        let mut config_string = String::new();
        file.read_to_string(&mut config_string).expect("Could not read contents of config.json");
        config = serde_json::from_str(&config_string).expect("Could not parse config.json file!");
    }

    let mut objects = vec!(make_player());

    let (mut map, position) = make_map(&mut objects, &config);
    let player_x = position.0;
    let player_y = position.1;
    objects[PLAYER].x = player_x;
    objects[PLAYER].y = player_y;

    let root = Root::initializer()
        .font("rexpaint16x16.png", FontLayout::AsciiInRow)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Rogue-like")
        .init();

    let mut game = Game::with_root(root);

    setup_fov(&mut game.fov, &map);

    messages.message("Welcome Stranger! Prepare to perish in the Desolation of Salt!", ORANGE);

    /* main game loop */
    run_game(move || {
        step_game(&mut game,
                  &mut config,
                  &mut previous_player_position,
                  &mut map,
                  &mut objects,
                  &mut messages,
                  &mut inventory)
    });
}

