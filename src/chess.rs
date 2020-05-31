use sdl2::render::{Texture, TextureCreator};
use sdl2::image::LoadTexture;

pub type ChessType = u8;
pub type Role      = u8;
pub type ChessId   = u8;

pub const KING:    ChessType = 0x1;
pub const ADVISOR: ChessType = 0x2;
pub const BISHOP:  ChessType = 0x3;
pub const KNIGHT:  ChessType = 0x4;
pub const ROOK:    ChessType = 0x5;
pub const CANNON:  ChessType = 0x6;
pub const PAWN:    ChessType = 0x7;

pub const RED:     Role      = 0x0;
pub const BLACK:   Role      = 0x8;

pub const EMPTY:   ChessId   = 0x0;

pub fn get_chess_id(role: Role, chess_type: ChessType) -> ChessId {
    role | chess_type
}

pub fn get_chess_role(id: ChessId) -> Role {
    return id & 0x8;
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
                    KING    => { path.push_str("k.gif"); }
                    ADVISOR => { path.push_str("a.gif"); }
                    BISHOP  => { path.push_str("b.gif"); }
                    KNIGHT  => { path.push_str("n.gif"); }
                    ROOK    => { path.push_str("r.gif"); }
                    CANNON  => { path.push_str("c.gif"); }
                    PAWN    => { path.push_str("p.gif"); }
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
