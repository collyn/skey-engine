//! Precomposed UTF-8 Vietnamese character lookup tables.
//! tone 0=none, 1=grave, 2=acute, 3=hook, 4=tilde, 5=dot

type ToneTable = [[&'static str; 6]; 3];

pub static A_LO: ToneTable = [
    ["a", "à", "á", "ả", "ã", "ạ"],
    ["â", "ầ", "ấ", "ẩ", "ẫ", "ậ"],
    ["ă", "ằ", "ắ", "ẳ", "ẵ", "ặ"],
];
pub static A_UP: ToneTable = [
    ["A", "À", "Á", "Ả", "Ã", "Ạ"],
    ["Â", "Ầ", "Ấ", "Ẩ", "Ẫ", "Ậ"],
    ["Ă", "Ằ", "Ắ", "Ẳ", "Ẵ", "Ặ"],
];
pub static E_LO: ToneTable = [
    ["e", "è", "é", "ẻ", "ẽ", "ẹ"],
    ["ê", "ề", "ế", "ể", "ễ", "ệ"],
    ["e", "è", "é", "ẻ", "ẽ", "ẹ"],
];
pub static E_UP: ToneTable = [
    ["E", "È", "É", "Ẻ", "Ẽ", "Ẹ"],
    ["Ê", "Ề", "Ế", "Ể", "Ễ", "Ệ"],
    ["E", "È", "É", "Ẻ", "Ẽ", "Ẹ"],
];
pub static I_LO: ToneTable = [
    ["i", "ì", "í", "ỉ", "ĩ", "ị"],
    ["i", "ì", "í", "ỉ", "ĩ", "ị"],
    ["i", "ì", "í", "ỉ", "ĩ", "ị"],
];
pub static I_UP: ToneTable = [
    ["I", "Ì", "Í", "Ỉ", "Ĩ", "Ị"],
    ["I", "Ì", "Í", "Ỉ", "Ĩ", "Ị"],
    ["I", "Ì", "Í", "Ỉ", "Ĩ", "Ị"],
];
pub static O_LO: ToneTable = [
    ["o", "ò", "ó", "ỏ", "õ", "ọ"],
    ["ô", "ồ", "ố", "ổ", "ỗ", "ộ"],
    ["ơ", "ờ", "ớ", "ở", "ỡ", "ợ"],
];
pub static O_UP: ToneTable = [
    ["O", "Ò", "Ó", "Ỏ", "Õ", "Ọ"],
    ["Ô", "Ồ", "Ố", "Ổ", "Ỗ", "Ộ"],
    ["Ơ", "Ờ", "Ớ", "Ở", "Ỡ", "Ợ"],
];
pub static U_LO: ToneTable = [
    ["u", "ù", "ú", "ủ", "ũ", "ụ"],
    ["u", "ù", "ú", "ủ", "ũ", "ụ"],
    ["ư", "ừ", "ứ", "ử", "ữ", "ự"],
];
pub static U_UP: ToneTable = [
    ["U", "Ù", "Ú", "Ủ", "Ũ", "Ụ"],
    ["U", "Ù", "Ú", "Ủ", "Ũ", "Ụ"],
    ["Ư", "Ừ", "Ứ", "Ử", "Ữ", "Ự"],
];
pub static Y_LO: ToneTable = [
    ["y", "ỳ", "ý", "ỷ", "ỹ", "ỵ"],
    ["y", "ỳ", "ý", "ỷ", "ỹ", "ỵ"],
    ["y", "ỳ", "ý", "ỷ", "ỹ", "ỵ"],
];
pub static Y_UP: ToneTable = [
    ["Y", "Ỳ", "Ý", "Ỷ", "Ỹ", "Ỵ"],
    ["Y", "Ỳ", "Ý", "Ỷ", "Ỹ", "Ỵ"],
    ["Y", "Ỳ", "Ý", "Ỷ", "Ỹ", "Ỵ"],
];

pub fn vowel_utf8(base: char, variant: u8, tone: u8, upper: bool) -> &'static str {
    let v = (variant as usize).min(2);
    let t = (tone as usize).min(5);
    let table: &ToneTable = match base.to_ascii_lowercase() {
        'a' => if upper { &A_UP } else { &A_LO },
        'e' => if upper { &E_UP } else { &E_LO },
        'i' => if upper { &I_UP } else { &I_LO },
        'o' => if upper { &O_UP } else { &O_LO },
        'u' => if upper { &U_UP } else { &U_LO },
        'y' => if upper { &Y_UP } else { &Y_LO },
        _ => return if upper { &A_UP } else { &A_LO }[0][0], // fallback
    };
    table[v][t]
}

pub fn is_vowel(c: char) -> bool {
    matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

pub const D_STROKE_LO: &str = "\u{0111}"; // đ
pub const D_STROKE_UP: &str = "\u{0110}"; // Đ

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_vowels() {
        assert_eq!(vowel_utf8('a', 0, 2, false), "á");
        assert_eq!(vowel_utf8('a', 0, 1, false), "à");
        assert_eq!(vowel_utf8('a', 1, 2, false), "ấ");
        assert_eq!(vowel_utf8('a', 2, 2, false), "ắ");
        assert_eq!(vowel_utf8('o', 2, 1, false), "ờ");
        assert_eq!(vowel_utf8('u', 2, 2, false), "ứ");
        assert_eq!(vowel_utf8('e', 1, 2, false), "ế");
    }

    #[test]
    fn uppercase_vowels() {
        assert_eq!(vowel_utf8('a', 0, 2, true), "Á");
        assert_eq!(vowel_utf8('A', 1, 0, true), "Â");
        assert_eq!(vowel_utf8('o', 2, 1, true), "Ờ");
    }
}
