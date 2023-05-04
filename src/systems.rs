use std::ops::{Mul, AddAssign};

use rand::Rng;
use winit::event::*;

pub const SCREEN_SIZE: Vec2i = Vec2i {x: 800, y:500};

pub const CARD_SIZE: Vec2 = Vec2 { x: 160.0, y: 240.0 };

pub const DECK_QUAD: Quad = Quad {
    pos: Vec2 { x: -700.0, y: 350.0 },
    size: CARD_SIZE
};

const TICKS_PER_SECOND: f32 = 60.0;
const TICK_TIME: f32 = 1.0 / TICKS_PER_SECOND;

pub struct GameState {
    pub score: u8,
    pub mouse_pos: Vec2,
    pub stock: Stack,
    pub talon: Stack,
    pub tableaux: [Tableau; 7],
    pub foundations: [Stack; 4],
    pub hand: Stack,
    previous_time: instant::Instant,
    tick: f32
}

pub struct Tableau {
    pub cards: Vec<u8>,
    pub shown_cards: u8,
    pub x_position: f32
}

impl Tableau {
    pub fn empty() -> Self {
        Self {
            cards: vec![],
            shown_cards: 0,
            x_position: 0.0
        }
    }

    pub fn empty_tableaux() -> [Tableau; 7] {
        [
            Tableau::empty(),
            Tableau::empty(),
            Tableau::empty(),
            Tableau::empty(),
            Tableau::empty(),
            Tableau::empty(),
            Tableau::empty()
        ]
    }
}

#[derive(Debug)]
pub struct Quad {
    pub pos: Vec2,
    pub size: Vec2
}

#[derive(Debug)]
pub struct Stack {
    pub cards: Vec<u8>,
    pub quad: Quad
}

impl Stack {
    pub fn random_deck() -> Self {
        let mut cards : Vec<u8> = vec![];
        let mut possible_cards : Vec<u8> = (0..52).collect();
        
        for _ in 0..52 {
            let rand_index = rand::thread_rng().gen_range(0..possible_cards.len());
            let random_card = possible_cards.remove(rand_index);
            cards.push(random_card);
        }

        Stack {
            cards,
            quad: DECK_QUAD,
        }
    }

    pub fn empty() -> Self {
        Stack {
            cards: vec![],
            quad: Quad { pos: Vec2::zero(), size: CARD_SIZE },
        }
    }
}

impl Quad {
    pub fn new(pos: Vec2, size: Vec2) -> Self {
        Self {
            pos,
            size
        }
    }

    pub fn contains(&self, pos: Vec2) -> bool {
        return pos.y >= self.bottom() && 
               pos.y <= self.top() && 
               pos.x >= self.left() && 
               pos.x <= self.right()
    }

    pub fn top(&self) -> f32 {
        self.pos.y + (self.size.y / 2.0)
    }
    pub fn bottom(&self) -> f32 {
        self.pos.y - (self.size.y / 2.0)
    }
    pub fn right(&self) -> f32 {
        self.pos.x + (self.size.x / 2.0)
    }
    pub fn left(&self) -> f32 {
        self.pos.x - (self.size.x / 2.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vec2i {
    pub x: i32,
    pub y: i32
}

impl Mul<f32> for Vec2i {
    type Output = Vec2;

    fn mul(self, mul: f32) -> Vec2 {
        Vec2::new(self.x as f32 * mul, self.y as f32 * mul)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, mul: f32) -> Vec2 {
        Vec2::new(self.x as f32 * mul, self.y as f32 * mul)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32
}

impl Vec2 {
    pub fn new<T: num::ToPrimitive>(x: T, y: T) -> Self {
        Vec2 {
            x: x.to_f32().unwrap(),
            y: y.to_f32().unwrap()
        }
    }

    pub fn zero() -> Self {
        Vec2::new(0,0)
    }

    pub fn normalize(&mut self) -> Self {
        let mag = (self.x * self.x + self.y * self.y).sqrt();
        if mag != 0.0 { 
            self.x /= mag;
            self.y /= mag;
        }
        *self
    }
}

impl GameState {
    pub fn new() -> Self {

        let mut stock = Stack::random_deck();

        let tableaux = GameState::fill_tableaux(&mut stock);

        let mut talon = Stack::empty();
        talon.quad.pos = Vec2::new(-520, 350);

        GameState {
            stock,
            talon,
            tableaux,
            hand: Stack::empty(),
            foundations: GameState::create_foundations(),
            previous_time: instant::Instant::now(),
            mouse_pos: Vec2::zero(),
            tick: 0.0,
            score: 0
        }
    }

    pub fn fill_tableaux(deck: &mut Stack) -> [Tableau; 7] {
        let mut tableau = Tableau::empty_tableaux();
        for i in 0..7 {
            let stack = Tableau {
                x_position: -700.0 + ((CARD_SIZE.x + 20.0) * i as f32),
                cards: deck.cards.drain(0..(i + 1)).collect(),
                shown_cards: 1
            };
            tableau[i] = stack;
        }
        tableau
    }

    pub fn create_foundations() -> [Stack; 4] {
        let mut foundations = [Stack::empty(), Stack::empty(), Stack::empty(), Stack::empty()];
        for i in 0..4 {
            foundations[i].quad.pos =  Vec2::new(-160.0 + ((CARD_SIZE.x + 20.0) * i as f32), 350.0);
        } 
        foundations
    }

    pub fn update(&mut self) {
        let current_time = instant::Instant::now();
        let elapsed_time = current_time.duration_since(self.previous_time).as_secs_f32();
        self.previous_time = current_time;

        self.tick += elapsed_time;

        if self.tick > TICK_TIME {
            self.hand.quad.pos = self.mouse_pos;
            self.tick -= TICK_TIME;
        }
    }

    pub fn mouse_click(&mut self) {
        println!("{:?}", self.mouse_pos);
        if self.stock.quad.contains(self.mouse_pos) {
            if self.stock.cards.len() > 0 {
                self.talon.cards.insert(0, self.stock.cards.pop().unwrap());
            }
        }
        if self.talon.quad.contains(self.mouse_pos) {
            if self.stock.cards.len() == 0 {
                self.stock.cards.splice(.., self.talon.cards.drain(..));
            } else {
                self.hand.cards.push(self.talon.cards.remove(0));
            }
        }
        for stack in self.foundations.iter() {
            if stack.quad.contains(self.mouse_pos) {
                println!("{:?}", stack);
            }
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput { 
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.mouse_click();
                return true;
            }
            WindowEvent::CursorMoved {
                position,
                ..
            } => {
                self.mouse_pos = Vec2::new((position.x - (SCREEN_SIZE.x as f32 / 2.0) as f64) * 2.0, -(position.y - (SCREEN_SIZE.y as f32 / 2.0) as f64) * 2.0);
                return true;
            }
            _ => { 
                return false;
            }
        }
    }
}
