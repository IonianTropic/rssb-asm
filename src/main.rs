use core::panic;
use std::{env, fs::File, io::{Read, Write}, path::Path, collections::HashMap};


fn main() {
    let config = Config::new();
    write(eval(read(config.input)), config.output);
}


fn read<P: AsRef<Path>>(file: P) -> String {
    let mut f = File::open(file).unwrap();
    let mut src = String::new();
    let _ = f.read_to_string(&mut src).unwrap();
    src
}


fn eval(src: String) -> [u8; 256] {
    let mut program: [u8; 256] = [0; 256];
    let token_stream = scanner(src);
    let (data_stream, text_stream) = partition_stream(token_stream);

    let mut package: HashMap<String, u8> = HashMap::new(); // table containing labels and address

    let data_offset = 16;
    let mut idx = 0;
    let mut data_section: Vec<u8> = Vec::new();
    for token in data_stream {
        let tok_txt = token.text.as_str();
        match token.token_type {
            TokenType::Label => {
                package.insert(token.text, data_offset+idx);
                idx += 1;
            }
            TokenType::Operand => {
                match tok_txt {
                    tok_txt if tok_txt.chars().all(char::is_numeric) => {
                        data_section.push(tok_txt.parse().unwrap());
                    }
                    tok_txt if tok_txt.chars().all(char::is_alphabetic) => {
                        if package.contains_key(&token.text) {
                            data_section.push(package[&token.text]);
                        } else {
                            panic!("Invalid Label!");
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }
    let data_len = data_section.len();

    let max_text_len = 256 - ((data_offset as usize)+data_len);
    let text_offset = data_offset+(data_len as u8);
    idx = 0;
    let mut text_section: Vec<u8> = Vec::new();
    for token in text_stream {
        let tok_txt = token.text.as_str();
        match token.token_type {
            TokenType::Label => {
                package.insert(token.text, text_offset+idx);
                idx += 1;
            }
            TokenType::Operand => {
                match tok_txt {
                    tok_txt if tok_txt.chars().all(char::is_numeric) => {
                        text_section.push(tok_txt.parse().unwrap());
                    }
                    tok_txt if tok_txt.chars().all(char::is_alphabetic) => {
                        if package.contains_key(&token.text) {
                            text_section.push(package[&token.text]);
                        } else {
                            panic!("Invalid Label!");
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }
    let text_len = text_section.len();
    if text_len > max_text_len {
        panic!("Too much assembly!");
    }

    program[0] = text_offset;
    idx = 0;
    for d in data_section {
        program[(data_offset+idx) as usize] = d;
        idx += 1;
    }
    idx = 0;
    for t in text_section {
        program[(text_offset+idx) as usize] = t;
        idx += 1;
    }
    // println!("{:?}", program);
    program
}

fn partition_stream(token_stream: Vec<Token>) -> (Vec<Token>, Vec<Token>) {
    let mut text_stream: Vec<Token> = Vec::new();
    let mut data_stream: Vec<Token> = Vec::new();
    let mut current_section = SectionType::None;
    for token in token_stream {
        match token.token_type {
            TokenType::Section => {
                match token.text.as_str() {
                    "data" => current_section = SectionType::Data,
                    "text" => current_section = SectionType::Text,
                    _ => panic!("Invalid section name"),
                }
            }
            _ => {
                match current_section {
                    SectionType::Text => {
                        text_stream.push(token);
                    }
                    SectionType::Data => {
                        data_stream.push(token);
                    }
                    _ => (),
                }
            }
        }
    }
    (data_stream, text_stream)
}


fn scanner(src: String) -> Vec<Token> {
    let mut scanner_state: ScannerState;
    let mut lhs: usize;
    let mut token_stream: Vec<Token> = Vec::new();
    let mut line_number = 0;
    for line in src.lines() {
        line_number += 1;
        lhs = 0;
        scanner_state = ScannerState::NewLine;
        for (idx, c) in line.char_indices() {
            // println!("{:?}", scanner_state);
            match scanner_state {
                ScannerState::NewLine => match c {
                    c if c.is_whitespace() => {
                        scanner_state = ScannerState::WhiteSpace;
                    }
                    ';' => scanner_state = ScannerState::Comment,
                    c if c.is_alphanumeric() || c == '_' => {
                        lhs = idx;
                        scanner_state = ScannerState::Label;
                    }
                    c if c == '.' => {
                        lhs = idx;
                        scanner_state = ScannerState::Section;
                    }
                    _ => scanner_state = ScannerState::Error,
                }
                ScannerState::Section => match c {
                    c if c.is_whitespace() => {
                        token_stream.push(Token {
                            token_type: TokenType::Section,
                            text: line.get(lhs+1..idx).unwrap().to_string(),
                        });
                        scanner_state = ScannerState::WhiteSpace;
                    }
                    c if c.is_alphanumeric() || c == '_' || c == '-' => (),
                    _ => scanner_state = ScannerState::Error,
                }
                ScannerState::Label => match c {
                    ':' => {
                        token_stream.push(Token {
                            token_type: TokenType::Label,
                            text: line.get(lhs..idx).unwrap().to_string(),
                        });
                        scanner_state = ScannerState::WhiteSpace;
                    }
                    c if c.is_alphanumeric() || c == '_' || c == '-' => (),
                    _ => scanner_state = ScannerState::Error,
                }
                ScannerState::Operand => match c {
                    c if c.is_whitespace() => {
                        scanner_state = ScannerState::WhiteSpace;
                        token_stream.push(Token {
                            token_type: TokenType::Operand,
                            text: line.get(lhs..idx).unwrap().to_string(),
                        });
                    }
                    ';' => {
                        token_stream.push(Token {
                            token_type: TokenType::Operand,
                            text: line.get(lhs..idx).unwrap().to_string(),
                        });
                        scanner_state = ScannerState::Comment;
                    }
                    c if c.is_alphanumeric() || c == '_' || c == '.' => (),
                    _ => scanner_state = ScannerState::Error,
                }
                ScannerState::WhiteSpace => match c {
                    c if c.is_alphanumeric() || c == '_' || c == '.' => {
                        lhs = idx;
                        scanner_state = ScannerState::Operand;
                    }
                    c if c.is_whitespace() => (),
                    ';' => scanner_state = ScannerState::Comment,
                    _ => scanner_state = ScannerState::Error,
                }
                ScannerState::Comment => {
                    break;
                }
                ScannerState::Error => {
                    panic!("Assembly Evaluation Error at ({}, {}) from {:?}", line_number, idx, c)
                }
            }
        }
        match scanner_state {
            ScannerState::Section => {
                token_stream.push(Token {
                    token_type: TokenType::Section,
                    text: line.split_at(lhs+1).1.to_string(),
                });
            }
            ScannerState::Operand => {
                token_stream.push(Token {
                    token_type: TokenType::Operand,
                    text: line.split_at(lhs).1.to_string(),
                });
            }
            _ => (),
        }
    }
    token_stream
}


#[derive(Debug)]
enum ScannerState {
    NewLine,
    Label,
    Section,
    WhiteSpace,
    Operand,
    Error,
    Comment,
}


#[derive(Debug)]
struct Token {
    token_type: TokenType,
    text: String,
}


#[derive(Debug)]
enum TokenType {
    Section,
    Label,
    Operand,
}

enum SectionType {
    Text,
    Data,
    None,
}


fn write<P: AsRef<Path>>(program: [u8; 256], file: P) {
    let mut f = File::create(file).unwrap();
    let _ = f.write_all(&program);
}


#[derive(Debug)]
struct Config {
    input: String,
    output: String,
}


impl Config {
    fn new() -> Self {
        let mut option_output = false;
        let mut output = String::from("a.s");
        let mut input = String::from("elephantjoyride");
        for arg in env::args() {
            match arg.as_str() {
                "-o" => option_output = true,
                _ => {
                    match option_output {
                        true => {
                            output = arg;
                        }
                        false => {
                            input = arg;
                        }
                    }
                }
            }
        }
        if input == "elephantjoyride" {
            panic!("Invalid file name");
        }
        Self {
            input,
            output,
        }
    }
}
