use crate::page::Page;



pub enum Action {
    GoTo(Box<dyn Page>),
    MakeRequest(trivia::messages::Request),
    Nothing,
}
