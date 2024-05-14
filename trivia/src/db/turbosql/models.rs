use turbosql::Turbosql;

#[derive(Debug, Turbosql, Default)]
pub struct User {
    pub rowid: Option<i64>,
    pub username: String,
    pub password: String,
    pub email: String,
    pub phone: String,
    pub city: String,
    pub street: String,
    pub apartment: u32,
    pub birth_date: String, // using NaiveDate.format(DATE_FORMAT)
}

#[derive(Turbosql, Default)]
pub struct Question {
    pub rowid: Option<i64>,
    pub content: String,
}

#[derive(Turbosql, Default)]
pub struct Answer {
    pub rowid: Option<i64>,
    pub content: String,
    pub correct: bool,
    pub question_id: i64,
}

#[derive(Turbosql, Default)]
pub struct Statistics {
    pub rowid: Option<i64>,
    pub correct_answers: i64,
    pub total_answers: i64,
    pub average_answer_time: f64,
    pub total_games: i64,
    pub overall_score: f64,
}

#[derive(Default)]
pub struct HighscoreResult {
    pub username: String,
    pub overall_score: f64,
}
