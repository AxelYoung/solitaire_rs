use std::{ops::{Mul, AddAssign}, vec, fs::File};
use std::io::Cursor;
use std::io::Read;

use rand::Rng;
use winit::event::*;

use kira::{
    manager::{
        AudioManager, AudioManagerSettings,
        backend::cpal::CpalBackend,
    },
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};

pub const SCREEN_SIZE: Vec2i = Vec2i {x: 1200, y:900};

pub const CARD_SIZE: Vec2 = Vec2 { x: 160.0, y: 240.0 };

pub const DECK_QUAD: Quad = Quad {
    pos: Vec2 { x: -700.0, y: 350.0 },
    size: CARD_SIZE
};

const TICKS_PER_SECOND: f32 = 60.0;
const TICK_TIME: f32 = 1.0 / TICKS_PER_SECOND;

pub struct GameState {
    pub stock: Stack,
    pub talon: Stack,
    pub tableaux: [Tableau; 7],
    pub foundations: [Stack; 4],
    pub hand: Stack,
    hand_origin: u8,
    mouse_pos: Vec2,
    previous_time: instant::Instant,
    tick: f32
}

#[derive(Debug, PartialEq)]
pub struct Tableau {
    pub cards: Vec<Card>,
    pub card_quads: Vec<Quad>,
    pub shown_cards: u8,
    pub x_position: f32
}

#[derive(Debug, PartialEq)]
pub struct Card {
    pub value: u8,
    pub rank: u8,
    pub color: Color,
    pub suit: Suit
}

impl Card {

    pub fn new(value: u8) -> Self {
        Self {
            value,
            rank: Card::get_rank(value),
            color: Card::get_color(value),
            suit: Card::get_suit(value)
        }
    }
    
    fn get_rank(value: u8) -> u8{
        value % 13
    }

    fn get_color(value: u8) -> Color {
        match value / 13 {
            0 => { Color::Black },
            2 => { Color::Black },
            _ => { Color::Red }
        }
    }

    fn get_suit(value: u8) -> Suit {
        match value / 13 {
            0 => { Suit::Spade },
            1 => { Suit::Heart },
            2 => { Suit::Club },
            _ => { Suit::Diamond }
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Color {
    Red,
    Black
}

#[derive(PartialEq, Debug)]
pub enum Suit {
    Spade,
    Heart,
    Club,
    Diamond
}

impl Tableau {
    pub fn empty() -> Self {
        Self {
            cards: vec![],
            card_quads: vec![],
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

    pub fn calculate_card_quads(&mut self) {
        self.card_quads = vec![];
        if self.cards.len() == 0 {
            self.card_quads.push( 
                Quad {
                    pos: Vec2 { x: self.x_position, y: 0.0 },
                    size: CARD_SIZE
                }
            );
        } else {
            for i in 0..self.cards.len() {
                self.card_quads.push( 
                    Quad {
                        pos: Vec2 { x: self.x_position, y: -(i as f32 * 70.0) },
                        size: CARD_SIZE
                    }
                );
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Quad {
    pub pos: Vec2,
    pub size: Vec2
}

pub struct Stack {
    pub cards: Vec<Card>,
    pub quad: Quad
}

impl Stack {
    pub fn random_deck() -> Self {
        let mut cards = vec![];
        let mut possible_cards : Vec<u8> = (0..52).collect();
        
        for _ in 0..52 {
            let rand_index = rand::thread_rng().gen_range(0..possible_cards.len());
            let random_card = possible_cards.remove(rand_index);
            cards.push(Card::new(random_card));
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

#[derive(Debug, Clone, Copy, PartialEq)]
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
            hand_origin: 0,
            tick: 0.0
        }
    }

    pub fn fill_tableaux(deck: &mut Stack) -> [Tableau; 7] {
        let mut tableau = Tableau::empty_tableaux();
        for i in 0..7 {
            let x_position = -700.0 + ((CARD_SIZE.x + 20.0) * i as f32);
            let mut stack = Tableau {
                x_position,
                card_quads: vec![],
                cards: deck.cards.drain(0..(i + 1)).collect(),
                shown_cards: 1
            };
            stack.calculate_card_quads();
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
        if self.hand.cards.len() == 0 {
            if self.stock.quad.contains(self.mouse_pos) {
                if self.stock.cards.len() > 0 {
                    self.talon.cards.insert(0, self.stock.cards.pop().unwrap());
                } else {
                    self.stock.cards.splice(.., self.talon.cards.drain(..));
                }
                GameState::play_audio(1);
            }
            if self.talon.quad.contains(self.mouse_pos) {
                if self.talon.cards.len() > 0 {
                    self.hand.cards.push(self.talon.cards.remove(0));
                    self.hand_origin = 0;
                    GameState::play_audio(0);
                    return;
                }
            }
            for (t, tableau) in self.tableaux.iter_mut().enumerate() {
                // Reverse is important, checks collision front to back
                for i in (0..tableau.card_quads.len()).rev() {
                    if tableau.cards.len() > 0 {
                        if i >= tableau.cards.len() - tableau.shown_cards as usize {
                            // for each shown card in each tableau
                            if tableau.card_quads[i].contains(self.mouse_pos) {
                                tableau.shown_cards -= (tableau.cards.len() - i) as u8;
                                self.hand.cards.splice(.., tableau.cards.drain(i..tableau.cards.len()));
                                tableau.calculate_card_quads();
                                println!("{:?}", tableau);
                                self.hand_origin = 5 + t as u8;
                                GameState::play_audio(0);
                                return;
                            }
                        }
                    }

                }
            }
            for (f, foundation) in self.foundations.iter_mut().enumerate() {
                if foundation.cards.len() > 0 {
                    if foundation.quad.contains(self.mouse_pos) {
                        self.hand.cards.push(foundation.cards.remove(0));
                        self.hand_origin = 1 + f as u8;
                        GameState::play_audio(0);
                        return;
                    }
                }
            }
        } else {
            for (t, tableau) in self.tableaux.iter_mut().enumerate() {
                if tableau.card_quads[tableau.card_quads.len() - 1].contains(self.mouse_pos) {
                    if tableau.cards.len() == 0 || 
                        GameState::can_place_on_tableau(&tableau.cards[tableau.cards.len() - 1], &self.hand.cards[0]) {
                            tableau.shown_cards += self.hand.cards.len() as u8;
                            tableau.cards.append(&mut self.hand.cards);
                            tableau.calculate_card_quads();
                            println!("{:?}", tableau);
                            match self.hand_origin {
                                5.. => {
                                    let origin = self.hand_origin - 5;
                                    if t as u8 != origin {
                                        if self.tableaux[origin as usize].cards.len() > 0 {
                                            if self.tableaux[origin as usize].shown_cards == 0 {
                                                self.tableaux[origin as usize].shown_cards += 1;
                                            }
                                        } 
                                    }
                                },
                                _ => {}
                            }
                        GameState::play_audio(1);
                        return;
                    }
                }
            }
            for foundation in self.foundations.iter_mut() {
                if foundation.quad.contains(self.mouse_pos) {
                    if self.hand.cards.len() == 1 {
                        if GameState::can_place_on_foundation(&foundation, &self.hand.cards[0]) {
                                foundation.cards.insert(0, self.hand.cards.remove(0));
                                match self.hand_origin {
                                    5.. => {
                                        let origin = self.hand_origin - 5;
                                        if self.tableaux[origin as usize].cards.len() > 0 {
                                            if self.tableaux[origin as usize].shown_cards == 0 {
                                                self.tableaux[origin as usize].shown_cards += 1;
                                            }
                                        } 
                                    },
                                    _ => {}
                                }
                                GameState::play_audio(1);
                                return;
                            }
                    }
                }
            }
        }
    }

    pub fn return_card(&mut self) {
        if self.hand.cards.len() != 0 {
            match self.hand_origin {
                0 => {
                    self.talon.cards.insert(0, self.hand.cards.remove(0));
                },
                1..=4 => {
                    self.foundations[(self.hand_origin - 1) as usize].cards.insert(0, self.hand.cards.remove(0));
                },
                5.. => {
                    self.tableaux[(self.hand_origin - 5) as usize].shown_cards += self.hand.cards.len() as u8;
                    self.tableaux[(self.hand_origin - 5) as usize].cards.append(&mut self.hand.cards);
                    self.tableaux[(self.hand_origin - 5) as usize].calculate_card_quads();
                }
            }
            GameState::play_audio(1);
        }
    }

    fn can_place_on_tableau(tableau: &Card, hand: &Card) -> bool {
        tableau.color != hand.color && tableau.rank == hand.rank + 1
    }

    fn can_place_on_foundation(foundation: &Stack, hand: &Card) -> bool {
        let foundation_size = foundation.cards.len();
        if foundation_size == 0 {
            if hand.rank == 0 { return true; }
            else { return false; }
        }
        let foundation_card = &foundation.cards[0];
        if foundation_card.suit == hand.suit && foundation_card.rank == hand.rank - 1 { return true }
        false
    }

    fn play_audio(id: u8) {
        let mut audio_manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).unwrap();
        let audio;
        match id {
            0 => { audio = include_bytes!("aud/place_card.ogg").to_vec(); }
            _ => { audio = include_bytes!("aud/pick_up_card.ogg").to_vec();}
        }
        let cursor = Cursor::new(audio);
        let sound_data = StaticSoundData::from_cursor(cursor, StaticSoundSettings::default()).unwrap();
        audio_manager.play(sound_data.clone()).unwrap();
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
            WindowEvent::KeyboardInput { 
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Space),
                    ..
                },
                ..
            } => {
                self.return_card();
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
