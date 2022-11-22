use crate::core::char_util::char_type_of;
use crate::core::char_util::utf8_len;
use crate::core::char_util::CharType;
use crate::core::lexeme::{Lexeme, LexemeType};
use crate::core::segmentor::Segmenter;

const SEGMENTER_NAME: &str = "LETTER_SEGMENTER";

const LETTER_CONNECTOR: [char; 7] = ['#', '&', '+', '-', '.', '@', '_'];

const NUM_CONNECTOR: [char; 2] = [',', '.'];

pub struct LetterSegmenter {
    start: i32,
    end: i32,

    english_start: i32,
    english_end: i32,

    arabic_start: i32,
    arabic_end: i32,
}

impl Segmenter for LetterSegmenter {
    fn analyze(&mut self, input: &str) -> Vec<Lexeme> {
        let mut new_lexemes = Vec::new();
        let mut english_lexemes = self.process_english_letter(input);
        let mut arabic_lexemes = self.process_arabic_letter(input);
        let mut mix_lexemes = self.process_mix_letter(input);
        new_lexemes.append(&mut english_lexemes);
        new_lexemes.append(&mut arabic_lexemes);
        new_lexemes.append(&mut mix_lexemes);
        new_lexemes
    }
    fn name(&self) -> &str {
        return SEGMENTER_NAME;
    }
}

impl LetterSegmenter {
    pub fn new() -> Self {
        LetterSegmenter {
            start: -1,
            end: -1,
            english_start: -1,
            english_end: -1,
            arabic_start: -1,
            arabic_end: -1,
        }
    }

    /// mix letter
    /// windos2000 | zhiyi.shen@gmail.com
    pub fn process_mix_letter(&mut self, input: &str) -> Vec<Lexeme> {
        let mut new_lexemes = Vec::new();
        let char_count = utf8_len(input);
        for (cursor, curr_char) in input.chars().enumerate() {
            let curr_char_type = char_type_of(curr_char);
            if self.start == -1 {
                if CharType::ARABIC == curr_char_type || CharType::ENGLISH == curr_char_type {
                    self.start = cursor as i32;
                    self.end = self.start;
                }
            } else {
                if CharType::ARABIC == curr_char_type || CharType::ENGLISH == curr_char_type {
                    self.end = cursor as i32;
                } else if CharType::USELESS == curr_char_type && self.is_letter_connector(curr_char)
                {
                    self.end = cursor as i32;
                } else {
                    let new_lexeme = Lexeme::new(
                        0,
                        self.start as usize,
                        (self.end - self.start + 1) as usize,
                        LexemeType::LETTER,
                    );
                    new_lexemes.push(new_lexeme);
                    self.start = -1;
                    self.end = -1;
                }
            }
        }

        if self.end == (char_count - 1) as i32 {
            let new_lexeme = Lexeme::new(
                0,
                self.start as usize,
                (self.end - self.start + 1) as usize,
                LexemeType::LETTER,
            );
            new_lexemes.push(new_lexeme);
            self.start = -1;
            self.end = -1;
        }
        new_lexemes
    }

    // english
    fn process_english_letter(&mut self, input: &str) -> Vec<Lexeme> {
        let mut new_lexemes = Vec::new();
        let char_count = utf8_len(input);
        for (cursor, curr_char) in input.chars().enumerate() {
            let curr_char_type = char_type_of(curr_char);
            if self.english_start == -1 {
                if CharType::ENGLISH == curr_char_type {
                    self.english_start = cursor as i32;
                    self.english_end = self.english_start;
                }
            } else {
                if CharType::ENGLISH == curr_char_type {
                    self.english_end = cursor as i32;
                } else {
                    let new_lexeme = Lexeme::new(
                        0,
                        self.english_start as usize,
                        (self.english_end - self.english_start + 1) as usize,
                        LexemeType::ENGLISH,
                    );
                    new_lexemes.push(new_lexeme);
                    self.english_start = -1;
                    self.english_end = -1;
                }
            }
        }
        if self.english_end == (char_count - 1) as i32 {
            let new_lexeme = Lexeme::new(
                0,
                self.english_start as usize,
                (self.english_end - self.english_start + 1) as usize,
                LexemeType::ENGLISH,
            );
            new_lexemes.push(new_lexeme);
            self.english_start = -1;
            self.english_end = -1;
        }
        new_lexemes
    }

    // arabic
    fn process_arabic_letter(&mut self, input: &str) -> Vec<Lexeme> {
        let mut new_lexemes = Vec::new();
        let char_count = utf8_len(input);
        for (cursor, curr_char) in input.chars().enumerate() {
            let curr_char_type = char_type_of(curr_char);
            if self.arabic_start == -1 {
                if CharType::ARABIC == curr_char_type {
                    self.arabic_start = cursor as i32;
                    self.arabic_end = self.arabic_start;
                }
            } else {
                if CharType::ARABIC == curr_char_type {
                    self.arabic_end = cursor as i32;
                } else if CharType::USELESS == curr_char_type && self.is_num_connector(curr_char) {
                    // do nothing
                } else {
                    let new_lexeme = Lexeme::new(
                        0,
                        self.arabic_start as usize,
                        (self.arabic_end - self.arabic_start + 1) as usize,
                        LexemeType::ARABIC,
                    );
                    new_lexemes.push(new_lexeme);
                    self.arabic_start = -1;
                    self.arabic_end = -1;
                }
            }
        }
        if self.arabic_end == (char_count - 1) as i32 {
            let new_lexeme = Lexeme::new(
                0,
                self.arabic_start as usize,
                (self.arabic_end - self.arabic_start + 1) as usize,
                LexemeType::ARABIC,
            );
            new_lexemes.push(new_lexeme);
            self.arabic_start = -1;
            self.arabic_end = -1;
        }
        new_lexemes
    }

    pub fn is_letter_connector(&self, input: char) -> bool {
        LETTER_CONNECTOR.contains(&input)
    }

    pub fn is_num_connector(&self, input: char) -> bool {
        NUM_CONNECTOR.contains(&input)
    }
}
