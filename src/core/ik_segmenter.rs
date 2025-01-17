use crate::core::char_util::{regularize_str, utf8_len, CharType};
use crate::core::cjk_segmenter::CJKSegmenter;
use crate::core::cn_quantifier_segmenter::CnQuantifierSegmenter;
use crate::core::ik_arbitrator::IKArbitrator;
use crate::core::letter_segmentor::LetterSegmenter;
use crate::core::lexeme::{Lexeme, LexemeType};
use crate::core::lexeme_path::LexemePath;
use crate::core::ordered_linked_list::OrderedLinkedList;
use crate::core::segmentor::Segmenter;
use crate::dict::dictionary::GLOBAL_DICT;
use std::collections::{HashMap, LinkedList};

#[derive(Debug, Clone)]
pub enum TokenMode {
    INDEX,
    SEARCH,
}
// ik main class
pub struct IKSegmenter {
    arbitrator: IKArbitrator,
}

unsafe impl Send for IKSegmenter {}
unsafe impl Sync for IKSegmenter {}

impl IKSegmenter {
    pub fn new() -> Self {
        let ik = IKSegmenter {
            arbitrator: IKArbitrator::default(),
        };
        ik
    }

    fn new_segmenters() -> Vec<Box<dyn Segmenter>> {
        vec![
            Box::new(LetterSegmenter::default()),
            Box::new(CnQuantifierSegmenter::default()),
            Box::new(CJKSegmenter::default()),
        ]
    }

    pub fn tokenize(&/*mut*/ self, text: &str, mode: TokenMode) -> Vec<Lexeme> {
        let regular_str = regularize_str(text);
        let input = regular_str.as_str();
        let mut origin_lexemes = OrderedLinkedList::<Lexeme>::new();
        let mut segmenters = IKSegmenter::new_segmenters();
        for (cursor, curr_char) in input.chars().enumerate() {
            let curr_char_type = CharType::from(curr_char);
            for segmenter in segmenters.iter_mut() {
                segmenter.analyze(input, cursor, &curr_char_type, &mut origin_lexemes);
            }
        }

        let mut path_map = self.arbitrator.process(&origin_lexemes, &mode);
        let mut results = self.output_to_result(&mut path_map, input);
        let mut final_results = Vec::with_capacity(results.len());
        // remove stop word
        let mut result = results.pop_front();
        while let Some(ref mut result_value) = result {
            match &mode {
                TokenMode::SEARCH => {
                    self.compound(&mut results, result_value);
                }
                _ => {}
            }

            let lock_guard = {cfg_if::cfg_if! {
                if #[cfg(feature="use-parking-lot")] {Some(GLOBAL_DICT.read())}
                else /*if #[cfg(feature="use-std-sync")]*/ {GLOBAL_DICT.read().map_or(None,|x|Some(x))}
            }};
            if lock_guard.is_none() || !lock_guard.is_some_and(|x|x.is_stop_word(
                    input,
                    result_value.begin_pos(),
                    result_value.len()))
            {
                result_value.parse_lexeme_text(input);
                final_results.push(result_value.clone())
            }
            result = results.pop_front();
        }
        final_results
    }

    fn output_to_result(
        &self,
        path_map: &mut HashMap<usize, LexemePath>,
        input: &str,
    ) -> LinkedList<Lexeme> {
        let mut results = LinkedList::new();
        let mut index = 0usize;
        let char_count = utf8_len(input);
        while index < char_count {
            let curr_char = input.chars().nth(index).unwrap();
            let cur_char_type = CharType::from(curr_char);
            match cur_char_type {
                CharType::USELESS => {
                    index += 1;
                    continue;
                }
                _ => {}
            }
            let path = path_map.get_mut(&index);
            if let Some(p) = path {
                let mut cur_lexeme = p.poll_first();
                while let Some(ref lexeme) = cur_lexeme {
                    results.push_back(lexeme.clone());
                    index = lexeme.end_pos();
                    cur_lexeme = p.poll_first();
                    if let Some(ref lexeme) = cur_lexeme {
                        while index < lexeme.begin_pos() {
                            let curr_char = input.chars().nth(index).unwrap();
                            let cur_char_type = CharType::from(curr_char);
                            self.add_single_lexeme(&mut results, &cur_char_type, index);
                            index += 1;
                        }
                    }
                }
            } else {
                self.add_single_lexeme(&mut results, &cur_char_type, index);
                index += 1;
            }
        }
        results
    }

    fn add_single_lexeme(
        &self,
        results: &mut LinkedList<Lexeme>,
        cur_char_type: &CharType,
        index: usize,
    ) {
        let mut lexeme_type = None;
        match cur_char_type {
            CharType::CHINESE => {
                lexeme_type = Some(LexemeType::CNCHAR);
            }
            CharType::OtherCjk => {
                lexeme_type = Some(LexemeType::OtherCJK);
            }
            _ => {}
        }
        lexeme_type.map(|l_type| {
            let single_char_lexeme = Lexeme::new(index..index + 1, l_type);
            results.push_back(single_char_lexeme);
        });
    }

    fn compound(&self, results: &mut LinkedList<Lexeme>, result: &mut Lexeme) {
        if !results.is_empty() {
            match result.lexeme_type() {
                LexemeType::ARABIC => {
                    let mut append_ok = false;
                    let next_lexeme = results.front();
                    next_lexeme.map(|next| match next.lexeme_type() {
                        LexemeType::CNUM => {
                            append_ok = result.append(next, LexemeType::CNUM);
                        }
                        LexemeType::COUNT => {
                            append_ok = result.append(next, LexemeType::CQUAN);
                        }
                        _ => {}
                    });
                    if append_ok {
                        results.pop_front();
                    }
                }
                _ => {}
            }

            match result.lexeme_type() {
                LexemeType::CNUM if !results.is_empty() => {
                    let mut append_ok = false;
                    let next_lexeme = results.front();
                    next_lexeme.map(|next| match next.lexeme_type() {
                        LexemeType::COUNT => {
                            append_ok = result.append(next, LexemeType::CQUAN);
                        }
                        _ => {}
                    });
                    if append_ok {
                        results.pop_front();
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use log;
    use std::thread;

    #[test]
    fn test_index_segment() {
        let ik = IKSegmenter::new();
        let texts = _get_input_texts();
        for text in texts.iter() {
            let tokens = ik.tokenize(text, TokenMode::INDEX);
            for token in tokens.iter() {
                log::info!("{:?}", token);
            }
            log::info!("{}", "----------------------")
        }
    }

    #[test]
    fn test_search_segment() {
        let ik = IKSegmenter::new();
        let texts = _get_input_texts();
        for text in texts {
            let tokens = ik.tokenize(text, TokenMode::SEARCH);
            for token in tokens.iter() {
                log::info!("{:?}", token);
            }
            log::info!("{}", "----------------------")
        }
    }

    fn _get_input_texts() -> Vec<&'static str> {
        let texts = vec![
            "张三说的确实在理",
            "中华人民共和国",
            "zhiyi.shen@gmail.com",
            "我感觉很happy,并且不悲伤!",
            "结婚的和尚未结婚的",
            "中国有960万平方公里的国土",
            "我的年纪是十八",
        ];
        texts
    }

    #[test]
    fn test_thread_safe() {
        let ik = IKSegmenter::new();
        let t = thread::spawn(move || {
            println!("{:?}", ik.tokenize("明天星期几?", TokenMode::INDEX));
        });
        t.join().unwrap();
    }
}
