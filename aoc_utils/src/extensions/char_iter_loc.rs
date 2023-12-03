/// The location of a character within a char stream
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Loc {
    // The zero-indexed line on which a character occurs
    pub line_num: usize,

    // The zero-indexed column in which a character occurs
    pub col_num: usize,
}

pub trait CharIterLocExt {
    fn with_char_loc(self) -> impl Iterator<Item = (Loc, char)>;
}

impl<Iter> CharIterLocExt for Iter
where
    Iter: Iterator<Item = char>,
{
    fn with_char_loc(self) -> impl Iterator<Item = (Loc, char)> {
        self.scan(Loc::default(), |state: &mut Loc, c: char| {
            let this_loc = *state;
            match c {
                '\n' => {
                    state.line_num += 1;
                    state.col_num = 0;
                }
                _ => {
                    state.col_num += 1;
                }
            }

            Some((this_loc, c))
        })
    }
}
