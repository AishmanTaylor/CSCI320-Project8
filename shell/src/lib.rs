#![cfg_attr(not(test), no_std)]

use bare_metal_modulo::{ModNumC};
use pluggable_interrupt_os::vga_buffer::{BUFFER_WIDTH, BUFFER_HEIGHT, plot, ColorCode, Color, plot_num, plot_str};
use pc_keyboard::{DecodedKey, KeyCode};
use rand::{rngs::SmallRng, SeedableRng, RngCore};
use array_append::push;
const NEW_WALL_FREQ: isize = 100;

// |GENERAL IDEA OF GAME STATE|
// [Score: (NUM)                                                  ]
// ################################################################
// #															  #
// #															  #
// #															  #
// #															  #
// #															  #
// #															  #
// #															  #
// #															  #
// #															  #
// #															  #
// #															  #
// #															  #
// #															  #
// #															  #
// ################################################################

//How to put score at the top of this? 

const WALLS: &str = "################################################################################
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
#                                                                              #
################################################################################";


pub struct Game {
    walls: Walls,
    player: Player,
    cannon: Cannon,
    playerbullet: PlayerBullet,
    asteroidhandler: AsteroidHandler,
    rng: SmallRng,
    tick_count: isize
}

impl Game {
    pub fn new() -> Self {
        Self {
            walls: Walls::new(WALLS), 
            player: Player::new(), 
            cannon: Cannon::new(), 
            playerbullet: PlayerBullet::new(), 
            asteroidhandler: AsteroidHandler::new(),
            rng: SmallRng::seed_from_u64(3),
            tick_count: 0
        }
    }

    pub fn key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(key) => {
                let mut player = self.player;
                let mut cannon = self.cannon;
                let mut bullet = self.playerbullet;
                let mut asteroid = self.asteroidhandler;
                match key {
                    KeyCode::S => {
                        player.down();
                        cannon.down();
                    }
                    KeyCode::W => {
                        player.up();
                        cannon.up();
                    } 
                    KeyCode::A => {
                        player.left();
                        cannon.left();
                    }
                    KeyCode::D => {
                        player.right();
                        cannon.right();
                    }
                    KeyCode::Q => {
                        cannon.q_pressed(&mut self.player)
                    }
                    KeyCode::E => {
                        cannon.e_pressed(&mut self.player)
                    }
                    KeyCode::Spacebar => {
                        bullet.spawn_bullet(&mut self.player, &mut self.cannon)
                    }
                    _ => {}
                }
                if !player.is_colliding(&self.walls) {
                    self.player = player;
                } else if !cannon.is_colliding(&self.walls) {
                    self.cannon = cannon;
                } else if !bullet.is_colliding(&self.walls) {
                    self.playerbullet = bullet;
                } else if !asteroid.is_colliding(&self.walls) {
                    self.asteroidhandler = asteroid;
                }
            }
            DecodedKey::Unicode(_) => {}
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        if self.tick_count % NEW_WALL_FREQ == 0 {
            self.asteroidhandler.add_asteroid(&mut self.rng);
        }
        self.walls.draw();
        self.player.draw_player();
        self.cannon.draw_cannon();
        self.asteroidhandler.num_update(&mut self.rng);
        self.asteroidhandler.tick_update(self.asteroidhandler.dx, self.asteroidhandler.dy);

        plot_num(self.tick_count, BUFFER_WIDTH / 2, 0, ColorCode::new(Color::White, Color::Black));
        plot_str("Score:", 60, 0, ColorCode::new(Color::White, Color::Black));
        // plot_num(self.player.bomb_count as isize, 66, 0, ColorCode::new(Color::LightRed, Color::Black));
    }

}
pub struct Walls {
    walls: [[bool; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

impl Walls {
    pub fn new(map: &str) -> Self {
        let mut walls = [[false; BUFFER_WIDTH]; BUFFER_HEIGHT];
        for (row, chars) in map.split('\n').enumerate() {
            for (col, value) in chars.char_indices() {
                walls[row][col] = value == '#';
            }
        }
        Self {walls}
    }

    pub fn draw(&self) {
        for row in 0..self.walls.len() {
            for col in 0..self.walls[row].len() {
                plot(self.char_at(row, col), col, row, ColorCode::new(Color::White, Color::Black));
            }
        }
    }

    pub fn occupied(&self, row: usize, col: usize) -> bool {
        self.walls[row][col]
    }

    fn char_at(&self, row: usize, col: usize) -> char {
        if self.walls[row][col] {
            '#'
        } else {
            ' '
        }
    }
}

#[derive(Copy,Clone)]
pub struct Player {
    x: usize,
    y: usize
}

impl Player {
    pub fn new() -> Self {
        Player {
            x: BUFFER_WIDTH / 2, 
            y: BUFFER_HEIGHT / 2
        }
    }

    pub fn draw_player(&mut self){
        plot('*', self.x, self.y, ColorCode::new(Color::White, Color::Black));
    }
    
    pub fn is_colliding(&self, walls: &Walls) -> bool {
        walls.occupied(self.y, self.x)
    }

    pub fn down(&mut self) {
        self.y += 1;
    }

    pub fn up(&mut self) {
        self.y -= 1;
    }

    pub fn left(&mut self) {
        self.x -= 1;
    }

    pub fn right(&mut self) {
        self.x += 1;
    }

}

#[derive(Copy,Clone)]
pub struct Cannon {
    x: usize,
    y: usize,
    letters: [char; BUFFER_WIDTH],
    dx: ModNumC<usize, BUFFER_WIDTH>,
    dy: ModNumC<usize, BUFFER_HEIGHT>,
}

impl Cannon {
    pub fn new() -> Self {
        Cannon {
            x: BUFFER_WIDTH / 2 + 1, 
            y: BUFFER_HEIGHT / 2,
            letters: ['|'; BUFFER_WIDTH],
            dx: ModNumC::new(0),
            dy: ModNumC::new(0),
        }
    }
    
    //HELP HERE
    pub fn is_colliding(&self, walls: &Walls) -> bool {
        walls.occupied(self.x, self.y)
    }

    pub fn draw_cannon(&mut self){
        plot('|', self.x, self.y, ColorCode::new(Color::White, Color::Black));
    }

    pub fn down(&mut self) {
        self.y += 1;
    }

    pub fn up(&mut self) {
        self.y -= 1;
    }

    pub fn left(&mut self) {
        self.x -= 1;
    }

    pub fn right(&mut self) {
        self.x += 1;
    }

    fn q_pressed(&mut self, player: &mut Player){
        if (self.x - player.x) > 0 && (self.y - player.y) == 0 {
            self.y -= 1;
            plot('`', self.x, self.y, ColorCode::new(Color::White, Color::Black));
        } else if (self.x - player.x) > 0 && (self.y - player.y) < 0 {
            self.x -= 1;
            plot('-', self.x, self.y, ColorCode::new(Color::White, Color::Black));
        } else if (self.x - player.x) == 0 && (self.y - player.y) < 0 {
            self.x -= 1;
            plot(',', self.x, self.y, ColorCode::new(Color::White, Color::Black));
        } else if (self.x - player.x) < 0 && (self.y - player.y) < 0 {
            self.y += 1;
            plot('|', self.x, self.y, ColorCode::new(Color::White, Color::Black));
        } else if (self.x - player.x) < 0 && (self.y - player.y) == 0 {
            self.y += 1;
            plot('`', self.x, self.y, ColorCode::new(Color::White, Color::Black));
        } else if (self.x - player.x) < 0 && (self.y - player.y) > 0 {
            self.x += 1;
            plot('-', self.x, self.y, ColorCode::new(Color::White, Color::Black));
        } else if (self.x - player.x) == 0 && (self.y - player.y) > 0 {
            self.x += 1;
            plot(',', self.x, self.y, ColorCode::new(Color::White, Color::Black));
        } else if (self.x - player.x) > 0 && (self.y - player.y) > 0 {
            self.y -= 1;
            plot('|', self.x, self.y, ColorCode::new(Color::White, Color::Black));
        }
    } 

    fn e_pressed(&mut self, player: &mut Player){
        if (self.x - player.x) > 0 && (self.y - player.y) == 0 {
            self.y += 1;
            plot(',', self.x, self.y, ColorCode::new(Color::White, Color::Black));
        } else if (self.x - player.x) > 0 && (self.y - player.y) < 0 {
            self.x += 1;
            plot('-', self.x, self.y, ColorCode::new(Color::White, Color::Black));
        } else if (self.x - player.x) == 0 && (self.y - player.y) < 0 {
            self.x += 1;
            plot('`', self.x, self.y, ColorCode::new(Color::White, Color::Black));
        } else if (self.x - player.x) < 0 && (self.y - player.y) < 0 {
            self.y -= 1;
            plot('|', self.x, self.y, ColorCode::new(Color::White, Color::Black));
        } else if (self.x - player.x) < 0 && (self.y - player.y) == 0 {
            self.y -= 1;
            plot(',', self.x, self.y, ColorCode::new(Color::White, Color::Black));
        } else if (self.x - player.x) < 0 && (self.y - player.y) > 0 {
            self.x -= 1;
            plot('-', self.x, self.y, ColorCode::new(Color::White, Color::Black));
        } else if (self.x - player.x) == 0 && (self.y - player.y) > 0 {
            self.x -= 1;
            plot('`', self.x, self.y, ColorCode::new(Color::White, Color::Black));
        } else if (self.x - player.x) > 0 && (self.y - player.y) > 0 {
            self.y += 1;
            plot('|', self.x, self.y, ColorCode::new(Color::White, Color::Black));
        }
    }

}
#[derive(Copy, Clone)]
pub struct Bullet {
    x: usize, 
    y: usize, 
    dx: usize, 
    dy: usize
}

#[derive(Copy,Clone)]
pub struct PlayerBullet {
    // |HELP|
    //When space bar is pressed at player, shoot this in direction it was facing until bullet hits an edge then delete
    //If bullet hits an asteroid, delete instance of bullet, add points, break asteroid 
    x: usize,
    y: usize,
    dx: usize,
    dy: usize,
    bullets: [Option<Bullet>; 25]
}

impl PlayerBullet {
    //HELP when space is pressed shoot in direction it's facing
    pub fn new() -> Self {
        PlayerBullet {
            x: BUFFER_WIDTH / 2, 
            y: BUFFER_HEIGHT / 2,
            dx: 0,
            dy: 0,
            bullets: [None; 25]
        }
    }

    pub fn is_colliding(&self, walls: &Walls) -> bool {
        walls.occupied(self.x, self.y)
    }

    fn spawn_bullet(&mut self, player: &mut Player, cannon: &mut Cannon){
        self.dx = cannon.x - player.x;
        self.dy = cannon.y - player.y;
        plot('.', self.x, self.y, ColorCode::new(Color::White, Color::Black));
    }

    fn tick_update(&mut self){
        self.x += self.dx;
        self.y == self.dy;
    }

}

#[derive(Copy, Clone)]
pub struct Asteroid {
    x: usize, 
    y: usize, 
    dx: usize, 
    dy: usize
}

#[derive(Copy,Clone)]
pub struct AsteroidHandler {
    //HELP
    x: usize,
    y: usize,
    dx: usize,
    dy: usize,
    asteroids: [Option<Asteroid>; 10]
}

impl AsteroidHandler {
    //Handle spawning and moving in random direction
    //Handle collisions with bullet
    //Scoring
    //Rewarch class video for detailed breakdown for random spawning of asteroids and collision handling 
    pub fn new() -> Self {
        AsteroidHandler {
            x: BUFFER_WIDTH / 2, 
            y: BUFFER_HEIGHT / 2,
            dx: 0,
            dy: 0,
            asteroids: [None; 10]
        }
    }

    pub fn is_colliding(&self, walls: &Walls) -> bool {
        walls.occupied(self.x, self.y)
    }

    pub fn add_asteroid(&mut self, rng: &mut SmallRng) {
        let num1: usize = 1 + rng.next_u32() as usize % (BUFFER_WIDTH - 1);
        let num2: usize = 1 + rng.next_u32() as usize % (BUFFER_HEIGHT - 1);
        self.asteroids.push(self.add(num1, num2));
    }

    pub fn num_update(&mut self, rng: &mut SmallRng) {
        let num1: usize = 1 + rng.next_u32() as usize % (BUFFER_WIDTH - 1);
        let num2: usize = 1 + rng.next_u32() as usize % (BUFFER_HEIGHT - 1);
        self.tick_update(num1, num2);
    }

    pub fn add(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }

    fn tick_update(&mut self, dx: usize, dy: usize){
        self.dx = dx;
        self.dy = dy;
        self.x += self.dx;
        self.y += self.dy;
    }

}