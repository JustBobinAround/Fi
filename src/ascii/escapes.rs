pub trait ParsableSequence<T> {
    fn parse_sequence<I>(chars: &mut std::iter::Peekable<I>) -> Vec<T> where I: Iterator<Item = char>;
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
            if c as char == '\x1b' {
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
    EnableAltBuffer,           //?1049h
    DisableAltBuffer,          //?1049l
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
}

enum SpecialLongCase {
    NoSpecial,
    ScreenMode,
    PrivateMode
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
                                "1049" => {escapes.push(Escape::DisableAltBuffer)},
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
                                "1049" => {escapes.push(Escape::EnableAltBuffer)},
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
