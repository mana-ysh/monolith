use monolith::lang::prelude::*;
use monolith::dataset::{conll, StdLoader};
pub use monolith::dataset::Load;
use monolith::preprocessing::Preprocess;
use monolith::preprocessing::Vocab;

/// (word_ids, char_ids, tag_ids) of a sentence.
pub type Sample = (Vec<u32>, Vec<Vec<u32>>, Vec<u32>);

pub struct Preprocessor {
    word_v: Vocab,
    char_v: Vocab,
    pos_v: Vocab,
}

impl Preprocessor {
    pub fn new(word_v: Vocab) -> Self {
        Preprocessor {
            word_v: word_v,
            char_v: Vocab::new(),
            pos_v: Vocab::with_default_token("NN".to_string()),
        }
    }
}

impl<T: Phrasal> Preprocess<T> for Preprocessor {
    type Output = Sample;

    fn fit_each(&mut self, x: &T) -> Option<Self::Output> {
        let mut word_ids = vec![];
        let mut char_ids = vec![];
        let mut pos_ids = vec![];
        x.iter().for_each(|token| {
            let form = token.form();
            word_ids.push(self.word_v.add(form.to_lowercase()));
            char_ids.push(
                token
                    .form()
                    .chars()
                    .map(|c| self.char_v.add(c.to_string()))
                    .collect(),
            );
            pos_ids.push(self.pos_v.add(token.postag().unwrap().to_string()));
        });
        Some((word_ids, char_ids, pos_ids))
    }

    fn transform_each(&self, x: T) -> Self::Output {
        let mut word_ids = vec![];
        let mut char_ids = vec![];
        let mut pos_ids = vec![];
        x.iter().for_each(|token| {
            let form = token.form();
            word_ids.push(self.word_v.get(&form.to_lowercase()));
            char_ids.push(
                token
                    .form()
                    .chars()
                    .map(|c| self.char_v.get(&c.to_string()))
                    .collect(),
            );
            pos_ids.push(self.pos_v.get(token.postag().unwrap()));
        });
        (word_ids, char_ids, pos_ids)
    }
}

pub type Loader<'a, P> = StdLoader<Sentence<conll::Token<'a>>, P>;
