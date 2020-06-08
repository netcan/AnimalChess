use sdl2::render::{Texture, TextureCreator};
use sdl2::image::LoadTexture;
use std::slice::Iter;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ChessKind {
    ELEPHANT,
    LION,
    TIGER,
    PANTHER,
    WOLF,
    DOG,
    CAT,
    RAT,
    EMPTY,
}

impl ChessKind {
    pub fn get_idx(self) -> usize {
        use ChessKind::*;
        match self {
            ELEPHANT => 0,
            LION     => 1,
            TIGER    => 2,
            PANTHER  => 3,
            WOLF     => 4,
            DOG      => 5,
            CAT      => 6,
            RAT      => 7,
            EMPTY    => panic!("caller should guarantee chess type not empty")
        }
    }
    pub fn iter() -> Iter<'static, Self> {
        use self::ChessKind::*;
        static KIND: [ChessKind; 8] = [
            ELEPHANT, LION, TIGER, PANTHER, WOLF, DOG, CAT, RAT,
        ];
        KIND.iter()
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RoleType {
    RED,
    BLACK,
    EMPTY,
}

impl RoleType {
    pub fn iter() -> Iter<'static, Self> {
        use self::RoleType::*;
        static ROLE: [RoleType; 2] = [ RED, BLACK ];
        ROLE.iter()
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct ChessId {
    pub role: RoleType,
    pub kind: ChessKind,
}

impl ChessId {
    pub fn get_chess_idx(self) -> usize {
        use RoleType::*;
        self.kind.get_idx() + match self.role {
            RED   => 0,
            BLACK => 8,
            EMPTY => panic!("caller should guarantee chess id not empty")
        }
    }

    pub fn get_chess_texture<T>(self, texture_creator: &TextureCreator<T>) -> Texture {
        let mut path = String::from("assets/");
        use RoleType::*;
        use ChessKind::*;
        match self.role {
            RED   => { path.push('r'); }
            BLACK => { path.push('b'); }
            _     => { unreachable!(); }
        }
        match self.kind {
            ELEPHANT => { path.push_str("e.png"); }
            LION     => { path.push_str("l.png"); }
            TIGER    => { path.push_str("t.png"); }
            PANTHER  => { path.push_str("p.png"); }
            WOLF     => { path.push_str("w.png"); }
            DOG      => { path.push_str("d.png"); }
            CAT      => { path.push_str("c.png"); }
            RAT      => { path.push_str("r.png"); }
            _        => { unreachable!();         }
        }

        texture_creator.load_texture(path).expect("load texture failed")
    }
}

pub const EMPTY_CHESS: ChessId = ChessId { kind: ChessKind::EMPTY, role: RoleType::EMPTY };
