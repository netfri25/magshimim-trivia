use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Question {
    #[serde(rename = "type")]
    kind: QuestionKind,

    difficulty: Difficulty,
    category: String,
    question: String,
    correct_answer: String,
    incorrect_answers: Vec<String>
}

impl Question {
    pub fn kind(&self) -> QuestionKind {
        self.kind
    }

    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn question(&self) -> &str {
        &self.question
    }

    pub fn possible_answers(&self) -> PossibleAnswers {
        PossibleAnswers {
            correct_answer: &self.correct_answer,
            incorrect_answers: &self.incorrect_answers
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum QuestionKind {
    Boolean,
    Multiple,
}

#[derive(Debug, Clone, Copy)]
pub struct PossibleAnswers<'a> {
    pub correct_answer: &'a str,
    pub incorrect_answers: &'a [String],
}
