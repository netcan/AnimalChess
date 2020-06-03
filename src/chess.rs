use sdl2::render::{Texture, TextureCreator};
use sdl2::image::LoadTexture;

pub type ChessType = u8;
pub type Role      = u8;
pub type ChessId   = u8;

pub const ELEPHANT: ChessType = 0x0;
pub const LION:     ChessType = 0x1;
pub const TIGER:    ChessType = 0x2;
pub const PANTHER:  ChessType = 0x3;
pub const WOLF:     ChessType = 0x4;
pub const DOG:      ChessType = 0x5;
pub const CAT:      ChessType = 0x6;
pub const RAT:      ChessType = 0x7;

pub const RED:     Role      = 0x8;
pub const BLACK:   Role      = 0x10;

pub const EMPTY:   ChessId   = 0x0;

pub fn get_chess_id(role: Role, chess_type: ChessType) -> ChessId {
    role | chess_type
}

pub fn get_chess_role(id: ChessId) -> Role {
    return id & 0x18;
}

pub fn get_chess_type(id: ChessId) -> ChessType {
    return id & 0x7;
}

pub struct Chess {
    pub id: ChessId,
    pub texture: Texture,
}

impl Chess {
    fn get_chess_texture<T>(chess_id: ChessId, texture_creator: &TextureCreator<T>) -> Texture {
        let mut path = String::from("assets/");
        match chess_id {
            EMPTY => { path.push_str("oo.gif"); }
            id => {
                match get_chess_role(id) {
                    RED   => { path.push('r'); }
                    BLACK => { path.push('b'); }
                    _     => unreachable!()
                }

                match get_chess_type(id) {
                    ELEPHANT => { path.push_str("e.png"); }
                    LION     => { path.push_str("l.png"); }
                    TIGER    => { path.push_str("t.png"); }
                    PANTHER  => { path.push_str("p.png"); }
                    WOLF     => { path.push_str("w.png"); }
                    DOG      => { path.push_str("d.png"); }
                    CAT      => { path.push_str("c.png"); }
                    RAT      => { path.push_str("r.png"); }
                    _       => unreachable!()
                }
            }
        }

        texture_creator.load_texture(path).expect("load texture failed")
    }

    pub fn new<T>(id: ChessId, texture_creator: &TextureCreator<T>) -> Self {
        Self { id, texture: Self::get_chess_texture(id, texture_creator) }
    }
}
