use bughouse_rs::logic::board::Piece;

pub fn validate_user_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_alphabetic() || c.is_whitespace())
}

pub fn parse_piece(s: &String) -> Option<Piece> {
    match s.as_str() {
        "b" => Some(Piece::b),
        "B" => Some(Piece::B),
        "E" => Some(Piece::E),
        "k" => Some(Piece::k),
        "K" => Some(Piece::K),
        "L" => Some(Piece::L),
        "n" => Some(Piece::n),
        "N" => Some(Piece::N),
        "p" => Some(Piece::p),
        "P" => Some(Piece::P),
        "q" => Some(Piece::q),
        "Q" => Some(Piece::Q),
        "r" => Some(Piece::r),
        "R" => Some(Piece::R),
        "Ub" => Some(Piece::Ub),
        "UB" => Some(Piece::UB),
        "Un" => Some(Piece::Un),
        "UN" => Some(Piece::UN),
        "Uq" => Some(Piece::Uq),
        "UQ" => Some(Piece::UQ),
        "Ur" => Some(Piece::Ur),
        "UR" => Some(Piece::UR),
        _ => None,
    }
}

pub fn parse_pos(s: &String) -> Option<(usize, usize)> {
    let mut buf = s.bytes();
    let col = buf.next()? as usize;
    let row = buf.next()? as usize;
    if col >= 97 && col <= 104 && row >= 48 && row <= 55 {
        Some((col - 97, row - 48))
    } else {
        None
    }
}
