#![warn(clippy::all, clippy::pedantic)]
use bracket_lib::prelude::*;

enum GameMode {
    Menu,
    Play,
    End,
}

const SCREEN_WIDTH : i32 = 80; 
const SCREEN_HEIGHT : i32 = 50; 
const FRAME_DURATION : f32 = 75.0;
const PLAYER_OFFSET : i32 = 20;

struct Player {
    x: i32,
    y: i32,
    velocity: f32,
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Player {
            x,
            y,
            velocity: 0.0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set(
            PLAYER_OFFSET,
            self.y,
            YELLOW,
            BLACK,
            to_cp437('@')
        );
    }

    fn gravity_and_move(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }
        self.y += self.velocity as i32;
        self.x += 1;
        if self.y < 0 {
            self.y = 0;
        }
    }

    fn flap(&mut self) {
        self.velocity = -2.1;
    }
}

struct State {
    player: Player,
    frame_time: f32,
    obstacle_one: Obstacle,
    obstacle_two: Obstacle,
    mode: GameMode,
    score: i32,
    high_score: i32,
}

impl State {
    fn new(high_score: i32) -> Self {
        State {
            player: Player::new(0, 25),
            frame_time: 0.0,
            obstacle_one: Obstacle::new(SCREEN_WIDTH, 0),
            obstacle_two: Obstacle::new(SCREEN_WIDTH, 1),
            mode: GameMode::Menu,
            score: 0,
            high_score,
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move();
        }
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }
        self.player.render(ctx);
        ctx.print(0, 0, "Press SPACE to flap.");
        ctx.print(0, 1, &format!("SCORE: {}", self.score));

        // show debug values at the top right
        // ctx.print(50, 0, &format!("Player X: {}", self.player.x + PLAYER_OFFSET));
        ctx.print(50, 1, &format!("Obstacles X: {} , {}", self.obstacle_one.x, self.obstacle_two.x));

        // add obstacles
        self.obstacle_one.render(ctx, self.player.x);
        self.obstacle_two.render(ctx, self.player.x);
        if self.player.x > self.obstacle_one.x {
            self.score += 1;
            self.obstacle_one = Obstacle::new(
                self.player.x + SCREEN_WIDTH, self.score
            );
        } else if self.player.x > self.obstacle_two.x {
            self.score += 1;
            self.obstacle_two = Obstacle::new(
                self.player.x + SCREEN_WIDTH, self.score
            );
        } 

        if self.player.y > SCREEN_HEIGHT ||
            self.obstacle_one.hit_obstacle(&self.player) ||
            self.obstacle_two.hit_obstacle(&self.player)
        {
            self.mode = GameMode::End;
        }
    }

    fn restart(&mut self) {
        if self.score > self.high_score {
            self.high_score = self.score;
        }
        self.player = Player::new(20, 25);
        self.frame_time = 0.0;
        self.obstacle_one = Obstacle::new(SCREEN_WIDTH + PLAYER_OFFSET, 0);
        self.obstacle_two = Obstacle::new(self.obstacle_one.x + (SCREEN_WIDTH / 2), 1);
        self.score = 0;
        self.mode = GameMode::Play;
    }

    fn main_menu(&mut self, ctx: &mut BTerm, title: &str, subtitle: &str) {
        ctx.cls();
        ctx.print_centered(5, title);
        ctx.print_centered(6, subtitle);
        ctx.print_centered(8, "[P] Play");
        ctx.print_centered(9, "[Q] Quit");

        // get the key press
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx, "Welcome to Flappy Dragon", "By Ryan!"),
            GameMode::End => self.main_menu(ctx, "You have died.", 
                &format!("Your Score: {}, High Score: {}", self.score, self.high_score)),
            GameMode::Play => self.play(ctx),
        }
    }
}

struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32,
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(10, 40),
            size: i32::max(2, 20 - score),
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x;
        let half_size = self.size / 2;

        // draw the top half of the obstacle
        for y in 0..self.gap_y - half_size {
            ctx.set(
                screen_x,
                y,
                RED,
                BLACK,
                to_cp437('|'),
            );
        }

        // draw the bottom half of the obstacle
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(
                screen_x,
                y,
                RED,
                BLACK,
                to_cp437('|'),
            );
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.x + PLAYER_OFFSET == self.x;
        let player_above_gap = player.y < self.gap_y - half_size;
        let player_below_gap = player.y > self.gap_y + half_size;
        does_x_match && (player_above_gap || player_below_gap)
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon")
        .build()?;
    main_loop(context, State::new(0))
}
