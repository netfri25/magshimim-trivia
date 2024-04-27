use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Question {
    #[serde(rename = "type")]
    kind: QuestionKind,

    difficulty: Difficulty,
    category: String,
    question: String,
    correct_answer: String,
    incorrect_answers: Vec<String>,
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

    pub fn correct_answer(&self) -> &str {
        &self.correct_answer
    }

    pub fn incorrect_answers(&self) -> &[String] {
        &self.incorrect_answers
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

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestionData {
    pub question: String,
    pub answers: Vec<String>,
    pub correct_answer_index: usize,
}

impl QuestionData {
    pub fn new(question: String, answers: Vec<String>, correct_answer_index: usize) -> Self {
        Self {
            question,
            answers,
            correct_answer_index,
        }
    }

    pub(crate) fn correct_answer(&self) -> &str {
        &self.answers[self.correct_answer_index]
    }
}
