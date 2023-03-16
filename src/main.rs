use std::{
    fs, 
    vec, 
    cmp
};
use itertools::Itertools;
use ggez::{
    event,
    GameResult,
    Context,
    glam::Vec2,
    graphics::{
        self,
        Canvas,
        DrawParam,
        InstanceArray,
        Color, Text
    }
};

const DESIRED_FPS: u32 = 120;

// Lower = faster drops. Probably shouldn't go lower than 2
const SAND_DROP_DELAY: usize = 3;

const SAND_COLOR: Color = Color::new(0.98, 0.98, 0.82, 1.0);
const ROCK_COLOR: [f32; 4] = [0.55, 0.27, 0.09, 1.0];
const START_COLOR: [f32; 4] = [1.0, 0.84, 0.0, 1.0];

// Draw param scaling to fit a grid square
const SAND_SCALE: [f32; 2] = [5.8, 5.8];
const ROCK_SCALE: [f32; 2] = [5.8, 5.8];

const GRID_SIZE: (i16, i16) = (320, 160);
const GRID_CELL_SIZE: (u16, u16) = (7, 7);

const WINDOW_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * GRID_CELL_SIZE.0 as f32,
    GRID_SIZE.1 as f32 * GRID_CELL_SIZE.1 as f32
);

const FPS_TEXT_POS: Vec2 = Vec2 {
    x: (GRID_SIZE.0 as f32 - 12.0) * GRID_CELL_SIZE.0 as f32,
    y: (GRID_SIZE.1 as f32 - 3.5) * GRID_CELL_SIZE.1 as f32
};

// Util consts for sand moving
const ONE_X_STEP: Vec2 = Vec2::new(1.0, 0.0);
const ONE_Y_STEP: Vec2 = Vec2::new(0.0, 1.0);

pub struct GameState {
    cave: Vec<Vec<char>>,
    sand: Vec<Vec2>,
    source: Vec2,
    rocks: InstanceArray,
    moving_sands: InstanceArray,
    settled_sands: InstanceArray
}

impl GameState {
    pub fn new(ctx: &mut Context, cave: Vec<Vec<char>>, source: Vec2) -> GameResult<GameState> {
        let rocks = get_rocks_and_start(ctx, &cave);
        let state = GameState { 
            cave, 
            sand: vec![], 
            source, 
            rocks, 
            moving_sands: InstanceArray::new(ctx, None),
            settled_sands: InstanceArray::new(ctx, None)
        };

        Ok(state)
    }
}

impl event::EventHandler<ggez::GameError> for GameState {

    fn update(&mut self, ctx: &mut Context) -> GameResult {

        if ctx.time.ticks() % SAND_DROP_DELAY == 0 {
            add_sand(self.source, &mut self.sand, &mut self.moving_sands);
        }

        while ctx.time.check_update_time(DESIRED_FPS) {
            move_all_sand_one_step(&mut self.cave, &mut self.sand, &mut self.moving_sands, &mut self.settled_sands);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(
            ctx,
            Color::new(0.03, 0.03, 0.07, 1.0)
        );

        let fps = format!("FPS: {:.0}", ctx.time.fps());
        canvas.draw(
            &Text::new(&fps), FPS_TEXT_POS
        );

        canvas.draw(&self.rocks, graphics::DrawParam::default());
        canvas.draw(&self.settled_sands, graphics::DrawParam::default());
        canvas.draw(&self.moving_sands, graphics::DrawParam::default());

        canvas.finish(ctx)?;

        ggez::timer::yield_now();
        Ok(())
    }
}

fn get_rocks_and_start(ctx: &Context, cave: &Vec<Vec<char>>) -> InstanceArray {
    let mut rocks = InstanceArray::new(ctx, None);

    rocks.set( 
        (0..cave.len()).flat_map(|i| {
            (0..cave[0].len()).filter_map(move |j| {
                let x = i as i32;
                let y = j as i32;
                if cave[i][j] == '#' {
                    Some(DrawParam::new()
                        .dest(get_dest(y, x))
                        .scale(ROCK_SCALE)
                        .color(ROCK_COLOR))
                } else if cave[i][j] == '+' {
                    Some(DrawParam::new()
                        .dest(get_dest(y, x))
                        .scale(ROCK_SCALE)
                        .color(START_COLOR))
                } else { None }
            })
        })
    );

    return rocks;
}

fn get_dest(x: i32, y: i32) -> Vec2 {
    return Vec2::new((x as i32 * GRID_CELL_SIZE.0 as i32) as f32, (y as i32 * GRID_CELL_SIZE.1 as i32) as f32);
}

fn floor_cave(cave: &mut Vec<Vec<char>>) -> &Vec<Vec<char>>{
    if let Some(lowest_y) = cave.iter().rposition(|row| row.contains(&'#')) {

        cave.drain((lowest_y + 2)..cave.len());

        let width = cave[0].len();
        let req_width = 2 * cave.len();
        if width < req_width {
            for _ in 0..((req_width - width + 2) / 2) {
                cave.into_iter().for_each( |row| {row.insert(0, '.'); row.push('.')} );
            }
        }

        cave.push(vec!['#'; cave[0].len()]);

        return cave;
    } else {
        panic!("No structures in cave!");
    }
}

fn add_sand(source: Vec2, sand: &mut Vec<Vec2>, moving_sands: &mut InstanceArray) {
    sand.push(source);
    moving_sands.push(DrawParam::new()
        .dest(get_dest(source.x as i32, source.y as i32))
        .color(SAND_COLOR)
        .scale(SAND_SCALE));
}

fn move_all_sand_one_step(cave: &mut Vec<Vec<char>>, sand: &mut Vec<Vec2>, moving_sands: &mut InstanceArray, settled_sands: &mut InstanceArray) {

    let instances = moving_sands.instances();
    let mut updated_sands = vec![];
    let mut idx_to_remove = None;

    for i in 0..sand.len() {
        let grain = &mut sand[i];
        let x = grain.x as usize;
        let y = grain.y as usize;

        if cave[y][x] == 'o' || y == cave.len() {
            settled_sands.push(instances[i]);
            idx_to_remove = Some(i);
            continue; 
        }

        if cave[y+1][x] != '.' {
            if cave[y+1][x-1] == '.' {
                *grain -= ONE_X_STEP;
            } else if cave[y+1][x+1] == '.' {
                *grain += ONE_X_STEP;
            } else {
                cave[y][x] = 'o';
                settled_sands.push(instances[i]);
                idx_to_remove = Some(i);
                continue;
            }
        }
        *grain += ONE_Y_STEP;

        updated_sands.push(
            DrawParam::new()
                .dest(get_dest(grain.x as i32, grain.y as i32))
                .color(SAND_COLOR)
                .scale(SAND_SCALE)
        );
    }

    if let Some(idx) = idx_to_remove {
        sand.remove(idx);
    }

    moving_sands.set(updated_sands);
}

fn parse_cave(input: &str, dims: (usize, usize), source: (i32, i32), offset: (i32, i32)) -> Vec<Vec<char>> {

    let offset_source = (source.0 + offset.0, source.1 + offset.1);
    let mut cave = vec![vec!['.'; dims.1]; dims.0];
    cave[offset_source.1 as usize][offset_source.0 as usize] = '+';

    for line in input.lines() {

        let pairs = line.replace(" -> ", ",").split(',').map(|str| str.parse::<i32>().unwrap()).collect::<Vec<i32>>();
        for pair in pairs.chunks(2).tuple_windows::<(_, _)>() {
            let idx_0 = (cmp::min(pair.0[0], pair.1[0]) + offset.0) as usize;
            let idx_1 = cmp::min(pair.0[1], pair.1[1]) as usize;
            let idx_2 = (cmp::max(pair.0[0], pair.1[0]) + offset.0) as usize;
            let idx_3 = cmp::max(pair.0[1], pair.1[1]) as usize;
 
            if idx_1 == idx_3 {
                for i in idx_0..(idx_2+1)  {
                    cave[idx_1][i] = '#';
                }
            } else {
                for i in idx_1..(idx_3+1) {
                    cave[i][idx_2] = '#';
                }
            }
        }
    }

    return cave;
}

fn main() -> GameResult {
    let input = fs::read_to_string("input").unwrap();

    let dims = (200, 200);
    let mut source = (500, 0);
    let offset = (-400, 0);
    let floored = true;

    // Parse the cave geometry from the input file
    let mut cave = parse_cave(&input, dims, source, offset);

    // Add the floor, if any, and determine source coordinates
    if floored { 
        cave = floor_cave(&mut cave).to_vec(); 
        source = (cave[0].iter().position(|c| c == &'+').unwrap() as i32, source.1 + offset.1);
    } else {
        source = (source.0 + offset.0, source.1 + offset.1);
    }

    // Setup and start the animation
    let (mut ctx, events_loop) = ggez::ContextBuilder::new("sand_dropper", "aheddini")
        .window_setup(ggez::conf::WindowSetup::default().title("Sand Dropper"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(WINDOW_SIZE.0, WINDOW_SIZE.1))
        .build()?;

    let state = GameState::new(&mut ctx, cave, Vec2::new(source.0 as f32, source.1 as f32))?;

    event::run(ctx, events_loop, state);
}