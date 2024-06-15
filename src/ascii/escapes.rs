use std::io::{self, Write, Read};
use crate::logger::log_message;

const ESC_CHAR: char= '\x1b';
const CSI_CHAR: char= '[';
macro_rules! esc {
    ($name:literal) => {
        {format!("{}{}",ESC_CHAR, $name)}
    };
    ($name:expr) => {
        {format!("{}{}", ESC_CHAR, $name)}
    };
}
macro_rules! escC {
    ($name:literal) => {
        {format!("{}{}{}", ESC_CHAR, CSI_CHAR, $name)}
    };
    ($name:expr) => {
        {format!("{}{}{}", ESC_CHAR, CSI_CHAR, $name)}
    };
}
pub trait ParsableSequence<T> {
    fn parse_sequence<I>(chars: &mut std::iter::Peekable<I>) -> Vec<T> where I: Iterator<Item = char>;
    fn parse_writer(reader: &mut Box<dyn Read + Send>) -> Vec<T>;
}

#[derive(Debug)]
pub enum Sequence {
    Text(char),
    Escape(Vec<Escape>),
}

impl ParsableSequence<Sequence> for Sequence{
    fn parse_sequence<I>(chars: &mut std::iter::Peekable<I>) -> Vec<Sequence> where I: Iterator<Item = char> {
        let mut parsed_seq = Vec::new();

        while let Some(&c) = chars.peek() {
            if c as char == ESC_CHAR {
                let escapes = Escape::parse_sequence(chars);
                if escapes.len()>0 {
                    parsed_seq.push(Sequence::Escape(escapes));
                }
            } else {
                parsed_seq.push(Sequence::Text(c));
            }
            chars.next();
        }

        parsed_seq
    }

    fn parse_writer(reader: &mut Box<dyn Read + Send>)  -> Vec<Sequence>{
        let mut parsed_seq = Vec::new();
        let mut s: [u8; 1] = [0;1];

        match reader.read_exact(&mut s) {
            Ok(_) => {
                let c = s[0] as char;
                if c as char == ESC_CHAR {
                    let escapes = Escape::parse_writer(reader);
                    if escapes.len()>0 {
                        parsed_seq.push(Sequence::Escape(escapes));
                    }
                } else {
                    parsed_seq.push(Sequence::Text(c));
                }
            },
            Err(_) => {}
        }
        log_message(&format!("{:?}", parsed_seq));

        parsed_seq
    }
}

pub struct EscapeWriter<'a, T: Write> {
    escapes: Vec<Escape>,
    writer: &'a mut T,
    x_offset: u32,
    y_offset: u32
}

impl<'a, T: Write> EscapeWriter<'a, T> {
    pub fn new(writer: &'a mut T) -> Self{
        Self { 
            escapes: Vec::new(), 
            writer,
            x_offset: 0,
            y_offset: 0
        }
    }

    pub fn queue(&mut self, escape: Escape) {
        self.escapes.push(escape);
    }

    pub fn flush(&mut self) -> io::Result<()>{
        self.writer.flush()
    }

    pub fn send_all(&mut self) -> io::Result<()>{
        while let Some(escape) = self.escapes.pop() {
            self.writer.write(escape.to_string().as_bytes())?;
        }

        Ok(())
    }

    pub fn send_all_and_flush(&mut self) -> io::Result<()>{
        while let Some(escape) = self.escapes.pop() {
            self.writer.write(escape.to_string().as_bytes())?;
        }
        self.writer.flush()?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum Escape {
    ResetAllModes,   // 0m
    ZeroCursor,      // H
    MoveCursorTo((u32, u32)),    // line;colH || line;colf
    CursorUp(u32),         // #A
    CursorMoveOneLineUp,   // M
    CursorDown(u32),       // #B
    CursorRight(u32),      // #C
    CursorLeft(u32),       // #D
    CursorToNextLineStart(u32),   // #E
    CursorToPastLineStart(u32),   // #F
    ClearInDisplay,               // J
    ClearDisplayUntilScreenEnd,   // 0J
    ClearDisplayUntilScreenStart, // 1J
    ClearAll,                     // 2J
    EraseSavedLine,               // 3J
    EraseInLine,                  // K
    EraseFromCursorToEnd,         // 0K
    EraseFromCursorToStart,       // 1K
    EraseLine,                    // 2K
    CursorToCol(u32),   // #G
    SetBold,         // 1m
    SetDim,          // 2m
    SetItalic,       // 3m
    SetUnderline,    // 4m
    SetBlinking,     // 5m
    SetInverse,      // 7m
    SetHidden,       // 8m
    SetStrikethrough,// 9m
    SetForgroundBlack,     // 30m
    SetBackgroundBlack,    // 40m
    SetForgroundRed,       // 31m
    SetBackgroundRed,      // 41m
    SetForgroundGreen,     // 32m
    SetBackgroundGreen,    // 42m
    SetForgroundYellow,    // 33m
    SetBackgroundYellow,   // 43m
    SetForgroundBlue,      // 34m
    SetBackgroundBlue,     // 44m
    SetForgroundMagenta,   // 35m
    SetBackgroundMagenta,  // 45m
    SetForgroundCyan,      // 36m
    SetBackgroundCyan,     // 46m
    SetForgroundWhite,     // 37m
    SetBackgroundWhite,    // 47m
    SetForgroundDefault,   // 39m
    SetBackgroundDefault,  // 49m
    SetForgroundBriBlack,  // 90m
    SetBackgroundBriBlack,  // 100m
    SetForgroundBriRed,     // 91m
    SetBackgroundBriRed,    // 101m
    SetForgroundBriGreen,   // 92m
    SetBackgroundBriGreen,  // 102m
    SetForgroundBriYellow,  // 93m
    SetBackgroundBriYellow, // 103m
    SetForgroundBriBlue,    // 94m
    SetBackgroundBriBlue,   // 104m
    SetForgroundBriMagenta, // 95m
    SetBackgroundBriMagenta,// 105m
    SetForgroundBriCyan,    // 96m
    SetBackgroundBriCyan,   // 106m
    SetForgroundBriWhite,   // 97m
    SetBackgroundBriWhite,  // 107m
    SetForgroundCustomColor(u8), // 38;5;{id}m
    SetBackgroundCustomColor(u8),// 48;5;{id}m
    ResetBold,         // 22m
    ResetDim,          // 22m
    ResetItalic,       // 23m
    ResetUnderline,    // 24m
    ResetBlinking,     // 25m
    ResetInverse,      // 27m
    ResetHidden,       // 28m
    ResetStrikethrough,// 29m
    SaveCursorPos,
    RestoreCursorPos,
    RequestCursorPos,
    Set40_25MonoScreen,        //=0h
    Set40_25ColorScreen,       //=1h
    Set80_25MonoScreen,        //=2h
    Set80_25ColorScreen,       //=3h
    Set320_2004ColorScreen,    //=4h
    Set320_200MonoScreen,      //=5h
    Set640_200MonoScreen,      //=6h
    EnableLineWrap,            //=7h
    Set320_200ColorScreen,     //=13h
    Set640_200ColorScreen,     //=14h
    Set640_350MonoScreen,      //=15h
    Set640_350ColorScreen,     //=16h
    Set640_480MonoScreen,      //=17h
    Set640_480ColorScreen,     //=18h
    Set320_200ColorScreen256,  //=19h
    ResetScreenSet(u8),        //={val}l
    SetCursorVisible,          //?25h
    SetCursorInvisible,        //?25l
    RestoreScreen,             //?47h
    SaveScreen,                //?47l
    EnterAltScreen,           //?1049h
    ExitAltScreen,          //?1049l
}

impl Escape {
    pub fn send<T>(&self, writer: &mut T) -> io::Result<usize> where T: Write {
        writer.write(self.to_string().as_bytes())
    }
    pub fn to_string(&self) -> String{
        match self {
            Escape::ResetAllModes                => {escC!("0m")},   // 0m
            Escape::ZeroCursor                   => {escC!("H")},      // H
            Escape::MoveCursorTo((line,col))     => { escC!(format!("{};{}H", line, col)) },    // line;colH || line;colf
            Escape::CursorUp(i)                  => {escC!(format!("{}A",i))},         // #A
            Escape::CursorMoveOneLineUp          => {esc!("M")},   // M
            Escape::CursorDown(i)                => {escC!(format!("{}B",i))},         // #B
            Escape::CursorRight(i)               => {escC!(format!("{}C",i))},         // #C
            Escape::CursorLeft(i)                => {escC!(format!("{}D",i))},         // #D
            Escape::CursorToNextLineStart(i)     => {escC!(format!("{}E",i))},         // #E
            Escape::CursorToPastLineStart(i)     => {escC!(format!("{}F",i))},         // #F
            Escape::ClearInDisplay               => {escC!("J")},               // J
            Escape::ClearDisplayUntilScreenEnd   => {escC!("0J")},   // 0J
            Escape::ClearDisplayUntilScreenStart => {escC!("1J")}, // 1J
            Escape::ClearAll                     => {escC!("2J")},                     // 2J
            Escape::EraseSavedLine               => {escC!("3J")},               // 3J
            Escape::EraseInLine                  => {escC!("K")},                  // K
            Escape::EraseFromCursorToEnd         => {escC!("0K")},         // 0K
            Escape::EraseFromCursorToStart       => {escC!("1K")},       // 1K
            Escape::EraseLine                    => {escC!("2K")},                    // 2K
            Escape::CursorToCol(i)               => {escC!(format!("{}G", i))},   // #G
            Escape::SetBold                      => {escC!("1m")},         // 1m
            Escape::SetDim                       => {escC!("2m")},          // 2m
            Escape::SetItalic                    => {escC!("3m")},       // 3m
            Escape::SetUnderline                 => {escC!("4m")},    // 4m
            Escape::SetBlinking                  => {escC!("5m")},     // 5m
            Escape::SetInverse                   => {escC!("7m")},      // 7m
            Escape::SetHidden                    => {escC!("8m")},       // 8m
            Escape::SetStrikethrough             => {escC!("9m")},// 9m
            Escape::SetForgroundBlack            => {escC!("30m")},     // 30m
            Escape::SetBackgroundBlack           => {escC!("40m")},    // 40m
            Escape::SetForgroundRed              => {escC!("31m")},       // 31m
            Escape::SetBackgroundRed             => {escC!("41m")},      // 41m
            Escape::SetForgroundGreen            => {escC!("32m")},     // 32m
            Escape::SetBackgroundGreen           => {escC!("42m")},    // 42m
            Escape::SetForgroundYellow           => {escC!("33m")},    // 33m
            Escape::SetBackgroundYellow          => {escC!("43m")},   // 43m
            Escape::SetForgroundBlue             => {escC!("34m")},      // 34m
            Escape::SetBackgroundBlue            => {escC!("44m")},     // 44m
            Escape::SetForgroundMagenta          => {escC!("35m")},   // 35m
            Escape::SetBackgroundMagenta         => {escC!("45m")},  // 45m
            Escape::SetForgroundCyan             => {escC!("36m")},      // 36m
            Escape::SetBackgroundCyan            => {escC!("46m")},     // 46m
            Escape::SetForgroundWhite            => {escC!("37m")},     // 37m
            Escape::SetBackgroundWhite           => {escC!("47m")},    // 47m
            Escape::SetForgroundDefault          => {escC!("39m")},   // 39m
            Escape::SetBackgroundDefault         => {escC!("49m")},  // 49m
            Escape::SetForgroundBriBlack         => {escC!("90m")},  // 90m
            Escape::SetBackgroundBriBlack        => {escC!("100m")},  // 100m
            Escape::SetForgroundBriRed           => {escC!("91m")},     // 91m
            Escape::SetBackgroundBriRed          => {escC!("101m")},    // 101m
            Escape::SetForgroundBriGreen         => {escC!("92m")},   // 92m
            Escape::SetBackgroundBriGreen        => {escC!("102m")},  // 102m
            Escape::SetForgroundBriYellow        => {escC!("93m")},  // 93m
            Escape::SetBackgroundBriYellow       => {escC!("103m")}, // 103m
            Escape::SetForgroundBriBlue          => {escC!("94m")},    // 94m
            Escape::SetBackgroundBriBlue         => {escC!("104m")},   // 104m
            Escape::SetForgroundBriMagenta       => {escC!("95m")}, // 95m
            Escape::SetBackgroundBriMagenta      => {escC!("105m")},// 105m
            Escape::SetForgroundBriCyan          => {escC!("96m")},    // 96m
            Escape::SetBackgroundBriCyan         => {escC!("106m")},   // 106m
            Escape::SetForgroundBriWhite         => {escC!("97m")},   // 97m
            Escape::SetBackgroundBriWhite        => {escC!("107m")},  // 107m
            Escape::SetForgroundCustomColor(i)   => {escC!(format!("38;5;{}m",i))}, // 38;5;{id}m
            Escape::SetBackgroundCustomColor(i)  => {escC!(format!("48;5;{}m",i))},// 48;5;{id}m
            Escape::ResetBold                    => {escC!("22m")},         // 22m
            Escape::ResetDim                     => {escC!("22m")},          // 22m
            Escape::ResetItalic                  => {escC!("23m")},       // 23m
            Escape::ResetUnderline               => {escC!("24m")},    // 24m
            Escape::ResetBlinking                => {escC!("25m")},     // 25m
            Escape::ResetInverse                 => {escC!("27m")},      // 27m
            Escape::ResetHidden                  => {escC!("28m")},       // 28m
            Escape::ResetStrikethrough           => {escC!("29m")},// 29m
            Escape::SaveCursorPos                => {esc!("7")},
            Escape::RestoreCursorPos             => {esc!("8")},
            Escape::RequestCursorPos             => {escC!("6n")},
            Escape::Set40_25MonoScreen           => {escC!("=0h")},        //=0h
            Escape::Set40_25ColorScreen          => {escC!("=1h")},       //=1h
            Escape::Set80_25MonoScreen           => {escC!("=2h")},        //=2h
            Escape::Set80_25ColorScreen          => {escC!("=3h")},       //=3h
            Escape::Set320_2004ColorScreen       => {escC!("=4h")},    //=4h
            Escape::Set320_200MonoScreen         => {escC!("=5h")},      //=5h
            Escape::Set640_200MonoScreen         => {escC!("=6h")},      //=6h
            Escape::EnableLineWrap               => {escC!("=7h")},            //=7h
            Escape::Set320_200ColorScreen        => {escC!("=13h")},     //=13h
            Escape::Set640_200ColorScreen        => {escC!("=14h")},     //=14h
            Escape::Set640_350MonoScreen         => {escC!("=15h")},      //=15h
            Escape::Set640_350ColorScreen        => {escC!("=16h")},     //=16h
            Escape::Set640_480MonoScreen         => {escC!("=17h")},      //=17h
            Escape::Set640_480ColorScreen        => {escC!("=18h")},     //=18h
            Escape::Set320_200ColorScreen256     => {escC!("=19h")},  //=19h
            Escape::ResetScreenSet(i)            => {escC!(format!("={}l",i))},        //={val}l
            Escape::SetCursorVisible             => {escC!("?25h")},          //?25h
            Escape::SetCursorInvisible           => {escC!("?25l")},        //?25l
            Escape::RestoreScreen                => {escC!("?47h")},             //?47h
            Escape::SaveScreen                   => {escC!("?47l")},                //?47l
            Escape::EnterAltScreen               => {escC!("?1049h")},           //?1049h
            Escape::ExitAltScreen                => {escC!("?1049l")},          //?1049l
        }
    }
}

impl ParsableSequence<Escape> for Escape{
    fn parse_sequence<I>(chars: &mut std::iter::Peekable<I>) -> Vec<Escape> where I: Iterator<Item = char> {
        let mut start_long_esc = false;
        let mut escapes = Vec::new();
        while let Some(&c) = chars.peek() {
            let c = c as char;
            if c=='[' {
                start_long_esc = true;
                break;
            } else {
                match c {
                    '7' => {escapes.push(Escape::SaveCursorPos)},
                    '8' => {escapes.push(Escape::RestoreCursorPos)},
                    'M' => {escapes.push(Escape::CursorMoveOneLineUp)},
                    _ => {}
                }
            }
            chars.next();
        }

        if start_long_esc {
            escapes = parse_long_seq(escapes, chars);
        }

        escapes
    }

    fn parse_writer(reader: &mut Box<dyn Read + Send>) -> Vec<Escape>{
        let mut escapes = Vec::new();
        let mut s: [u8; 1] = [0;1];
        match reader.read_exact(&mut s) {
            Ok(_) => {
                let c = s[0] as char;
                match c {
                    '[' => {escapes = parse_long_write(escapes, reader);},
                    '7' => {escapes.push(Escape::SaveCursorPos)},
                    '8' => {escapes.push(Escape::RestoreCursorPos)},
                    'M' => {escapes.push(Escape::CursorMoveOneLineUp)},
                    _ => {}
                }
            },
            Err(_) => {}
        }

        escapes
        
    }
}

enum SpecialLongCase {
    NoSpecial,
    ScreenMode,
    PrivateMode
}
fn parse_long_write(mut escapes: Vec<Escape>, reader: &mut Box<dyn Read + Send>) -> Vec<Escape>  {
    let mut number = String::new();
    let mut special_esc_flag = SpecialLongCase::NoSpecial;
    let mut i = 0;
    let mut s: [u8; 1] = [0;1];
    loop {
        match reader.read_exact(&mut s) {
            Ok(_) => {
                let c = s[0] as char;
                log_message(&format!("{}",c));
                if i<1 {
                    if c=='=' {
                        special_esc_flag = SpecialLongCase::ScreenMode;
                    } else if c=='?'{
                        special_esc_flag = SpecialLongCase::PrivateMode;
                    }
                } else {
                    match c {
                        'l' => {
                            match special_esc_flag {
                                SpecialLongCase::ScreenMode => {
                                    if let Ok(number) = number.parse() {
                                        escapes.push(Escape::ResetScreenSet(number));
                                    }                                
                                },
                                SpecialLongCase::PrivateMode => {
                                    match number.as_str() {
                                        "25" => {escapes.push(Escape::SetCursorInvisible); },
                                        "47" => {escapes.push(Escape::RestoreScreen); },
                                        "1049" => {escapes.push(Escape::ExitAltScreen); },
                                        _ => {}
                                    }
                                },
                                _ => {}
                            }
                            break;
                        }
                        'h' => {
                            match special_esc_flag {
                                SpecialLongCase::ScreenMode => {
                                    match number.as_str() {
                                        "0" => { escapes.push( Escape::Set40_25MonoScreen ) ; },        //=0h
                                        "1" => { escapes.push( Escape::Set40_25ColorScreen ) ; },       //=1h
                                        "2" => { escapes.push( Escape::Set80_25MonoScreen ) ; },        //=2h
                                        "3" => { escapes.push( Escape::Set80_25ColorScreen ) ; },       //=3h
                                        "4" => { escapes.push( Escape::Set320_2004ColorScreen ) ; },    //=4h
                                        "5" => { escapes.push( Escape::Set320_200MonoScreen ) ; },      //=5h
                                        "6" => { escapes.push( Escape::Set640_200MonoScreen ) ; },      //=6h
                                        "7" => { escapes.push( Escape::EnableLineWrap) ; },            //=7h
                                        "13" => { escapes.push( Escape::Set320_200ColorScreen ) ; },     //=13h
                                        "14" => { escapes.push( Escape::Set640_200ColorScreen ) ; },     //=14h
                                        "15" => { escapes.push( Escape::Set640_350MonoScreen ) ; },      //=15h
                                        "16" => { escapes.push( Escape::Set640_350ColorScreen ) ; },     //=16h
                                        "17" => { escapes.push( Escape::Set640_480MonoScreen ) ; },      //=17h
                                        "18" => { escapes.push( Escape::Set640_480ColorScreen ) ; },     //=18h
                                        "19" => { escapes.push( Escape::Set320_200ColorScreen256 ) ; },     //=19h
                                        _ => {}
                                    }
                                },
                                SpecialLongCase::PrivateMode => {
                                    match number.as_str() {
                                        "25" => {escapes.push(Escape::SetCursorVisible); },
                                        "47" => {escapes.push(Escape::SaveScreen); },
                                        "1049" => {escapes.push(Escape::EnterAltScreen); },
                                        _ => {}
                                    }
                                },
                                _ => {}
                            }
                            break;
                        },
                        'n' => {
                            if number=="6" {
                                escapes.push(Escape::RequestCursorPos);
                            }
                            break;
                        },
                        'm' => {
                            if number.len()>0 {
                                let numbers = number.split(';');
                                let mut is_custom_color: u8 = 0;
                                let mut rgb = (-1,-1 ,-1);
                                for number in numbers {
                                    if is_custom_color>0 && number=="5"{
                                        if let Ok(number) = number.parse() {
                                            if is_custom_color==1 {
                                                escapes.push(Escape::SetForgroundCustomColor(number));
                                            }else if is_custom_color==2 {
                                                escapes.push(Escape::SetBackgroundCustomColor(number));
                                            } else {
                                                is_custom_color = 0;
                                            }
                                        } else {
                                            is_custom_color = 0;
                                        }
                                    }                       
                                    if is_custom_color>0 && number=="2"{
                                        unimplemented!("Truecolor escapes have not been implemented");
                                    }                       
                                    match number {
                                        "0" => { escapes.push(Escape::ResetAllModes) },
                                        "1" => { escapes.push(Escape::SetBold) },
                                        "2" => { escapes.push(Escape::SetDim) },
                                        "3" => { escapes.push(Escape::SetItalic) },
                                        "4" => { escapes.push(Escape::SetUnderline) },
                                        "5" => { escapes.push(Escape::SetBlinking) },
                                        "7" => { escapes.push(Escape::SetInverse) },
                                        "8" => { escapes.push(Escape::SetHidden) },
                                        "9" => { escapes.push(Escape::SetStrikethrough) },
                                        "22" => { 
                                            escapes.push(Escape::ResetBold);
                                            escapes.push(Escape::ResetDim);
                                        },
                                        "23" => { escapes.push(Escape::ResetItalic) },
                                        "24" => { escapes.push(Escape::ResetUnderline) },
                                        "25" => { escapes.push(Escape::ResetBlinking) },
                                        "27" => { escapes.push(Escape::ResetInverse) },
                                        "28" => { escapes.push(Escape::ResetHidden) },
                                        "29" => { escapes.push(Escape::ResetStrikethrough) },
                                        "30" => { escapes.push(Escape::SetForgroundBlack) },
                                        "40" => { escapes.push(Escape::SetBackgroundBlack) },
                                        "31" => { escapes.push(Escape::SetForgroundRed) },
                                        "41" => { escapes.push(Escape::SetBackgroundRed) },
                                        "32" => { escapes.push(Escape::SetForgroundGreen) },
                                        "42" => { escapes.push(Escape::SetBackgroundGreen) },
                                        "33" => { escapes.push(Escape::SetForgroundYellow) },
                                        "43" => { escapes.push(Escape::SetBackgroundYellow) },
                                        "34" => { escapes.push(Escape::SetForgroundBlue) },
                                        "44" => { escapes.push(Escape::SetBackgroundBlue) },
                                        "35" => { escapes.push(Escape::SetForgroundMagenta) },
                                        "45" => { escapes.push(Escape::SetBackgroundMagenta) },
                                        "36" => { escapes.push(Escape::SetForgroundCyan) },
                                        "46" => { escapes.push(Escape::SetBackgroundCyan) },
                                        "37" => { escapes.push(Escape::SetForgroundWhite) },
                                        "38" => { is_custom_color = 1;},
                                        "48" => { is_custom_color = 2;},
                                        "47" => { escapes.push(Escape::SetBackgroundWhite) },
                                        "39" => { escapes.push(Escape::SetForgroundDefault) },
                                        "49" => { escapes.push(Escape::SetBackgroundDefault) },
                                        "90" => { escapes.push(Escape::SetForgroundBriBlack) },
                                        "100" => { escapes.push(Escape::SetBackgroundBriBlack) },
                                        "91" => { escapes.push(Escape::SetForgroundBriRed) },
                                        "101" => { escapes.push(Escape::SetBackgroundBriRed) },
                                        "92" => { escapes.push(Escape::SetForgroundBriGreen) },
                                        "102" => { escapes.push(Escape::SetBackgroundBriGreen) },
                                        "93" => { escapes.push(Escape::SetForgroundBriYellow) },
                                        "103" => { escapes.push(Escape::SetBackgroundBriYellow) },
                                        "94" => { escapes.push(Escape::SetForgroundBriBlue) },
                                        "104" => { escapes.push(Escape::SetBackgroundBriBlue) },
                                        "95" => { escapes.push(Escape::SetForgroundBriMagenta) },
                                        "105" => { escapes.push(Escape::SetBackgroundBriMagenta) },
                                        "96" => { escapes.push(Escape::SetForgroundBriCyan) },
                                        "106" => { escapes.push(Escape::SetBackgroundBriCyan) },
                                        "97" => { escapes.push(Escape::SetForgroundBriWhite) },
                                        "107" => { escapes.push(Escape::SetBackgroundBriWhite) },
                                        _ => {}
                                    }
                                }
                            }
                            break;
                        },
                        'A' => {
                            if number.len()>0 {
                                if let Ok(number) = number.parse() {
                                    escapes.push(Escape::CursorUp(number));
                                }
                            }
                            break;
                        },
                        'B' => {
                            if number.len()>0 {
                                if let Ok(number) = number.parse() {
                                    escapes.push(Escape::CursorDown(number));
                                }
                            }
                            break;
                        },
                        'C' => {
                            if number.len()>0 {
                                if let Ok(number) = number.parse() {
                                    escapes.push(Escape::CursorRight(number));
                                }
                            }
                            break;
                        },
                        'D' => {
                            if number.len()>0 {
                                if let Ok(number) = number.parse() {
                                    escapes.push(Escape::CursorLeft(number));
                                }
                            }
                            break;
                        },
                        'E' => {
                            if number.len()>0 {
                                if let Ok(number) = number.parse() {
                                    escapes.push(Escape::CursorToNextLineStart(number));
                                }
                            }
                            break;
                        },
                        'F' => {
                            if number.len()>0 {
                                if let Ok(number) = number.parse() {
                                    escapes.push(Escape::CursorToPastLineStart(number));
                                }
                            }
                            break;
                        },
                        'G' => {
                            if number.len()>0 {
                                if let Ok(number) = number.parse() {
                                    escapes.push(Escape::CursorToCol(number));
                                }
                            }
                            break;
                        },
                        'H' => {
                            if number.len()==2 {
                                let numbers = number.split_once(';');
                                if let Some((line, col)) = numbers {
                                    if let (Ok(line), Ok(col)) = (line.trim().parse(), col.trim().parse()) {
                                        escapes.push(Escape::MoveCursorTo((line, col)));
                                    }
                                }
                            } else if number.len()==0 {
                                escapes.push(Escape::ZeroCursor);
                            }
                            break;
                        },
                        'J' => {
                            if number.len()==0 {
                                escapes.push(Escape::ClearInDisplay);
                            } else if number=="0" {
                                escapes.push(Escape::ClearDisplayUntilScreenEnd);
                            } else if number=="1" {
                                escapes.push(Escape::ClearDisplayUntilScreenStart);
                            } else if number=="2" {
                                escapes.push(Escape::ClearAll);
                            } else if number=="3" {
                                escapes.push(Escape::EraseSavedLine);
                            }
                            break;
                        },
                        'K' => {
                            if number.len()==0 {
                                escapes.push(Escape::EraseInLine);
                            } else if number=="0" {
                                escapes.push(Escape::EraseFromCursorToEnd);
                            } else if number=="1" {
                                escapes.push(Escape::EraseFromCursorToStart);
                            } else if number=="2" {
                                escapes.push(Escape::EraseLine);
                            }
                            break;
                        },
                        's' => {
                            if number.len()==0 {
                                escapes.push(Escape::SaveCursorPos);
                            }
                            break;
                        },
                        'u' => {
                            if number.len()==0 {
                                escapes.push(Escape::RestoreCursorPos);
                            }
                            break;
                        },
                        _ => {
                            if c.is_numeric() || c==';' {
                                number.push(c);
                            }
                        }
                    }
                }
                i += 1;
            },

            Err(e) => {
                println!("{:?}",e);
                break;}
        }
    }

    escapes
}

fn parse_long_seq<I>(mut escapes: Vec<Escape>, chars: &mut std::iter::Peekable<I>) -> Vec<Escape> where I: Iterator<Item = char> {
    let mut number = String::new();
    let mut special_esc_flag = SpecialLongCase::NoSpecial;
    let mut i = 0;
    while let Some(&c) = chars.peek() {
        if i<1 {
            if c=='=' {
                special_esc_flag = SpecialLongCase::ScreenMode;
            } else if c=='?'{
                special_esc_flag = SpecialLongCase::PrivateMode;
            }
        } else {
            match c {
                'l' => {
                    match special_esc_flag {
                        SpecialLongCase::ScreenMode => {
                            if let Ok(number) = number.parse() {
                                escapes.push(Escape::ResetScreenSet(number));
                            }
                        },
                        SpecialLongCase::PrivateMode => {
                            match number.as_str() {
                                "25" => {escapes.push(Escape::SetCursorInvisible)},
                                "47" => {escapes.push(Escape::RestoreScreen)},
                                "1049" => {escapes.push(Escape::ExitAltScreen)},
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                }
                'h' => {
                    match special_esc_flag {
                        SpecialLongCase::ScreenMode => {
                            match number.as_str() {
                                "0" => { escapes.push( Escape::Set40_25MonoScreen ) },        //=0h
                                "1" => { escapes.push( Escape::Set40_25ColorScreen ) },       //=1h
                                "2" => { escapes.push( Escape::Set80_25MonoScreen ) },        //=2h
                                "3" => { escapes.push( Escape::Set80_25ColorScreen ) },       //=3h
                                "4" => { escapes.push( Escape::Set320_2004ColorScreen ) },    //=4h
                                "5" => { escapes.push( Escape::Set320_200MonoScreen ) },      //=5h
                                "6" => { escapes.push( Escape::Set640_200MonoScreen ) },      //=6h
                                "7" => { escapes.push( Escape::EnableLineWrap) },            //=7h
                                "13" => { escapes.push( Escape::Set320_200ColorScreen ) },     //=13h
                                "14" => { escapes.push( Escape::Set640_200ColorScreen ) },     //=14h
                                "15" => { escapes.push( Escape::Set640_350MonoScreen ) },      //=15h
                                "16" => { escapes.push( Escape::Set640_350ColorScreen ) },     //=16h
                                "17" => { escapes.push( Escape::Set640_480MonoScreen ) },      //=17h
                                "18" => { escapes.push( Escape::Set640_480ColorScreen ) },     //=18h
                                "19" => { escapes.push( Escape::Set320_200ColorScreen256 ) },     //=19h
                                _ => {}
                            }
                        },
                        SpecialLongCase::PrivateMode => {
                            match number.as_str() {
                                "25" => {escapes.push(Escape::SetCursorVisible)},
                                "47" => {escapes.push(Escape::SaveScreen)},
                                "1049" => {escapes.push(Escape::EnterAltScreen)},
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                },
                'n' => {
                    if number=="6" {
                        escapes.push(Escape::RequestCursorPos);
                    }
                },
                'm' => {
                    if number.len()>0 {
                        let numbers = number.split(';');
                        let mut is_custom_color: u8 = 0;
                        let mut rgb = (-1,-1 ,-1);
                        for number in numbers {
                            if is_custom_color>0 && number=="5"{
                                if let Ok(number) = number.parse() {
                                    if is_custom_color==1 {
                                        escapes.push(Escape::SetForgroundCustomColor(number));
                                        break;                            
                                    }else if is_custom_color==2 {
                                        escapes.push(Escape::SetBackgroundCustomColor(number));
                                        break;                            
                                    } else {
                                        is_custom_color = 0;
                                    }
                                } else {
                                    is_custom_color = 0;
                                }
                            }                       
                            if is_custom_color>0 && number=="2"{
                                unimplemented!("Truecolor escapes have not been implemented");
                            }                       
                            match number {
                                "0" => { escapes.push(Escape::ResetAllModes) },
                                "1" => { escapes.push(Escape::SetBold) },
                                "2" => { escapes.push(Escape::SetDim) },
                                "3" => { escapes.push(Escape::SetItalic) },
                                "4" => { escapes.push(Escape::SetUnderline) },
                                "5" => { escapes.push(Escape::SetBlinking) },
                                "7" => { escapes.push(Escape::SetInverse) },
                                "8" => { escapes.push(Escape::SetHidden) },
                                "9" => { escapes.push(Escape::SetStrikethrough) },
                                "22" => { 
                                    escapes.push(Escape::ResetBold);
                                    escapes.push(Escape::ResetDim);
                                },
                                "23" => { escapes.push(Escape::ResetItalic) },
                                "24" => { escapes.push(Escape::ResetUnderline) },
                                "25" => { escapes.push(Escape::ResetBlinking) },
                                "27" => { escapes.push(Escape::ResetInverse) },
                                "28" => { escapes.push(Escape::ResetHidden) },
                                "29" => { escapes.push(Escape::ResetStrikethrough) },
                                "30" => { escapes.push(Escape::SetForgroundBlack) },
                                "40" => { escapes.push(Escape::SetBackgroundBlack) },
                                "31" => { escapes.push(Escape::SetForgroundRed) },
                                "41" => { escapes.push(Escape::SetBackgroundRed) },
                                "32" => { escapes.push(Escape::SetForgroundGreen) },
                                "42" => { escapes.push(Escape::SetBackgroundGreen) },
                                "33" => { escapes.push(Escape::SetForgroundYellow) },
                                "43" => { escapes.push(Escape::SetBackgroundYellow) },
                                "34" => { escapes.push(Escape::SetForgroundBlue) },
                                "44" => { escapes.push(Escape::SetBackgroundBlue) },
                                "35" => { escapes.push(Escape::SetForgroundMagenta) },
                                "45" => { escapes.push(Escape::SetBackgroundMagenta) },
                                "36" => { escapes.push(Escape::SetForgroundCyan) },
                                "46" => { escapes.push(Escape::SetBackgroundCyan) },
                                "37" => { escapes.push(Escape::SetForgroundWhite) },
                                "38" => { is_custom_color = 1;},
                                "48" => { is_custom_color = 2;},
                                "47" => { escapes.push(Escape::SetBackgroundWhite) },
                                "39" => { escapes.push(Escape::SetForgroundDefault) },
                                "49" => { escapes.push(Escape::SetBackgroundDefault) },
                                "90" => { escapes.push(Escape::SetForgroundBriBlack) },
                                "100" => { escapes.push(Escape::SetBackgroundBriBlack) },
                                "91" => { escapes.push(Escape::SetForgroundBriRed) },
                                "101" => { escapes.push(Escape::SetBackgroundBriRed) },
                                "92" => { escapes.push(Escape::SetForgroundBriGreen) },
                                "102" => { escapes.push(Escape::SetBackgroundBriGreen) },
                                "93" => { escapes.push(Escape::SetForgroundBriYellow) },
                                "103" => { escapes.push(Escape::SetBackgroundBriYellow) },
                                "94" => { escapes.push(Escape::SetForgroundBriBlue) },
                                "104" => { escapes.push(Escape::SetBackgroundBriBlue) },
                                "95" => { escapes.push(Escape::SetForgroundBriMagenta) },
                                "105" => { escapes.push(Escape::SetBackgroundBriMagenta) },
                                "96" => { escapes.push(Escape::SetForgroundBriCyan) },
                                "106" => { escapes.push(Escape::SetBackgroundBriCyan) },
                                "97" => { escapes.push(Escape::SetForgroundBriWhite) },
                                "107" => { escapes.push(Escape::SetBackgroundBriWhite) },
                                _ => {}
                            }
                        }
                        break;
                    }
                },
                'A' => {
                    if number.len()>0 {
                        if let Ok(number) = number.parse() {
                            escapes.push(Escape::CursorUp(number));
                        }
                        break;
                    }
                },
                'B' => {
                    if number.len()>0 {
                        if let Ok(number) = number.parse() {
                            escapes.push(Escape::CursorDown(number));
                        }
                        break;
                    }
                },
                'C' => {
                    if number.len()>0 {
                        if let Ok(number) = number.parse() {
                            escapes.push(Escape::CursorRight(number));
                        }
                        break;
                    }
                },
                'D' => {
                    if number.len()>0 {
                        if let Ok(number) = number.parse() {
                            escapes.push(Escape::CursorLeft(number));
                        }
                        break;
                    }
                },
                'E' => {
                    if number.len()>0 {
                        if let Ok(number) = number.parse() {
                            escapes.push(Escape::CursorToNextLineStart(number));
                        }
                        break;
                    }
                },
                'F' => {
                    if number.len()>0 {
                        if let Ok(number) = number.parse() {
                            escapes.push(Escape::CursorToPastLineStart(number));
                        }
                        break;
                    }
                },
                'G' => {
                    if number.len()>0 {
                        if let Ok(number) = number.parse() {
                            escapes.push(Escape::CursorToCol(number));
                        }
                        break;
                    }
                },
                'H' => {
                    if number.len()==2 {
                        let numbers = number.split_once(';');
                        if let Some((line, col)) = numbers {
                            if let (Ok(line), Ok(col)) = (line.trim().parse(), col.trim().parse()) {
                                escapes.push(Escape::MoveCursorTo((line, col)));
                            }
                        }
                        break;
                    } else if number.len()==0 {
                        escapes.push(Escape::ZeroCursor);
                    }
                },
                'J' => {
                    if number.len()==0 {
                        escapes.push(Escape::ClearInDisplay);
                    } else if number=="0" {
                        escapes.push(Escape::ClearDisplayUntilScreenEnd);
                    } else if number=="1" {
                        escapes.push(Escape::ClearDisplayUntilScreenStart);
                    } else if number=="2" {
                        escapes.push(Escape::ClearAll);
                    } else if number=="3" {
                        escapes.push(Escape::EraseSavedLine);
                    }
                },
                'K' => {
                    if number.len()==0 {
                        escapes.push(Escape::EraseInLine);
                    } else if number=="0" {
                        escapes.push(Escape::EraseFromCursorToEnd);
                    } else if number=="1" {
                        escapes.push(Escape::EraseFromCursorToStart);
                    } else if number=="2" {
                        escapes.push(Escape::EraseLine);
                    }
                },
                's' => {
                    if number.len()==0 {
                        escapes.push(Escape::SaveCursorPos);
                    }
                },
                'u' => {
                    if number.len()==0 {
                        escapes.push(Escape::RestoreCursorPos);
                    }
                },
                _ => {
                    if c.is_numeric() || c==';' {
                        number.push(c);
                    }
                }
            }
        }
        i += 1;
        chars.next();
    }

    escapes
}
