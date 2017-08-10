use std::collections::BTreeMap;

/// Represent a text encoding used in PDF.
/// An encoding maintains the connection between unicode code points,
/// bytes in PDF strings, and glyph names.
///
/// Currently, only WIN_ANSI_ENCODING and SYMBOL_ENCODING are supported,
/// and they are provided as built-in.
///
/// # Example
/// ````
/// use pdf_canvas::{BuiltinFont, FontSource};
/// assert_eq!("WinAnsiEncoding",
///            BuiltinFont::Helvetica.get_encoding().get_name());
/// assert_eq!("SymbolEncoding",
///            BuiltinFont::Symbol.get_encoding().get_name());
/// ````
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Encoding {
    name: String,
    name_to_code: BTreeMap<&'static str, u8>,
    unicode_to_code: BTreeMap<char, u8>,
}

impl Encoding {
    /// The name of the encoding, as used in the font object.
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    /// Get the encoded code point from a type1 character name.
    /// Character names are case sensitive and contains only ascii letters.
    /// If the name is not available in the encoding, or is not a proper
    /// character name, None is returned.
    ///
    /// # Example
    /// ````
    /// use pdf_canvas::{BuiltinFont, FontSource};
    /// let enc = BuiltinFont::Helvetica.get_encoding();
    /// assert_eq!(Some(32),  enc.get_code("space"));
    /// assert_eq!(Some(65),  enc.get_code("A"));
    /// assert_eq!(Some(229), enc.get_code("aring"));
    /// assert_eq!(None,      enc.get_code("Lslash"));
    /// assert_eq!(None,      enc.get_code(""));
    /// assert_eq!(None,      enc.get_code("☺"));
    /// ````
    pub fn get_code(&self, name: &str) -> Option<u8> {
        match self.name_to_code.get(name) {
            Some(&code) => Some(code),
            None => None,
        }
    }

    /// Convert a rust string to a vector of bytes in the encoding.
    /// # Example
    /// ````
    /// use pdf_canvas::{BuiltinFont, FontSource};
    /// let enc = BuiltinFont::Helvetica.get_encoding();
    /// let symb_enc = BuiltinFont::Symbol.get_encoding();
    /// assert_eq!(vec!(65, 66, 67), enc.encode_string("ABC"));
    /// assert_eq!(vec!(82, 228, 107, 115, 109, 246, 114, 103, 229, 115),
    ///            enc.encode_string("Räksmörgås"));
    /// assert_eq!(vec!(67, 111, 102, 102, 101, 101, 32, 128, 49, 46, 50, 48),
    ///            enc.encode_string("Coffee €1.20"));
    /// assert_eq!(vec!(97, 32, 206, 32, 194),
    ///            symb_enc.encode_string("α ∈ ℜ"));
    /// ````
    pub fn encode_string(&self, text: &str) -> Vec<u8> {
        let mut result = Vec::new();
        for ch in text.chars() {
            match ch {
                '\\' => {
                    result.push('\\' as u8);
                    result.push('\\' as u8)
                }
                '(' => {
                    result.push('\\' as u8);
                    result.push('(' as u8)
                }
                ')' => {
                    result.push('\\' as u8);
                    result.push(')' as u8)
                }
                ch => {
                    result.push(*self.unicode_to_code
                        .get(&ch)
                        .unwrap_or(&('?' as u8)))
                }
            }
        }
        result
    }

    fn init_block(&mut self, start: u8, data: Vec<&'static str>) {
        let mut i = start - 1;
        for name in data {
            i += 1;
            self.name_to_code.insert(name, i);
        }
    }
}

lazy_static! {
    pub static ref WIN_ANSI_ENCODING: Encoding = {
        let mut codes = BTreeMap::new();
        // /WinAnsiEncoding is kind of close to first byte of unicode
        // Except for the 16 chars that are reserved in 8859-1 and
        // used in Windows-1252.
        for code in 32..255 {
            codes.insert(code as char, code);
        }
        codes.insert('€', 128);
        codes.insert('‚', 130);
        codes.insert('ƒ', 131);
        codes.insert('„', 132);
        codes.insert('…', 133);
        codes.insert('†', 134);
        codes.insert('‡', 135);
        codes.insert('ˆ', 136);
        codes.insert('‰', 137);
        codes.insert('Š', 138);
        codes.insert('‹', 139);
        codes.insert('Œ', 140);
        codes.insert('Ž', 142);
        codes.insert('‘', 145);
        codes.insert('’', 146);
        codes.insert('“', 147);
        codes.insert('”', 148);
        codes.insert('•', 149);
        codes.insert('–', 150);
        codes.insert('—', 151);
        codes.insert('˜', 152);
        codes.insert('™', 153);
        codes.insert('š', 154);
        codes.insert('›', 155);
        codes.insert('ž', 158);
        codes.insert('Ÿ', 159);
        let mut result = Encoding {
            name: "WinAnsiEncoding".to_string(),
            name_to_code: BTreeMap::new(),
            unicode_to_code: codes
        };
        result.init_block(0o40, vec!(
            "space", "exclam", "quotedbl", "numbersign",
            "dollar", "percent", "ampersand", "quotesingle"));
        result.init_block(0o50, vec!(
            "parenleft", "parenright", "asterisk", "plus",
            "comma", "hyphen", "period", "slash"));
        result.init_block(0o60, vec!(
            "zero", "one", "two", "three", "four", "five", "six", "seven"));
        result.init_block(0o70, vec!(
            "eight", "nine", "colon", "semicolon",
            "less", "equal", "greater", "question"));
        result.init_block(0o100, vec!(
            "at", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J",
            "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V",
            "W", "X", "Y", "Z"));
        result.init_block(0o133, vec!(
            "bracketleft",
            "backslash", "bracketright", "asciicircum", "underscore"));
        result.init_block(0o140, vec!(
            "grave", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j",
            "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v",
            "w", "x", "y", "z"));
        result.init_block(0o173, vec!(
            "braceleft", "bar", "braceright", "asciitilde"));
        result.init_block(0o200, vec!(
            "Euro", "..1", "quotesinglbase", "florin",
            "quotedblbase", "ellipsis", "dagger", "daggerdbl"));
        result.init_block(0o210, vec!(
            "circumflex", "perthousand", "Scaron", "guilsinglleft",
            "OE", "..5", "Zcaron", "..7"));
        result.init_block(0o220, vec!(
            "..0", "quoteleft", "quoteright", "quotedblleft",
            "quotedblright", "bullet", "endash", "emdash"));
        result.init_block(0o230, vec!(
            "tilde", "trademark", "scaron", "guilsinglright",
            "oe", "..5", "zcaron", "Ydieresis"));
        result.init_block(0o240, vec!(
            "..0", "exclamdown", "cent", "sterling",
            "currency", "yen", "brokenbar", "section"));
        result.init_block(0o250, vec!(
            "dieresis", "copyright", "ordfeminine", "guillemotleft",
            "logicalnot", "..5", "registered", "macron"));
        result.init_block(0o260, vec!(
            "degree", "plusminus", "twosuperior", "threesuperior",
            "acute", "mu", "paragraph", "periodcentered"));
        result.init_block(0o270, vec!(
            "cedilla", "onesuperior", "ordmasculine", "guillemotright",
            "onequarter", "onehalf", "threequarters", "questiondown"));
        result.init_block(0o300, vec!(
            "Agrave", "Aacute", "Acircumflex", "Atilde",
            "Adieresis", "Aring", "AE", "Ccedilla"));
        result.init_block(0o310, vec!(
            "Egrave", "Eacute", "Ecircumflex", "Edieresis",
            "Igrave", "Iacute", "Icircumflex", "Idieresis"));
        result.init_block(0o320, vec!(
            "Eth", "Ntilde", "Ograve", "Oacute",
            "Ocircumflex", "Otilde", "Odieresis", "multiply"));
        result.init_block(0o330, vec!(
            "Oslash", "Ugrave", "Uacute", "Ucircumflex",
            "Udieresis", "Yacute", "Thorn", "germandbls"));
        result.init_block(0o340, vec!(
            "agrave", "aacute", "acircumflex", "atilde",
            "adieresis", "aring", "ae", "ccedilla"));
        result.init_block(0o350, vec!(
            "egrave", "eacute", "ecircumflex", "edieresis",
            "igrave", "iacute", "icircumflex", "idieresis"));
        result.init_block(0o360, vec!(
            "eth", "ntilde", "ograve", "oacute",
            "ocircumflex", "otilde", "odieresis", "divide"));
        result.init_block(0o370, vec!(
            "oslash", "ugrave", "uacute", "ucircumflex",
            "udieresis", "yacute", "thorn", "ydieresis"));
        result
    };

    pub static ref SYMBOL_ENCODING: Encoding = {
        let mut codes = BTreeMap::new();
        let mut names = BTreeMap::new();
        for code in 32..255 {
            codes.insert(code as char, code);
        }
        {
            let mut enc = |ch: char, name: &'static str, code: u8| {
                codes.insert(ch, code);
                names.insert(name, code);
            };
            enc('Α', "Alpha",          0o101);
            enc('Β', "Beta",           0o102);
            enc('Χ', "Chi",            0o103);
            enc('Δ', "Delta",          0o104);
            enc('Ε', "Epsilon",        0o105);
            enc('Η', "Eta",            0o110);
            enc('€', "Euro",           0o240);
            enc('Γ', "Gamma",          0o107);
            enc('ℑ', "Ifraktur",       0o301);
            enc('Ι', "Iota",           0o111);
            enc('Κ', "Kappa",          0o113);
            enc('Λ', "Lambda",         0o114);
            enc('Μ', "Mu",             0o115);
            enc('Ν', "Nu",             0o116);
            enc('Ω', "Omega",          0o127);
            enc('Ο', "Omicron",        0o117);
            enc('Φ', "Phi",            0o106);
            enc('Π', "Pi",             0o120);
            enc('Ψ', "Psi",            0o131);
            enc('ℜ', "Rfraktur",       0o302);
            enc('Ρ', "Rho",            0o122);
            enc('Σ', "Sigma",          0o123);
            enc('Τ', "Tau",            0o124);
            enc('Θ', "Theta",          0o121);
            enc('Υ', "Upsilon",        0o125);
            enc('ϒ', "Upsilon1",       0o241);
            enc('Ξ', "Xi",             0o130);
            enc('Ζ', "Zeta",           0o132);
            enc('ℵ', "aleph",          0o141);
            enc('α', "alpha",          0o141);
            enc('&', "ampersand",      0o046);
            enc('∠', "angle",          0o320);
            enc('〈', "angleleft",      0o341);
            enc('〉', "angleright",     0o361);
            enc('≈', "approxequal",    0o273);
            enc('↔', "arrowboth",      0o253);
            enc('⇔', "arrowdblboth",   0o333);
            enc('⇓', "arrowdbldown",   0o337);
            enc('⇐', "arrowdblleft",   0o334);
            enc('⇒', "arrowdblright",  0o336);
            enc('⇑', "arrowdblup",     0o335);
            enc('↓', "arrowdown",      0o257);
            enc('\u{23af}', "arrowhorizex", 0o276);
            enc('←', "arrowleft",      0o254);
            enc('→', "arrowright",     0o256);
            enc('↑', "arrowup",        0o255);
            enc('\u{23d0}', "arrowvertex", 0o275);
            enc('*', "asteriskmath",   0o052);
            enc('|', "bar",            0o175);
            enc('β', "beta",           0o142);
            enc('{', "braceleft",      0o173);
            enc('}', "braceright",     0o175);
            enc('⎧', "bracelefttp",    0o354);
            enc('⎨', "braceleftmid",   0o355);
            enc('⎩', "braceleftbt",    0o356);
            enc('⎫', "bracerighttp",   0o374);
            enc('⎬', "bracerightmid",  0o375);
            enc('⎭', "bracerightbt",   0o376);
            enc('⎪', "braceex",        0o357);
            enc('[', "bracketleft",    0o133);
            enc(']', "bracketright",   0o135);
            enc('⎡', "bracketlefttp",  0o351);
            enc('⎢', "bracketleftex",  0o352);
            enc('⎣', "bracketleftbt",  0o353);
            enc('⎤', "bracketrighttp", 0o371);
            enc('⎥', "bracketrightex", 0o372);
            enc('⎦', "bracketrightbt", 0o373);
            enc('•', "bullet",         0o267);
            enc('↵', "carriagereturn", 0o277);
            enc('χ', "chi",            0o143);
            enc('⊗', "circlemultiply", 0o304);
            enc('⊕', "circleplus",     0o305);
            enc('♣', "club",           0o247);
            enc(':', "colon",          0o072);
            enc(',', "comma",          0o054);
            enc('≅', "congruent",      0o100);
            // NOTE: copyrightsans and copyrightserif is a single unicode point
            enc('©', "copyrightsans",  0o343);
            enc('©', "copyrightserif", 0o323);
            enc('°', "degree",         0o260);
            enc('δ', "delta",          0o144);
            enc('♦', "diamond",        0o250);
            enc('÷', "divide",         0o270);
            enc('⋅', "dotmath",        0o327);
            enc('8', "eight",          0o070);
            enc('∈', "element",        0o316); // NOTE: and ∊ ?
            enc('…', "ellipsis",       0o274);
            enc('∅', "emptyset",       0o306);
            enc('ε', "epsilon",        0o145);
            enc('=', "equal",          0o075);
            enc('≡', "equivalence",    0o272);
            enc('η', "eta",            0o150);
            enc('!', "exclam",         0o041);
            enc('∃', "existential",    0o044);
            enc('5', "five",           0o065);
            enc('ƒ', "florin",         0o246);
            enc('4', "four",           0o064);
            enc('⁄', "fraction",       0o244);
            enc('γ', "gamma",          0o147);
            enc('∇', "gradient",       0o321);
            enc('>', "greater",        0o076);
            enc('≥', "greaterequal",   0o263);
            enc('♥', "heart",          0o251);
            enc('∞', "infinity",       0o245);
            enc('∫', "integral",       0o362);
            enc('⌠', "integraltp",     0o363);
            enc('⎮', "integralex",     0o364);
            enc('⌡', "integralbt",     0o365);
            enc('∩', "intersection",   0o307);
            enc('ι', "iota",           0o151);
            enc('κ', "kappa",          0o153);
            enc('λ', "lambda",         0o154);
            enc('<', "less",           0o074);
            enc('≤', "lessequal",      0o243);
            enc('∧', "logicaland",     0o331);
            enc('¬', "logicalnot",     0o330);
            enc('∨', "logicalor",      0o332);
            enc('◊', "lozenge",        0o340);
            enc('-', "minus",          0o055);
            enc('\u{2032}', "minute",  0o242); // prime / minutes / feet
            enc('μ', "mu",             0o155);
            enc('×', "multiply",       0o264); // small and large in unicode
            enc('⨯', "multiply",       0o264); // only one in symbol
            enc('9', "nine",           0o071);
            enc('∉', "notelement",     0o317);
            enc('≠', "notequal",       0o271);
            enc('⊄', "notsubset",      0o313);
            enc('ν', "nu",             0o156);
            enc('#', "numbersign",     0o043);
            enc('ω', "omega",          0o167);
            enc('ϖ', "omega1",         0o166);
            enc('ο', "omicron",        0o157);
            enc('1', "one",            0o060);
            enc('(', "parenleft",      0o050);
            enc(')', "parenright",     0o051);
            enc('⎛', "parenlefttp",    0o346);
            enc('⎜', "parenleftex",    0o347);
            enc('⎝', "parenleftbt",    0o350);
            enc('⎞', "parenrighttp",   0o366);
            enc('⎟', "parenrightex",   0o367);
            enc('⎠', "parenrightbt",   0o360);
            enc('∂', "partialdiff",    0o266);
            enc('%', "percent",        0o045);
            enc('.', "period",         0o056);
            enc('⟂', "perpendicular",  0o136);
            enc('ɸ', "phi",            0o146);
            enc('φ', "phi1",           0o152);
            enc('π', "pi",             0o160);
            enc('+', "plus",           0o053);
            enc('±', "plusminus",      0o261);
            enc('∏', "product",        0o325);
            enc('⊂', "propersubset",   0o314);
            enc('⊃', "propersuperset", 0o311);
            enc('∝', "proportional",   0o265);
            enc('ψ', "psi",            0o171);
            enc('?', "question",       0o077);
            enc('√', "radical",        0o326);
            enc('⎺', "radicalex",      0o140); // Very approximate unicode
            enc('⊆', "reflexsubset",   0o315);
            enc('⊇', "reflexsuperset", 0o312);
            enc('®', "registersans",   0o342);
            enc('®', "registerserif",  0o322); // NOTE No distinct unicode?
            enc('ρ', "rho",            0o162);
            enc('\u{2033}', "second",  0o262); // Double prime/seconds/inches
            enc(';', "semicolon",      0o073);
            enc('7', "seven",          0o067);
            enc('σ', "sigma",          0o163);
            enc('ς', "sigma1",         0o126);
            enc('∼', "similar",        0o176);
            enc('6', "six",            0o066);
            enc('/', "slash",          0o057);
            enc(' ', "space",          0o040);
            enc('♠', "spade",          0o252);
            enc('∋', "suchthat",       0o047);
            enc('∑', "summation",      0o345);
            enc('τ', "tau",            0o164);
            enc('∴', "therefore",      0o134);
            enc('θ', "theta",          0o161);
            enc('ϑ', "theta1",         0o112);
            enc('3', "three",          0o063);
            enc('™', "trademarksans",  0o344);
            enc('™', "trademarkserif", 0o324); // NOTE No distinct unicode?
            enc('2', "two",            0o062);
            enc('_', "underscore",     0o137);
            enc('∪', "union",          0o310);
            enc('∀', "universal",      0o042);
            enc('υ', "upsilon",        0o165);
            enc('℘', "weierstrass",    0o303); // Maybe not correct unicode?
            enc('ξ', "xi",             0o170);
            enc('0', "zero",           0o060);
            enc('ζ', "zeta",           0o172);
        }
        Encoding {
            name: "SymbolEncoding".to_string(),
            name_to_code: names,
            unicode_to_code: codes
        }
    };
}

#[test]
fn test_get_winansi_points() {
    let ref enc = WIN_ANSI_ENCODING;
    assert_eq!(Some('A' as u8), enc.get_code("A"));
    assert_eq!(Some('Z' as u8), enc.get_code("Z"));
    assert_eq!(Some('a' as u8), enc.get_code("a"));
    assert_eq!(Some('z' as u8), enc.get_code("z"));
    assert_eq!(Some(' ' as u8), enc.get_code("space"));
    assert_eq!(Some('&' as u8), enc.get_code("ampersand"));
}
