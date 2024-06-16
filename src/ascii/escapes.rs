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
            self.writer.write(&escape.into_bytes())?;
        }

        Ok(())
    }

    pub fn send_all_and_flush(&mut self) -> io::Result<()>{
        while let Some(escape) = self.escapes.pop() {
            self.writer.write(&escape.into_bytes())?;
        }
        self.writer.flush()?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum Escape {
    ResetAllModes,                // 0m
    ZeroCursor,                   // H
    MoveCursorTo((u32, u32)),     // line;colH || line;colf
    CursorUp(u32),                // #A
    CursorMoveOneLineUp,          // M
    CursorDown(u32),              // #B
    CursorRight(u32),             // #C
    CursorLeft(u32),              // #D
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
    CursorToCol(u32),             // #G
    SetBold,                      // 1m
    SetDim,                       // 2m
    SetItalic,                    // 3m
    SetUnderline,                 // 4m
    SetBlinking,                  // 5m
    SetInverse,                   // 7m
    SetHidden,                    // 8m
    SetStrikethrough,             // 9m
    SetForgroundBlack,            // 30m
    SetBackgroundBlack,           // 40m
    SetForgroundRed,              // 31m
    SetBackgroundRed,             // 41m
    SetForgroundGreen,            // 32m
    SetBackgroundGreen,           // 42m
    SetForgroundYellow,           // 33m
    SetBackgroundYellow,          // 43m
    SetForgroundBlue,             // 34m
    SetBackgroundBlue,            // 44m
    SetForgroundMagenta,          // 35m
    SetBackgroundMagenta,         // 45m
    SetForgroundCyan,             // 36m
    SetBackgroundCyan,            // 46m
    SetForgroundWhite,            // 37m
    SetBackgroundWhite,           // 47m
    SetForgroundDefault,          // 39m
    SetBackgroundDefault,         // 49m
    SetForgroundBriBlack,         // 90m
    SetBackgroundBriBlack,        // 100m
    SetForgroundBriRed,           // 91m
    SetBackgroundBriRed,          // 101m
    SetForgroundBriGreen,         // 92m
    SetBackgroundBriGreen,        // 102m
    SetForgroundBriYellow,        // 93m
    SetBackgroundBriYellow,       // 103m
    SetForgroundBriBlue,          // 94m
    SetBackgroundBriBlue,         // 104m
    SetForgroundBriMagenta,       // 95m
    SetBackgroundBriMagenta,      // 105m
    SetForgroundBriCyan,          // 96m
    SetBackgroundBriCyan,         // 106m
    SetForgroundBriWhite,         // 97m
    SetBackgroundBriWhite,        // 107m
    SetForgroundCustomColor(u8),  // 38;5;{id}m
    SetBackgroundCustomColor(u8), // 48;5;{id}m
    ResetBold,                    // 22m
    ResetDim,                     // 22m
    ResetItalic,                  // 23m
    ResetUnderline,               // 24m
    ResetBlinking,                // 25m
    ResetInverse,                 // 27m
    ResetHidden,                  // 28m
    ResetStrikethrough,           // 29m
    SaveCursorPos,
    RestoreCursorPos,
    RequestCursorPos,
    Set40_25MonoScreen,           //=0h
    Set40_25ColorScreen,          //=1h
    Set80_25MonoScreen,           //=2h
    Set80_25ColorScreen,          //=3h
    Set320_2004ColorScreen,       //=4h
    Set320_200MonoScreen,         //=5h
    Set640_200MonoScreen,         //=6h
    EnableLineWrap,               //=7h
    Set320_200ColorScreen,        //=13h
    Set640_200ColorScreen,        //=14h
    Set640_350MonoScreen,         //=15h
    Set640_350ColorScreen,        //=16h
    Set640_480MonoScreen,         //=17h
    Set640_480ColorScreen,        //=18h
    Set320_200ColorScreen256,     //=19h
    ResetScreenSet(u8),           //={val}l
    SetCursorVisible,             //?25h
    SetCursorInvisible,           //?25l
    RestoreScreen,                //?47h
    SaveScreen,                   //?47l
    EnterAltScreen,               //?1049h
    ExitAltScreen,                //?1049l
}

impl Escape {
    pub fn send<T>(&self, writer: &mut T) -> io::Result<usize> where T: Write {
        writer.write(&self.into_bytes())
    }
    pub fn as_static_bytes(&self) -> Option<&'static [u8]> {
        match self {
            Escape::ResetAllModes                => {Some(b"\x1b[0m")},   
            Escape::ZeroCursor                   => {Some(b"\x1b[H")},      
            Escape::CursorMoveOneLineUp          => {Some(b"\x1bM")},   
            Escape::ClearInDisplay               => {Some(b"\x1b[J")},               
            Escape::ClearDisplayUntilScreenEnd   => {Some(b"\x1b[0J")},   
            Escape::ClearDisplayUntilScreenStart => {Some(b"\x1b[1J")}, 
            Escape::ClearAll                     => {Some(b"\x1b[2J")},                     
            Escape::EraseSavedLine               => {Some(b"\x1b[3J")},               
            Escape::EraseInLine                  => {Some(b"\x1b[K")},                  
            Escape::EraseFromCursorToEnd         => {Some(b"\x1b[0K")},         
            Escape::EraseFromCursorToStart       => {Some(b"\x1b[1K")},       
            Escape::EraseLine                    => {Some(b"\x1b[2K")},                    
            Escape::SetBold                      => {Some(b"\x1b[1m")},         
            Escape::SetDim                       => {Some(b"\x1b[2m")},          
            Escape::SetItalic                    => {Some(b"\x1b[3m")},       
            Escape::SetUnderline                 => {Some(b"\x1b[4m")},    
            Escape::SetBlinking                  => {Some(b"\x1b[5m")},     
            Escape::SetInverse                   => {Some(b"\x1b[7m")},      
            Escape::SetHidden                    => {Some(b"\x1b[8m")},       
            Escape::SetStrikethrough             => {Some(b"\x1b[9m")},
            Escape::SetForgroundBlack            => {Some(b"\x1b[30m")},     
            Escape::SetBackgroundBlack           => {Some(b"\x1b[40m")},    
            Escape::SetForgroundRed              => {Some(b"\x1b[31m")},       
            Escape::SetBackgroundRed             => {Some(b"\x1b[41m")},      
            Escape::SetForgroundGreen            => {Some(b"\x1b[32m")},     
            Escape::SetBackgroundGreen           => {Some(b"\x1b[42m")},    
            Escape::SetForgroundYellow           => {Some(b"\x1b[33m")},    
            Escape::SetBackgroundYellow          => {Some(b"\x1b[43m")},   
            Escape::SetForgroundBlue             => {Some(b"\x1b[34m")},      
            Escape::SetBackgroundBlue            => {Some(b"\x1b[44m")},     
            Escape::SetForgroundMagenta          => {Some(b"\x1b[35m")},   
            Escape::SetBackgroundMagenta         => {Some(b"\x1b[45m")},  
            Escape::SetForgroundCyan             => {Some(b"\x1b[36m")},      
            Escape::SetBackgroundCyan            => {Some(b"\x1b[46m")},     
            Escape::SetForgroundWhite            => {Some(b"\x1b[37m")},     
            Escape::SetBackgroundWhite           => {Some(b"\x1b[47m")},    
            Escape::SetForgroundDefault          => {Some(b"\x1b[39m")},   
            Escape::SetBackgroundDefault         => {Some(b"\x1b[49m")},  
            Escape::SetForgroundBriBlack         => {Some(b"\x1b[90m")},  
            Escape::SetBackgroundBriBlack        => {Some(b"\x1b[100m")},  
            Escape::SetForgroundBriRed           => {Some(b"\x1b[91m")},     
            Escape::SetBackgroundBriRed          => {Some(b"\x1b[101m")},    
            Escape::SetForgroundBriGreen         => {Some(b"\x1b[92m")},   
            Escape::SetBackgroundBriGreen        => {Some(b"\x1b[102m")},  
            Escape::SetForgroundBriYellow        => {Some(b"\x1b[93m")},  
            Escape::SetBackgroundBriYellow       => {Some(b"\x1b[103m")}, 
            Escape::SetForgroundBriBlue          => {Some(b"\x1b[94m")},    
            Escape::SetBackgroundBriBlue         => {Some(b"\x1b[104m")},   
            Escape::SetForgroundBriMagenta       => {Some(b"\x1b[95m")}, 
            Escape::SetBackgroundBriMagenta      => {Some(b"\x1b[105m")},
            Escape::SetForgroundBriCyan          => {Some(b"\x1b[96m")},    
            Escape::SetBackgroundBriCyan         => {Some(b"\x1b[106m")},   
            Escape::SetForgroundBriWhite         => {Some(b"\x1b[97m")},   
            Escape::SetBackgroundBriWhite        => {Some(b"\x1b[107m")},  
            Escape::ResetBold                    => {Some(b"\x1b[22m")},         
            Escape::ResetDim                     => {Some(b"\x1b[22m")},          
            Escape::ResetItalic                  => {Some(b"\x1b[23m")},       
            Escape::ResetUnderline               => {Some(b"\x1b[24m")},    
            Escape::ResetBlinking                => {Some(b"\x1b[25m")},     
            Escape::ResetInverse                 => {Some(b"\x1b[27m")},      
            Escape::ResetHidden                  => {Some(b"\x1b[28m")},       
            Escape::ResetStrikethrough           => {Some(b"\x1b[29m")},
            Escape::SaveCursorPos                => {Some(b"\x1b7")},
            Escape::RestoreCursorPos             => {Some(b"\x1b8")},
            Escape::RequestCursorPos             => {Some(b"\x1b[6n")},
            Escape::Set40_25MonoScreen           => {Some(b"\x1b[=0h")},        
            Escape::Set40_25ColorScreen          => {Some(b"\x1b[=1h")},       
            Escape::Set80_25MonoScreen           => {Some(b"\x1b[=2h")},        
            Escape::Set80_25ColorScreen          => {Some(b"\x1b[=3h")},       
            Escape::Set320_2004ColorScreen       => {Some(b"\x1b[=4h")},    
            Escape::Set320_200MonoScreen         => {Some(b"\x1b[=5h")},      
            Escape::Set640_200MonoScreen         => {Some(b"\x1b[=6h")},      
            Escape::EnableLineWrap               => {Some(b"\x1b[=7h")},            
            Escape::Set320_200ColorScreen        => {Some(b"\x1b[=13h")},     
            Escape::Set640_200ColorScreen        => {Some(b"\x1b[=14h")},     
            Escape::Set640_350MonoScreen         => {Some(b"\x1b[=15h")},      
            Escape::Set640_350ColorScreen        => {Some(b"\x1b[=16h")},     
            Escape::Set640_480MonoScreen         => {Some(b"\x1b[=17h")},      
            Escape::Set640_480ColorScreen        => {Some(b"\x1b[=18h")},     
            Escape::Set320_200ColorScreen256     => {Some(b"\x1b[=19h")},  
            Escape::SetCursorVisible             => {Some(b"\x1b[?25h")},          
            Escape::SetCursorInvisible           => {Some(b"\x1b[?25l")},        
            Escape::RestoreScreen                => {Some(b"\x1b[?47h")},             
            Escape::SaveScreen                   => {Some(b"\x1b[?47l")},                
            Escape::EnterAltScreen               => {Some(b"\x1b[?1049h")},           
            Escape::ExitAltScreen                => {Some(b"\x1b[?1049l")},          
            _ => {None}
        }
    }

    pub fn into_bytes(&self) -> Vec<u8>{
        let strs = match self {
            Escape::ResetAllModes                => {escC!("0m")},   
            Escape::ZeroCursor                   => {escC!("H")},      
            Escape::MoveCursorTo((line,col))     => { escC!(format!("{};{}H", line, col)) },    
            Escape::CursorUp(i)                  => {escC!(format!("{}A",i))},         
            Escape::CursorMoveOneLineUp          => {esc!("M")},   
            Escape::CursorDown(i)                => {escC!(format!("{}B",i))},         
            Escape::CursorRight(i)               => {escC!(format!("{}C",i))},         
            Escape::CursorLeft(i)                => {escC!(format!("{}D",i))},         
            Escape::CursorToNextLineStart(i)     => {escC!(format!("{}E",i))},         
            Escape::CursorToPastLineStart(i)     => {escC!(format!("{}F",i))},         
            Escape::ClearInDisplay               => {escC!("J")},               
            Escape::ClearDisplayUntilScreenEnd   => {escC!("0J")},   
            Escape::ClearDisplayUntilScreenStart => {escC!("1J")}, 
            Escape::ClearAll                     => {escC!("2J")},                     
            Escape::EraseSavedLine               => {escC!("3J")},               
            Escape::EraseInLine                  => {escC!("K")},                  
            Escape::EraseFromCursorToEnd         => {escC!("0K")},         
            Escape::EraseFromCursorToStart       => {escC!("1K")},       
            Escape::EraseLine                    => {escC!("2K")},                    
            Escape::CursorToCol(i)               => {escC!(format!("{}G", i))},   
            Escape::SetBold                      => {escC!("1m")},         
            Escape::SetDim                       => {escC!("2m")},          
            Escape::SetItalic                    => {escC!("3m")},       
            Escape::SetUnderline                 => {escC!("4m")},    
            Escape::SetBlinking                  => {escC!("5m")},     
            Escape::SetInverse                   => {escC!("7m")},      
            Escape::SetHidden                    => {escC!("8m")},       
            Escape::SetStrikethrough             => {escC!("9m")},
            Escape::SetForgroundBlack            => {escC!("30m")},     
            Escape::SetBackgroundBlack           => {escC!("40m")},    
            Escape::SetForgroundRed              => {escC!("31m")},       
            Escape::SetBackgroundRed             => {escC!("41m")},      
            Escape::SetForgroundGreen            => {escC!("32m")},     
            Escape::SetBackgroundGreen           => {escC!("42m")},    
            Escape::SetForgroundYellow           => {escC!("33m")},    
            Escape::SetBackgroundYellow          => {escC!("43m")},   
            Escape::SetForgroundBlue             => {escC!("34m")},      
            Escape::SetBackgroundBlue            => {escC!("44m")},     
            Escape::SetForgroundMagenta          => {escC!("35m")},   
            Escape::SetBackgroundMagenta         => {escC!("45m")},  
            Escape::SetForgroundCyan             => {escC!("36m")},      
            Escape::SetBackgroundCyan            => {escC!("46m")},     
            Escape::SetForgroundWhite            => {escC!("37m")},     
            Escape::SetBackgroundWhite           => {escC!("47m")},    
            Escape::SetForgroundDefault          => {escC!("39m")},   
            Escape::SetBackgroundDefault         => {escC!("49m")},  
            Escape::SetForgroundBriBlack         => {escC!("90m")},  
            Escape::SetBackgroundBriBlack        => {escC!("100m")},  
            Escape::SetForgroundBriRed           => {escC!("91m")},     
            Escape::SetBackgroundBriRed          => {escC!("101m")},    
            Escape::SetForgroundBriGreen         => {escC!("92m")},   
            Escape::SetBackgroundBriGreen        => {escC!("102m")},  
            Escape::SetForgroundBriYellow        => {escC!("93m")},  
            Escape::SetBackgroundBriYellow       => {escC!("103m")}, 
            Escape::SetForgroundBriBlue          => {escC!("94m")},    
            Escape::SetBackgroundBriBlue         => {escC!("104m")},   
            Escape::SetForgroundBriMagenta       => {escC!("95m")}, 
            Escape::SetBackgroundBriMagenta      => {escC!("105m")},
            Escape::SetForgroundBriCyan          => {escC!("96m")},    
            Escape::SetBackgroundBriCyan         => {escC!("106m")},   
            Escape::SetForgroundBriWhite         => {escC!("97m")},   
            Escape::SetBackgroundBriWhite        => {escC!("107m")},  
            Escape::SetForgroundCustomColor(i)   => {escC!(format!("38;5;{}m",i))}, 
            Escape::SetBackgroundCustomColor(i)  => {escC!(format!("48;5;{}m",i))},
            Escape::ResetBold                    => {escC!("22m")},         
            Escape::ResetDim                     => {escC!("22m")},          
            Escape::ResetItalic                  => {escC!("23m")},       
            Escape::ResetUnderline               => {escC!("24m")},    
            Escape::ResetBlinking                => {escC!("25m")},     
            Escape::ResetInverse                 => {escC!("27m")},      
            Escape::ResetHidden                  => {escC!("28m")},       
            Escape::ResetStrikethrough           => {escC!("29m")},
            Escape::SaveCursorPos                => {esc!("7")},
            Escape::RestoreCursorPos             => {esc!("8")},
            Escape::RequestCursorPos             => {escC!("6n")},
            Escape::Set40_25MonoScreen           => {escC!("=0h")},        
            Escape::Set40_25ColorScreen          => {escC!("=1h")},       
            Escape::Set80_25MonoScreen           => {escC!("=2h")},        
            Escape::Set80_25ColorScreen          => {escC!("=3h")},       
            Escape::Set320_2004ColorScreen       => {escC!("=4h")},    
            Escape::Set320_200MonoScreen         => {escC!("=5h")},      
            Escape::Set640_200MonoScreen         => {escC!("=6h")},      
            Escape::EnableLineWrap               => {escC!("=7h")},            
            Escape::Set320_200ColorScreen        => {escC!("=13h")},     
            Escape::Set640_200ColorScreen        => {escC!("=14h")},     
            Escape::Set640_350MonoScreen         => {escC!("=15h")},      
            Escape::Set640_350ColorScreen        => {escC!("=16h")},     
            Escape::Set640_480MonoScreen         => {escC!("=17h")},      
            Escape::Set640_480ColorScreen        => {escC!("=18h")},     
            Escape::Set320_200ColorScreen256     => {escC!("=19h")},  
            Escape::ResetScreenSet(i)            => {escC!(format!("={}l",i))},        
            Escape::SetCursorVisible             => {escC!("?25h")},          
            Escape::SetCursorInvisible           => {escC!("?25l")},        
            Escape::RestoreScreen                => {escC!("?47h")},             
            Escape::SaveScreen                   => {escC!("?47l")},                
            Escape::EnterAltScreen               => {escC!("?1049h")},           
            Escape::ExitAltScreen                => {escC!("?1049l")},          
        };

        strs.into_bytes()
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
