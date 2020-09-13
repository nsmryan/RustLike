use noise::NoiseFn;
use noise::Perlin;

use sdl2::render::{BlendMode, WindowCanvas};
use sdl2::rect::Rect;
use sdl2::pixels::{Color as Sdl2Color};

use roguelike_core::types::*;
use roguelike_core::map::*;
use roguelike_core::constants::*;
use roguelike_core::movement::*;
use roguelike_core::config::*;
use roguelike_core::animation::{Sprite, Effect, Animation, AnimKey};
use roguelike_core::utils::{line, item_primary_at, distance, move_towards, lerp_color};

use roguelike_engine::game::*;

use crate::display::*;
use crate::plat::*;


pub fn render_all(display: &mut Display, game: &mut Game)  -> Result<(), String> {
    let screen_rect = display.targets.canvas_panel.target.output_size()?;

    let plots = display.state
                    .screen_sections
                    .plot(0,
                          0,
                          screen_rect.0 as usize,
                          screen_rect.1 as usize);

    display.targets.canvas_panel.target.set_draw_color(Sdl2Color::RGB(0, 0, 0));
    display.targets.canvas_panel.target.clear();

    // TODO need to work out mouse position with new system
    let mut mouse_map_pos = None;
    //for zone in zones.iter() {
    //    if zone.name == "map" && zone.contains(game.mouse_state.x as usize, game.mouse_state.y as usize) {
    //        let ((_x_offset, _y_offset), scaler) =
    //            zone.fit(game.data.map.width() as usize * FONT_WIDTH as usize,
    //                     game.data.map.height() as usize * FONT_HEIGHT as usize);

    //        let mouse_map_xy = zone.within(game.mouse_state.x as usize, game.mouse_state.y as usize);
    //        let map_x = mouse_map_xy.0 as f32 / (FONT_WIDTH as f32 * scaler);
    //        let map_y = mouse_map_xy.1 as f32 / (FONT_HEIGHT as f32 * scaler);
    //        mouse_map_pos = Some(Pos::new(map_x as i32, map_y as i32));

    //        if let Some(mouse_id) = game.data.find_mouse() {
    //            game.data.entities.set_xy(mouse_id, map_x as i32, map_y as i32);
    //        }
    //    }
    //}

    /* Draw Background */
    render_background(display, game);

    /* Draw Map */
    let cell_dims = display.targets.map_panel.cell_dims();
    if game.settings.render_map {
        let canvas = &mut display.targets.canvas_panel.target;
        let display_state = &mut display.state;

        let background = &mut display.targets.background_panel.target;

        canvas.with_texture_canvas(&mut display.targets.map_panel.target, |canvas| {
            canvas.set_draw_color(Sdl2Color::RGB(0, 0, 0));
            canvas.clear();

            canvas.copy(background, None, None).unwrap();

            render_map(canvas, display_state, cell_dims, game);

            render_overlays(canvas, display_state, game, mouse_map_pos, cell_dims);

            render_entities(canvas, display_state, game, cell_dims);
    
            render_impressions(canvas, display_state, game, cell_dims);

            render_effects(canvas, display_state, game, cell_dims);
        }).unwrap();
    }

    /* Draw Player Info */
    let cell_dims = display.targets.player_panel.cell_dims();

    {
        let canvas = &mut display.targets.canvas_panel.target;
        let display_state = &mut display.state;

        let player_panel = &mut display.targets.player_panel;

        let panel = player_panel.with_target(());

        canvas.with_texture_canvas(&mut display.targets.player_panel.target, |canvas| {
            let mut panel = panel.with_target(canvas);
            render_player_info(&mut panel, display_state, game, cell_dims);
        }).unwrap();
    }

    // TODO add back in
    //render_inventory(canvas, display_state, game);
    // TODO add back in
    //render_info(canvas, display_state, game, mouse_map_pos);


    /* Paste Panels on Screen */
    let screen = display.targets.canvas_panel.area();
    let (map_area, info_area) = screen.split_right(display.targets.info_panel.cells.0 as usize);

    // TODO just make the map panel the right size in the first place
    // and re-create it when the map changes.
    let map_cell_dims = display.targets.map_panel.cell_dims();
   
    let (map_width, map_height) = game.data.map.size();
    let src = display.targets.map_panel.get_rect_cells(map_width as usize, map_height as usize);

    let dst = display.targets.canvas_panel.get_rect_within(&map_area);

    display.targets.canvas_panel.target.copy(&display.targets.map_panel.target, src, dst).unwrap();

    /* Draw Player Info Panel */
    let dst = display.targets.canvas_panel.get_rect_within(&info_area);

    display.targets.canvas_panel.target.copy(&display.targets.player_panel.target, None, dst).unwrap();

    if game.settings.state == GameState::Inventory {
        //let area = Area::new((SCREEN_WIDTH as i32 / 2) - (INVENTORY_WIDTH as i32 / 2),
        //                     (SCREEN_HEIGHT as i32 / 2) - (INVENTORY_HEIGHT as i32 / 2),
        //                     INVENTORY_WIDTH,
        //                     INVENTORY_HEIGHT,
        //                     FONT_WIDTH as usize,
        //                     FONT_HEIGHT as usize);

        // TODO add back in
        //render_inventory(canvas, display_state, game);
    } else if game.settings.state == GameState::SkillMenu {
        //let area = Area::new((SCREEN_WIDTH as i32 / 2) - (INVENTORY_WIDTH as i32 / 2),
        //                     (SCREEN_HEIGHT as i32 / 2) - (INVENTORY_HEIGHT as i32 / 2),
        //                     INVENTORY_WIDTH,
        //                     INVENTORY_HEIGHT,
        //                     FONT_WIDTH as usize,
        //                     FONT_HEIGHT as usize);

        // TODO add back in
        //render_skill_menu(canvas, display_state, game);
    } else if game.settings.state == GameState::ClassMenu {
        //let area = Area::new((SCREEN_WIDTH as i32 / 2) - (INVENTORY_WIDTH as i32 / 2),
        //                     (SCREEN_HEIGHT as i32 / 2) - (INVENTORY_HEIGHT as i32 / 2),
        //                     INVENTORY_WIDTH,
        //                     INVENTORY_HEIGHT,
        //                     FONT_WIDTH as usize,
        //                     FONT_HEIGHT as usize);

        // TODO add back in
        //render_class_menu(canvas, display_state, game);
    } else if game.settings.state == GameState::ConfirmQuit {
        //let area = Area::new((SCREEN_WIDTH as i32 / 2) - (INVENTORY_WIDTH as i32 / 2),
        //                     (SCREEN_HEIGHT as i32 / 2) - (INVENTORY_HEIGHT as i32 / 2),
        //                     INVENTORY_WIDTH,
        //                     INVENTORY_HEIGHT,
        //                     FONT_WIDTH as usize,
        //                     FONT_HEIGHT as usize);

        // TODO add back in
        //render_confirm_quit(canvas, display_state, game);
    }

    // TODO console
    //if game.settings.state == GameState::Console {
    //    render_console(display, game);
    //}

    Ok(())
}

/// Draw an outline and title around an area of the screen
fn render_placard(panel: &mut Panel<&mut WindowCanvas>,
                  display_state: &mut DisplayState,
                  text: &str,
                  config: &Config) {
    let color = config.color_mint_green;
    
    // Draw a black background
    panel.target.set_draw_color(Sdl2Color::RGBA(0, 0, 0, 255));
    panel.target.clear();
    let (width, height) = panel.target.output_size().unwrap();
    let (cell_width, cell_height) = panel.cell_dims();

    panel.target.set_draw_color(sdl2_color(color));

    // Draw a thin line around the edges of the placard
    panel.target.draw_rect(Rect::new(cell_width as i32 / 2,
                                     (cell_height as i32 / 2),
                                     width as u32 - (cell_width / 2),
                                     height as u32 - (cell_height / 2))).unwrap();

    // draw a rectangle around where the placard header text will be placed.
    let half_text = text.len() / 2;
    let text_offset = (width / 2) - (cell_width * half_text as u32);
    panel.target.fill_rect(Rect::new(text_offset as i32 - 3,
                                     0,
                                     (text.len() * cell_width as usize) as u32 + 2,
                                     cell_height as u32)).unwrap();

    // Draw header text
    let mid_char_offset = (width / cell_width) / 2;
    let text_start = (mid_char_offset - half_text as u32) as i32;

    let text_pos = Pos::new(text_start, 0);

    let sprite_key =
        display_state.lookup_spritekey("tiles")
                     .expect("Could not find rexpaint file in renderer!");
    let tile_sprite = &mut display_state.sprites[&sprite_key];

    tile_sprite.draw_text(panel.target, &text, text_pos, panel.cell_dims(), config.color_dark_blue);
}

fn render_pips(panel: &mut Panel<&mut WindowCanvas>,
               display_state: &mut DisplayState,
               num_pips: u32,
               y_pos: i32,
               color: Color) {
    /* TODO add back in
    if num_pips > 0 {
        let blend_mode = display.targets.canvas.blend_mode();
        display.targets.canvas.set_blend_mode(blend_mode);

        let color = sdl2_color(color);
        display.targets.canvas.set_draw_color(color);

        let start = area.char_rect(1, y_pos);

        let spacing = 4;
        let mut pips = Vec::new();
        for pip_index in 0..num_pips as i32 {
            let x_offset =  start.x + start.height() as i32 * pip_index + spacing * pip_index;
            let pip = Rect::new(x_offset,
                                start.y + spacing,
                                start.height(),
                                start.height());
            pips.push(pip)
        }
        display.targets.canvas.fill_rects(&pips).unwrap();

        display.targets.canvas.set_blend_mode(BlendMode::None);
    }
    */
}

fn render_bar(panel: &mut Panel<&mut WindowCanvas>,
              display_state: &mut DisplayState,
              percent: f32,
              y_pos: i32,
              fg_color: Color,
              bg_color: Color) {
    let blend_mode = panel.target.blend_mode();

    panel.target.set_blend_mode(BlendMode::None);
    let color = sdl2_color(fg_color);
    panel.target.set_draw_color(color);

    let cell_dims = panel.cell_dims();

    let width = panel.num_pixels.0 as u32  - 3 * cell_dims.0;

    let x = cell_dims.0 as i32 * 2;

    let health_rect = Rect::new(x,
                                cell_dims.1 as i32 * y_pos,
                                (width as f32 * percent) as u32,
                                cell_dims.1);
    panel.target.fill_rect(health_rect).unwrap();

    let full_rect = Rect::new(x,
                              cell_dims.1 as i32 * y_pos,
                              width,
                              cell_dims.1);
    let color = sdl2_color(bg_color);
    panel.target.set_draw_color(color);
    panel.target.draw_rect(full_rect).unwrap();

    panel.target.set_blend_mode(blend_mode);
}


// TODO console
/*
fn render_console(display: &mut Display, game: &mut Game) {
    let color = game.config.color_console;
    let color = sdl2_color(color);
    display.canvas.set_draw_color(color);

    let console_rect =
        Rect::new(0, (SCREEN_HEIGHT - game.console.height) as i32, SCREEN_WIDTH, SCREEN_HEIGHT / 2);
    display.canvas.fill_rect(console_rect).unwrap();

    let color = Sdl2Color::RGBA(255, 255, 255, 255);
    display.canvas.set_draw_color(color);

    let line_width = 1;

    let y_offset = (SCREEN_HEIGHT - game.console.height) as i32;

    let top_line_rect =
        Rect::new(0, y_offset, SCREEN_WIDTH, line_width);
    display.canvas.fill_rect(top_line_rect).unwrap();

    let bottom_line_rect =
        Rect::new(0, SCREEN_HEIGHT as i32 - line_width as i32, SCREEN_WIDTH, line_width);
    display.canvas.fill_rect(bottom_line_rect).unwrap();

    let left_line_rect =
        Rect::new(0, y_offset, line_width, game.console.height);
    display.canvas.fill_rect(left_line_rect).unwrap();

    let right_line_rect =
        Rect::new(SCREEN_WIDTH as i32 - line_width as i32, y_offset, line_width, game.console.height);
    display.canvas.fill_rect(right_line_rect).unwrap();

    let console_area = 
        Area::new(0, y_offset, SCREEN_WIDTH as usize, y_offset as usize, FONT_WIDTH as usize, FONT_HEIGHT as usize);

    display.draw_char('>',
                            Pos::new(0, 0),
                            Color::white(),
                            &console_area);
    display.draw_text(&game.console.input.clone(),
                            Pos::new(1, 0),
                            Color::white(),
                            &console_area);

    let mut y_pos = 1;
    for output in game.console.output.iter() {
        display.draw_text(&output.clone(),
                                Pos::new(0, y_pos),
                                Color::white(),
                                &console_area);
        y_pos += 1;
    }
}
*/

fn render_player_info(panel: &mut Panel<&mut WindowCanvas>, display_state: &mut DisplayState, game: &mut Game, cell_dims: (u32, u32)) {
    render_placard(panel,
                   display_state,
                   "Player",
                   &game.config);

    let player_id = game.data.find_player().unwrap();

    let mut list: Vec<String> = Vec::new();

    let color = game.config.color_soft_green;

    if let Some(fighter) = game.data.entities.fighter.get(&player_id) {
        let hp = if fighter.hp > 0 {
            fighter.hp
        } else {
            0
        };
        let health_percent = hp as f32 / fighter.max_hp as f32;

        render_bar(panel, display_state, health_percent, 2, game.config.color_red, Color::white());
    }

    let energy = game.data.entities.energy[&player_id];
    render_pips(&mut panel, display_state, energy, 3, game.config.color_light_green);

    /* TODO add back in

    list.push(format!("position:"));

    list.push(format!(" ({}, {})", 
                      game.data.entities.pos[&player_id].x,
                      game.data.entities.pos[&player_id].y));

    list.push(format!(""));

    let move_mode = game.data.entities.move_mode[&player_id];
    list.push(format!("{}", move_mode.to_string()));

    list.push(format!(""));
    list.push(format!("turn {}", game.settings.turn_count));

    let text_pos = Pos::new(1, 5);
    tile_sprite.draw_text_list(canvas, 
                               &list,
                               text_pos,
                               color);

    */
}

fn render_info(canvas: &mut WindowCanvas,
               display_state: &mut DisplayState,
               game: &mut Game,
               mouse_xy: Option<Pos>) {
    /* TODO add back in
    render_placard(canvas,
                   "Info",
                   &game.config);

    if let Some(mouse) = mouse_xy {
        let color = game.config.color_soft_green;

        let player_id = game.data.find_player().unwrap();
        let player_pos = game.data.entities.pos[&player_id];

        let object_ids =
            get_entity_under_mouse(mouse, &mut game.data, &game.config);

        let mut y_pos = 1;

        let mut text_list = Vec::new();

        text_list.push(format!("({:>2},{:>2})", mouse.x, mouse.y));

        let text_pos = Pos::new(1, y_pos);
        display.draw_text_list(&text_list,
                                     text_pos,
                                     color);
        text_list.clear();

        y_pos += 1;

        // only display first object
        if let Some(obj_id) = object_ids.first() {
            let pos = game.data.entities.pos[obj_id];

            // only display things in the player's FOV
            if game.settings.god_mode ||
               game.data.map.is_in_fov(player_pos, pos, game.config.fov_radius_player) {
                if let Some(fighter) = game.data.entities.fighter.get(obj_id) {
                    y_pos += 1;

                    let health_percent = fighter.hp as f32 / fighter.max_hp as f32;

                    render_bar(display,
                               health_percent,
                               y_pos,
                               game.config.color_red,
                               Color::white());

                    y_pos += 2;
                }

                text_list.push(format!("{:?}", game.data.entities.name[obj_id]));

                text_list.push(format!(""));

                if game.data.entities.fighter.get_mut(obj_id).map_or(false, |fighter| fighter.hp <= 0) {
                    text_list.push(format!("{}", "dead"));
                } else if let Some(behave) = game.data.entities.behavior.get(obj_id) {
                    text_list.push(format!("{}", behave.description()));
                }
            }
        }

        let text_pos = Pos::new(1, y_pos);
        display.draw_text_list(&text_list,
                                     text_pos,
                                     color);
        text_list.clear();

        y_pos = 10;
        let text_pos = Pos::new(1, y_pos);
        text_list.push(format!("Tile is"));
        text_list.push(format!("{:?}",  game.data.map[mouse].surface));

        if game.data.map[mouse].blocked {
            text_list.push(format!("blocked"));
        }

        display.draw_text_list(&text_list,
                                     text_pos,
                                     color);
    }
    */
}

fn render_skill_menu(canvas: &mut WindowCanvas, display_state: &mut DisplayState, game: &mut Game) {
    /* TODO add back in
    let player_id = game.data.find_player().unwrap();

    // Render header
    render_placard(canvas,
                  "Skills",
                  &game.config);

    let mut list = Vec::new();

    for (index, skill) in game.data.entities.skills[&player_id].iter().enumerate() {
        list.push(format!("{} {:?}", index, skill));
    }

    let y_pos = 2;
    let text_pos = Pos::new(2, y_pos);
    let color = game.config.color_light_grey;

    display.draw_text_list(&list,
                                 text_pos,
                                 color);
    */
}

fn render_class_menu(canvas: &mut WindowCanvas, display_state: &mut DisplayState, game: &mut Game) {
    /* TODO add back in
    // Render header
    render_placard(canvas,
                  "Choice Class",
                  &game.config);

    let mut list = Vec::new();

    for (index, class) in EntityClass::classes().iter().enumerate() {
        list.push(format!("{} {:?}", index, class));
    }

    let y_pos = 2;
    let text_pos = Pos::new(2, y_pos);
    let color = game.config.color_light_grey;

    display.draw_text_list(&list,
                           text_pos,
                           color);
    */
}

fn render_confirm_quit(canvas: &mut WindowCanvas, display_state: &mut DisplayState, game: &mut Game) {
    /* TODO add back in
    // Render header
    render_placard(canvas,
                   "Quit?",
                   &game.config);

    let mut list = Vec::new();

    list.push("q: quit game".to_string());
    list.push("".to_string());
    list.push("esc: continue".to_string());

    let y_pos = 2;
    let text_pos = Pos::new(2, y_pos);
    let color = game.config.color_light_grey;

    display.draw_text_list(&list,
                                 text_pos,
                                 color);
    */
}

/// Render an inventory section within the given area
fn render_inventory(canvas: &mut WindowCanvas, display_state: &mut DisplayState, game: &mut Game) {
    // Render header
    /* TODO add back in
    render_placard(canvas,
                 display_state,
                 "Inventory",
                 &game.config);

    let player_id = game.data.find_player().unwrap();

    // Render each object's name in inventory
    let mut y_pos = 2;
    let mut item_index = 0;
    let item_ids = game.data.entities.inventory[&player_id].clone();

    for (index, obj_id) in item_ids.iter().enumerate() {
        let color;
        match &game.data.entities.item.get(obj_id) {
            Some(Item::Stone) => {
                color = game.config.color_light_grey;
            }

            Some(Item::Goal) => {
                color = game.config.color_red;
            }
            
            _ => {
                color = game.config.color_mint_green;
            }
        }

        // place prompt character
        // TODO need font image
        //display.draw_char(('0' as u8 + item_index) as char,
        //                        Pos::new(1, y_pos),
        //                        game.config.color_ice_blue,
        //                        area);

        // place object name
        let text_pos = Pos::new(2, y_pos);
        let item_marker =
            if index == 0 && item_primary_at(player_id, &mut game.data.entities, 0) {
                "<"
            } else {
                ""
            };
        let item_text = format!(" {:?} {}", game.data.entities.name[obj_id], item_marker);
        display.draw_text(&item_text,
                                text_pos,
                                color);
        
        y_pos += 1;

        item_index += 1;
    }

    if game.data.entities.inventory[&player_id].len() == 0 {
        let text_pos = Pos::new(1, y_pos);
        display.draw_text(&format!("empty"),
                                text_pos,
                                game.config.color_ice_blue);
    }
    */
}

/// render the background files, including water tiles
fn render_background(display: &mut Display, game: &mut Game) {
    if !display.targets.background_panel.dirty {
        return;
    }
    display.targets.background_panel.dirty = false;

    let player_id = game.data.find_player().unwrap();
    let pos = game.data.entities.pos[&player_id];

    let (map_width, map_height) = game.data.map.size();

    let sprite_key =
        display.state.lookup_spritekey("tiles")
                     .expect("Could not find rexpaint file in renderer!");
    let sprite = &mut display.state.sprites[&sprite_key];

    let canvas = &mut display.targets.canvas_panel.target;

    let cell_dims = display.targets.background_panel.cell_dims();

    canvas.with_texture_canvas(&mut display.targets.background_panel.target, |canvas| {
        canvas.set_draw_color(Sdl2Color::RGB(0, 0, 0));
        canvas.clear();

        for y in 0..map_height {
            for x in 0..map_width {
                let map_pos = Pos::new(x, y);

                let visible =
                    game.data.map.is_in_fov(pos, map_pos, game.config.fov_radius_player) ||
                    game.settings.god_mode;

                sprite.draw_char(canvas,
                                 MAP_EMPTY_CHAR as char,
                                 map_pos,
                                 cell_dims,
                                 empty_tile_color(&game.config, map_pos, visible));

                let tile = &game.data.map[(x, y)];
                if tile.tile_type == TileType::Water {
                    let color = tile_color(&game.config, x, y, tile, visible);
                    let chr = tile.chr;
                    sprite.draw_char(canvas, chr as char, map_pos, cell_dims, color);
                }
            }
        }
    }).unwrap();
}

/// Render the map, with environment and walls
fn render_map(canvas: &mut WindowCanvas, display_state: &mut DisplayState, cell_dims: (u32, u32), game: &mut Game) {
    let player_id = game.data.find_player().unwrap();
    let player_pos = game.data.entities.pos[&player_id];

    let (map_width, map_height) = game.data.map.size();

    let sprite_key =
        display_state.lookup_spritekey("tiles")
                     .expect("Could not find rexpaint file in renderer!");
    let sprite = &mut display_state.sprites[&sprite_key];

    for y in 0..map_height {
        for x in 0..map_width {
            let pos = Pos::new(x, y);

            // draw an outline around the tile
            {
                let mut outline_color = Color::white();
                outline_color.a /= 8;
                draw_outline_tile(canvas, pos, cell_dims, outline_color);
            }

            // Render game stuff
            let visible =
                game.data.map.is_in_fov(player_pos, pos, game.config.fov_radius_player) ||
                game.settings.god_mode;

            game.data.map[pos].explored |= visible;

            let explored = game.data.map[pos].explored || visible;

            let tile = &game.data.map[pos];

            let wall_color =
                if explored {
                    game.config.color_light_brown
                } else {
                    game.config.color_dark_brown
                };

            let chr = tile.chr;

            // draw empty tile first, in case there is transparency in the character
            // draw_char(display, MAP_EMPTY_CHAR as char, x, y, empty_tile_color(config, x, y, visible));

            // if the tile is not empty or water, draw it
            let color = tile_color(&game.config, x, y, tile, visible);
            if chr != MAP_EMPTY_CHAR && tile.tile_type != TileType::Water {
                sprite.draw_char(canvas, chr as char, pos, cell_dims, color);
            }

            match tile.surface {
                Surface::Rubble => {
                    sprite.draw_char(canvas, MAP_RUBBLE as char, pos, cell_dims, color);
                }

                Surface::Grass => {
                    sprite.draw_char(canvas, MAP_RUBBLE as char, pos, cell_dims, game.config.color_light_green);
                }

                Surface::Floor => {
                    // Nothing to draw
                }
            }

            // finally, draw the between-tile walls appropriate to this tile
            if tile.bottom_wall == Wall::ShortWall {
                sprite.draw_char(canvas, MAP_THIN_WALL_BOTTOM as char, pos, cell_dims, wall_color);
            } else if tile.bottom_wall == Wall::TallWall {
                sprite.draw_char(canvas, MAP_THICK_WALL_BOTTOM as char, pos, cell_dims, wall_color);
            }

            if tile.left_wall == Wall::ShortWall {
                sprite.draw_char(canvas, MAP_THIN_WALL_LEFT as char, pos, cell_dims, wall_color);

            } else if tile.left_wall == Wall::TallWall {
                sprite.draw_char(canvas, MAP_THICK_WALL_LEFT as char, pos, cell_dims, wall_color);
            }

            if x + 1 < map_width {
                let right_pos = Pos::new(pos.x + 1, pos.y);
                let right_tile = &game.data.map[right_pos];
                if right_tile.left_wall == Wall::ShortWall {
                sprite.draw_char(canvas, MAP_THIN_WALL_RIGHT as char, pos, cell_dims, wall_color);
                } else if right_tile.left_wall == Wall::TallWall {
                sprite.draw_char(canvas, MAP_THICK_WALL_RIGHT as char, pos, cell_dims, wall_color);
                }
            }

            if y - 1 >= 0 {
                let up_pos = Pos::new(pos.x, pos.y - 1);
                let up_tile = &game.data.map[up_pos];
                if up_tile.bottom_wall == Wall::ShortWall {
                    sprite.draw_char(canvas, MAP_THIN_WALL_TOP as char, pos, cell_dims, wall_color);
                } else if up_tile.bottom_wall == Wall::TallWall {
                    sprite.draw_char(canvas, MAP_THICK_WALL_TOP as char, pos, cell_dims, wall_color);
                }
            }

            // Draw a square around this tile to help distinguish it visually in the grid
            let alpha;
            if visible && game.data.map[pos].tile_type != TileType::Water {
                if game.settings.overlay {
                    alpha = game.config.grid_alpha_overlay;
                } else {
                    alpha = game.config.grid_alpha_visible;
                }
            } else {
                alpha = game.config.grid_alpha;
            }
            if game.config.fog_of_war && !visible {
                let mut blackout_color = Color::black();
                if game.data.map[pos].explored {
                    blackout_color.a = game.config.explored_alpha
                }
                
                sprite.draw_char(canvas, MAP_EMPTY_CHAR as char, pos, cell_dims, blackout_color);
            }
        }
    }
}

/// Render each effect currently playing in the game
/// The strategy here is to copy the effects vector, update all items,
/// and then remove finished effects from back to front. The
/// resulting vector of effects is then saved as the new effects vector.
fn render_effects(canvas: &mut WindowCanvas,
                  display_state: &mut DisplayState,
                  game: &mut Game,
                  cell_dims: (u32, u32)) {
    let mut remove_indices = Vec::new();

    let mut effects = display_state.effects.clone();

    let sprite_key =
        display_state.lookup_spritekey("tiles")
                     .expect("Could not find rexpaint file in renderer!");
    let sprite = &mut display_state.sprites[&sprite_key];

    for (index, effect) in effects.iter_mut().enumerate() {
        match effect {
            Effect::HeardSomething(pos, created_turn) => {
                sprite.draw_char(canvas,
                                 ENTITY_ELF as char,
                                 *pos,
                                 cell_dims,
                                 game.config.color_warm_grey);

                if *created_turn != game.settings.turn_count {
                    remove_indices.push(index);
                }
            }

            Effect::Sound(sound_aoe, sound_dt) => {
                let mut highlight_color = game.config.color_warm_grey;

                let radius = sound_aoe.positions.len();
                let sound_interval = game.config.sound_timeout / radius as f32;
                let cur_dist = *sound_dt / sound_interval;
                for (dist, dist_positions) in sound_aoe.positions.iter().enumerate() {
                    highlight_color.a =
                        game.config.sound_alpha / ((dist as i16 - cur_dist as i16).abs() as u8 + 1);

                    // NOTE(perf) with the new texture system, consider rendering a series of sound
                    //            tiles and simply pasting them, perhaps saving time from not
                    //            needing to blend.
                    for pos in dist_positions.iter() {
                        if !game.data.map[*pos].blocked {
                           draw_tile_highlight(canvas, cell_dims, *pos, highlight_color);
                        }
                    }
                }

                if *sound_dt >= game.config.sound_timeout {
                    remove_indices.push(index);
                } else {
                    *sound_dt += 1.0 / game.config.rate as f32;
                    if *sound_dt > game.config.sound_timeout {
                        *sound_dt = game.config.sound_timeout;
                    }
                }
            }
        }
    }

    remove_indices.sort();
    remove_indices.reverse();
    for index in remove_indices {
        effects.swap_remove(index);
    }

    display_state.effects = effects;
}

fn render_entity(canvas: &mut WindowCanvas,
                 entity_id: EntityId,
                 display_state: &mut DisplayState,
                 game: &mut Game,
                 cell_dims: (u32, u32)) -> Option<Sprite> {
    let mut animation_result = AnimationResult::new();

    let pos = game.data.entities.pos[&entity_id];
    let player_id = game.data.find_player().unwrap();
    let player_pos = game.data.entities.pos[&player_id];

    // only draw if within the map (outside is (-1, -1) like if in inventory)
    // and not in limbo.
    if game.data.map.is_within_bounds(pos) &&
       game.data.entities.limbo.get(&entity_id).is_none() {
        let is_in_fov = 
           game.data.map.is_in_fov(player_pos, pos, game.config.fov_radius_player) ||
           game.settings.god_mode;

        if let Some(anim_key) = game.data.entities.animation[&entity_id].get(0) {
            animation_result = 
                render_animation(*anim_key,
                                 entity_id,
                                 is_in_fov,
                                 canvas,
                                 display_state,
                                 &mut game.data,
                                 &game.settings,
                                 &game.config,
                                 cell_dims);

            if animation_result.done {
                game.data.entities.animation[&entity_id].pop_front();
            }
        } else {
            // NOTE the needs_removal can probably be removed
            let needs_removal = game.data.entities.needs_removal[&entity_id];
            if is_in_fov && !needs_removal {
                let color = game.data.entities.color[&entity_id];

                let chr = game.data.entities.chr[&entity_id];
                let sprite = Sprite::char(chr);
                display_state.draw_sprite(canvas, sprite, pos, color, cell_dims);
                animation_result.sprite = Some(sprite);
            }
        }
    }

    return animation_result.sprite;
}

fn render_impressions(canvas: &mut WindowCanvas, display_state: &mut DisplayState, game: &mut Game, cell_dims: (u32, u32)) {
    // check for entities that have left FOV and make an impression for them
    for impression in display_state.impressions.clone() {
        display_state.draw_sprite(canvas, impression.sprite, impression.pos, game.config.color_light_grey, cell_dims);
    }
}

/// Render each object in the game, filtering for objects not currently visible
fn render_entities(canvas: &mut WindowCanvas, display_state: &mut DisplayState, game: &mut Game, cell_dims: (u32, u32)) {
    let player_id = game.data.find_player().unwrap();

    display_state.drawn_sprites.clear();

    // step each objects animation
    for entity in game.data.entities.ids.clone() {
        if entity != player_id {
            let maybe_sprite = render_entity(canvas, entity, display_state, game, cell_dims);

            if let Some(sprite) = maybe_sprite {
                display_state.drawn_sprites.insert(entity, sprite);
            }
        }
    }

    let maybe_sprite = render_entity(canvas, player_id, display_state, game, cell_dims);
    if let Some(sprite) = maybe_sprite {
        display_state.drawn_sprites.insert(player_id, sprite);
    }
}

fn render_animation(anim_key: AnimKey,
                    entity_id: EntityId,
                    is_in_fov: bool,
                    canvas: &mut WindowCanvas,
                    display_state: &mut DisplayState,
                    data: &mut GameData,
                    settings: &GameSettings,
                    config: &Config,
                    cell_dims: (u32, u32)) -> AnimationResult {

    let pos = data.entities.pos[&entity_id];
    let mut color = data.entities.color[&entity_id];

    // TODO should also freeze animation or leave at first element to indicate disarmed trap
    if data.entities.armed.get(&entity_id) == Some(&false) {
        color = config.color_warm_grey;
    }

    let mut animation_result: AnimationResult = AnimationResult::new();
    match display_state.animations[&anim_key].clone() {
        Animation::Between(ref mut sprite_anim, start, end, ref mut dist, blocks_per_sec) => {
           if settings.god_mode || is_in_fov {
               *dist = *dist + (blocks_per_sec / config.rate as f32); 
               let num_blocks = *dist as usize;

               let draw_pos = move_towards(start, end, num_blocks);

               let sprite = sprite_anim.sprite();
               display_state.draw_sprite(canvas,
                                         sprite,
                                         draw_pos,
                                         color,
                                         cell_dims);
               animation_result.sprite = Some(sprite);

               sprite_anim.step();

               display_state.animations[&anim_key] =
                   Animation::Between(*sprite_anim, start, end, *dist, blocks_per_sec);

               animation_result.done = *dist >= distance(start, end) as f32;
           }
        }

        Animation::Loop(ref mut sprite_anim) => {
           if settings.god_mode || is_in_fov {
                let sprite = sprite_anim.sprite();
                let player_id = data.find_player().unwrap();
                display_state.draw_sprite(canvas,
                                          sprite,
                                          pos,
                                          color,
                                          cell_dims);
                animation_result.sprite = Some(sprite);

                sprite_anim.step();

                display_state.animations[&anim_key] =
                   Animation::Loop(*sprite_anim);

                // a looping animation never finishes
                animation_result.done = false;
            }
        }

        Animation::PlayEffect(effect) => {
            display_state.play_effect(effect);
            animation_result.done = true;

            // NOTE the sprite is not updated here- this may cause entity impressions to not work
            // in edge cases where an effect is playing.
        }

        Animation::Once(ref mut sprite_anim) => {
           if settings.god_mode || is_in_fov {
                let sprite = sprite_anim.sprite();
                display_state.draw_sprite(canvas,
                                          sprite,
                                          pos,
                                          color,
                                          cell_dims);
                animation_result.sprite = Some(sprite);

                let sprite_done = sprite_anim.step();

                display_state.animations[&anim_key] =
                   Animation::Once(*sprite_anim);

                animation_result.done = sprite_done;
            }
        }
    }

    return animation_result;
}

fn render_overlays(canvas: &mut WindowCanvas,
                   display_state: &mut DisplayState,
                   game: &mut Game,
                   map_mouse_pos: Option<Pos>,
                   cell_dims: (u32, u32)) {
    let player_id = game.data.find_player().unwrap();
    let player_pos = game.data.entities.pos[&player_id];

    let sprite_key =
        display_state.lookup_spritekey("tiles")
                     .expect("Could not find rexpaint file in renderer!");
    let tile_sprite = &mut display_state.sprites[&sprite_key];

    // render a grid of numbers if enabled
    if game.config.overlay_directions {
        let map_width = game.data.map.width();
        let map_height = game.data.map.height();
        for y in 0..map_height {
            for x in 0..map_width {
                let pos = Pos::new(x, y);
                let x_diff = x - player_pos.x;
                let y_diff = y - player_pos.y;

                if x_diff.abs() < 5 && y_diff.abs() < 5 {
                    let res: i8 = x_diff as i8 - y_diff as i8;
                    if res <= 0 {
                        // TODO need font image
                        //display.draw_char(MAP_GROUND as char, pos, game.config.color_light_green, area);
                    } else {
                        // TODO need font image
                        //display.draw_char(MAP_GROUND as char, pos, game.config.color_light_grey, area);
                    }
                    // TODO need font image
                    //display.draw_char(('0' as u8 + res.abs() as u8) as char, pos, game.config.color_red, area);
                }
            }
        }
    }

    // render FOV if enabled
    if game.config.overlay_player_fov {
        let map_width = game.data.map.width();
        let map_height = game.data.map.height();
        for y in 0..map_height {
            for x in 0..map_width {
                let pos = Pos::new(x, y);

                let dir = game.data.entities.direction[&player_id];
                let is_in_fov =
                    game.data.map.is_in_fov_direction(player_pos,
                                                      pos,
                                                      game.config.fov_radius_player,
                                                      dir);
                if is_in_fov {
                    // TODO need font image
                    //display.draw_char(MAP_GROUND as char, pos, game.config.color_light_green, area);
                }
            }
        }
    }

    // Draw player action overlay. Could draw arrows to indicate how to reach each location
    let mut highlight_color: Color = game.config.color_warm_grey;
    highlight_color.a = game.config.highlight_player_move;

    // draw direction overlays
    let mut direction_color = game.config.color_soft_green;
    direction_color.a /= 2;
    for entity_id in game.data.entities.ids.iter().map(|id| *id).collect::<Vec<EntityId>>().iter() {
        let pos = game.data.entities.pos[entity_id];

        if pos.x == -1 && pos.y == -1 {
            continue;
        }

        if game.data.map.is_in_fov(player_pos, pos, game.config.fov_radius_player) &&
           game.data.entities.alive[entity_id] {
            if let Some(dir) = game.data.entities.direction.get(entity_id) {
                // display.draw_tile_edge(pos, area, direction_color, dir);

                let rotation = match dir {
                    Direction::Up => -90.0,
                    Direction::Down => 90.0,
                    Direction::Right => 0.0,
                    Direction::Left => 180.0,
                    Direction::DownLeft => 135.0,
                    Direction::DownRight => 45.0,
                    Direction::UpLeft => -135.0,
                    Direction::UpRight => -45.0,
                };

                tile_sprite.draw_sprite_at_cell(canvas,
                                                ARROW_RIGHT as usize,
                                                pos,
                                                cell_dims,
                                                direction_color,
                                                rotation);
            }
        }
    }

    // draw attack and fov position highlights
    if let Some(mouse_xy) = map_mouse_pos {
        // Draw monster attack overlay
        let object_ids = get_entity_under_mouse(mouse_xy, &mut game.data, &game.config);
        for entity_id in object_ids.iter() {
            let pos = game.data.entities.pos[entity_id];

            if game.data.map.is_in_fov(player_pos, pos, game.config.fov_radius_player) &&
               game.data.entities.alive[entity_id] {
               render_attack_overlay(canvas, display_state, game, *entity_id, cell_dims);
               render_fov_overlay(canvas, display_state, game, *entity_id, cell_dims);
               render_movement_overlay(canvas, display_state, game, *entity_id, cell_dims);
            }
        }
    }

    // render attack overlay highlighting squares that an entity can attack
    if game.settings.overlay {
        let keys = game.data.entities.ids.iter().map(|id| *id).collect::<Vec<EntityId>>();
        for entity_id in keys {
            let pos = game.data.entities.pos[&entity_id];

            if game.data.map.is_within_bounds(pos) &&
               game.data.map.is_in_fov(player_pos, pos, game.config.fov_radius_player) &&
               game.data.entities.alive[&entity_id] {
               render_attack_overlay(canvas,
                                     display_state,
                                     game,
                                     entity_id,
                                     cell_dims);
            }
        }
    }

    // draw mouse path overlays
    if let Some(mouse_id) = game.data.find_mouse() {
        let mouse_pos = game.data.entities.pos[&mouse_id];
        let player_pos = game.data.entities.pos[&player_id];

        if game.config.draw_star_path {
            // get a path to the mouse path, regardless of distance
            let path = astar_path(&game.data.map, player_pos, mouse_pos, None, None);
            for pos in path {
                // TODO need font image
                //display.draw_char(MAP_EMPTY_CHAR as char, pos, highlight_color, area);
            }
        }

        if game.config.draw_mouse_line {
            // mouse pos at 0, 0 occurs when the mouse has not moved since startup.
            // this may cause a weirdness on the corner of the map
            if mouse_pos != Pos::new(0, 0) {
                let line = line(player_pos, mouse_pos).into_iter();
                for pos in line {
                    let pos = Pos::from(pos);
                    // TODO need font image
                    //display.draw_char(MAP_EMPTY_CHAR as char, pos, highlight_color, area);
                }
            }
        }

        if game.settings.draw_selection_overlay {
            // mouse pos at 0, 0 occurs when the mouse has not moved since startup.
            // this may cause a weirdness on the corner of the map
            if mouse_pos != Pos::new(0, 0) {
                let selected_pos =
                    game.settings.selection.selected_pos(player_pos, mouse_pos, game.config.fov_radius_player, &mut game.data);

                if let Some(pos) = selected_pos {
                    // TODO need font image
                    //display.draw_char(MAP_EMPTY_CHAR as char, pos, highlight_color, area);
                }
            }
        }
    }

    // Draw player movement overlay
    if game.settings.overlay {
        for move_action in Direction::move_actions().iter() {
            // for all movements except staying still
            // calculate the move that would occur
            if let Some(movement) =
                calculate_move(*move_action,
                               game.data.entities.movement[&player_id],
                               player_id,
                               &mut game.data) {
                // draw a highlight on that square
                // don't draw overlay on top of character
                if movement.pos != game.data.entities.pos[&player_id] {
                    dbg!();
                    draw_tile_highlight(canvas,
                                        cell_dims,
                                        movement.pos,
                                        highlight_color);
                }
            }
        }
    }

    if game.config.overlay_fov_alg {
        let highlight_color_fov = game.config.color_light_orange;
        let highlight_color_lines = game.config.color_red;

        for y in 0..game.data.map.height() {
            for x in 0..game.data.map.width() {
                let pos = Pos::new(x, y);
                let in_fov = game.data.map.is_in_fov(player_pos, pos, game.config.fov_radius_player);
                let in_fov_lines = game.data.map.is_in_fov_lines(player_pos, pos, game.config.fov_radius_player);

                if in_fov && !in_fov_lines {
                    // TODO add back in with rendering update
                    //display.draw_tile_outline(pos, area, highlight_color_fov);
                    draw_outline_tile(canvas, pos, cell_dims, highlight_color_fov);
                }

                if in_fov_lines && !in_fov {
                    // TODO add back in with rendering update
                    draw_outline_tile(canvas, pos, cell_dims, highlight_color_lines);
                }
            }
        }
    }

    // NOTE floodfill ranges:
    // 4-5 is the end of a cooridor
    // < 14 is pretty enclosed
    // 14-24 is within the corner of a building or at an opening from an enclosed space
    // low 30s tend to be next to a shear wall
    // 40 are nearly fully open
    // 49 may be fully open
    if game.config.overlay_floodfill {
        let highlight_color = game.config.color_light_orange;
        for y in 0..game.data.map.height() {
            for x in 0..game.data.map.width() {
                let pos = Pos::new(x, y);

                if !game.data.map[pos].blocked {
                    let near_count = game.data.map.floodfill(pos, 3).len();

                    let amount = near_count as f32 / 50.0;
                    let adj_color = lerp_color(game.config.color_ice_blue, game.config.color_red, amount);
                    // TODO need font image
                    //display.draw_char(MAP_EMPTY_CHAR as char, pos, adj_color, area);

                    draw_text_with_font(canvas,
                                        &mut display_state.font_map,
                                        &format!("{}", near_count),
                                        pos,
                                        highlight_color);
                }
            }
        }
    }
}

fn get_entity_under_mouse(mouse_pos: Pos,
                          data: &mut GameData,
                          config: &Config) -> Vec<EntityId> {
    let mut object_ids: Vec<EntityId> = Vec::new();

    for key in data.entities.ids.iter() {
        let pos = data.entities.pos[key];
        let is_mouse = data.entities.name[key] == EntityName::Mouse;
        let removing = data.entities.needs_removal[key];

        if !removing && !is_mouse && mouse_pos == pos {
            if data.map.is_in_fov(pos, mouse_pos, config.fov_radius_player) {
                object_ids.push(*key);
            }
        }
    }

    return object_ids;
}

fn empty_tile_color(config: &Config, pos: Pos, visible: bool) -> Color {
    let perlin = Perlin::new();

    let low_color;
    let high_color;
    if visible {
        low_color = config.color_tile_blue_light;
        high_color = config.color_tile_blue_dark;
    } else {
        low_color = config.color_tile_blue_dark;
        high_color = config.color_very_dark_blue;
    }
    let color =
        lerp_color(low_color,
                   high_color,
                   perlin.get([pos.x as f64 / config.tile_noise_scaler,
                               pos.y as f64 / config.tile_noise_scaler]) as f32);

   return color;
}

fn tile_color(config: &Config, _x: i32, _y: i32, tile: &Tile, visible: bool) -> Color {
    let color = match (tile.tile_type, visible) {
        (TileType::Wall, true) =>
            config.color_light_brown,
        (TileType::Wall, false) =>
            config.color_dark_brown,

        (TileType::Empty, true) =>
            config.color_light_brown,

        (TileType::Empty, false) =>
            config.color_dark_brown,

        (TileType::Water, true) =>
            config.color_blueish_grey,
        (TileType::Water, false) =>
            config.color_blueish_grey,

        (TileType::ShortWall, true) =>
            config.color_light_brown,
        (TileType::ShortWall, false) =>
            config.color_dark_brown,

        (TileType::Exit, true) =>
            config.color_orange,
        (TileType::Exit, false) =>
            config.color_red,
    };

    return color;
}

fn render_attack_overlay(canvas: &mut WindowCanvas,
                         display_state: &mut DisplayState,
                         game: &mut Game,
                         entity_id: EntityId,
                         cell_dims: (u32, u32)) {
    let player_id = game.data.find_player().unwrap();
    let player_pos = game.data.entities.pos[&player_id];

    let object_pos = game.data.entities.pos[&entity_id];

    let mut attack_highlight_color = game.config.color_red;
    attack_highlight_color.a = game.config.highlight_attack;

    // TODO consider the duplication with looking this up in rendering functions
    let sprite_key =
        display_state.lookup_spritekey("tiles")
                     .expect("Could not find rexpaint file in renderer!");
    let tile_sprite = &mut display_state.sprites[&sprite_key];

    if let Some(reach) = game.data.entities.attack.get(&entity_id) {
        let attack_positions = 
            reach.offsets()
                 .iter()
                 .map(|offset| Pos::new(object_pos.x as i32 + offset.x,
                                        object_pos.y as i32 + offset.y))
                 // filter out positions that are outside of the map, or with no clear
                 // path from the entity to the reached position
                 .filter(|pos| {
                     let in_bounds = game.data.map.is_within_bounds(*pos);
                     let traps_block = false;
                     let clear = game.data.clear_path(object_pos, *pos, traps_block);
                     let player_can_see = game.data.map.is_in_fov(player_pos, *pos, game.config.fov_radius_player);
                     // check for player position so it gets highligted, even
                     // though the player causes 'clear_path' to fail.
                     return player_can_see && in_bounds && (clear || *pos == player_pos);
                 })
                 .collect::<Vec<Pos>>();

        for position in attack_positions {
            tile_sprite.draw_char(canvas, MAP_EMPTY_CHAR as char, position, cell_dims, attack_highlight_color);
        }
    }
}

fn render_fov_overlay(canvas: &mut WindowCanvas,
                      display_state: &mut DisplayState,
                      game: &mut Game,
                      entity_id: EntityId,
                      cell_dims: (u32, u32)) {
    let player_id = game.data.find_player().unwrap();

    let mut highlight_color = game.config.color_light_grey;
    highlight_color.a = game.config.grid_alpha_overlay;

    for y in 0..game.data.map.height() {
        for x in 0..game.data.map.width() {
            let map_pos = Pos::new(x, y);

            let visible = game.data.is_in_fov(entity_id, map_pos, &game.config) &&
                          game.data.is_in_fov(player_id, map_pos, &game.config);


            if visible {
                let chr = game.data.entities.chr[&entity_id];
                draw_outline_tile(canvas, map_pos, cell_dims, highlight_color);
            }
        }
    }
}

fn render_movement_overlay(canvas: &mut WindowCanvas,
                           display_state: &mut DisplayState,
                           game: &mut Game,
                           entity_id: EntityId,
                           cell_dims: (u32, u32)) {
    let player_id = game.data.find_player().unwrap();

    let entity_pos = game.data.entities.pos[&entity_id];

    let mut highlight_color = game.config.color_light_grey;
    highlight_color.a = game.config.grid_alpha_overlay;

    let sprite_key =
        display_state.lookup_spritekey("tiles")
                     .expect("Could not find rexpaint file in renderer!");
    let tile_sprite = &mut display_state.sprites[&sprite_key];

    if let Some(reach) = game.data.entities.movement.get(&entity_id) {
        for move_pos in reach.reachables(entity_pos) {
            let visible = game.data.is_in_fov(player_id, move_pos, &game.config);
            if visible {
                let chr = game.data.entities.chr[&entity_id];

                tile_sprite.draw_char(canvas, chr as char, move_pos, cell_dims, highlight_color);
            }
        }
    }
}

pub fn sdl2_color(color: Color) -> Sdl2Color {
    return Sdl2Color::RGBA(color.r, color.g, color.b, color.a);
}

