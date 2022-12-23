use crate::core::char_util::utf8_len;
use crate::core::char_util::CharType;
use crate::core::lexeme::{Lexeme, LexemeType};
use crate::core::ordered_linked_list::OrderedLinkedList;
use crate::core::segmentor::Segmenter;
use crate::dict::dictionary::GLOBAL_DICT;

const SEGMENTER_NAME: &str = "CJK_SEGMENTER";

#[derive(Debug)]
pub struct CJKSegmenter {}

impl Segmenter for CJKSegmenter {
    fn analyze(
        &mut self,
        input: &str,
        cursor: usize,
        curr_char_type: CharType,
        origin_lexemes: &mut OrderedLinkedList<Lexeme>,
    ) {
        let char_count = utf8_len(input);
        // for (cursor, curr_char) in input.chars().enumerate() {
        // let curr_char_type = char_type_of(curr_char);
        if CharType::USELESS != curr_char_type {
            let hit_options = GLOBAL_DICT.lock().unwrap().match_in_main_dict_with_offset(
                input,
                cursor,
                char_count - cursor,
            );
            for hit in hit_options.iter() {
                if hit.is_match() {
                    let new_lexeme = Lexeme::new(0, hit.begin, hit.length(), LexemeType::CNWORD);
                    origin_lexemes.insert(new_lexeme);
                }
            }
        }
    }

    fn name(&self) -> &str {
        return SEGMENTER_NAME;
    }
}

impl CJKSegmenter {
    pub fn new() -> Self {
        CJKSegmenter {}
    }
}
