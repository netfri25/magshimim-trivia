# TODO
 - [ ] comment obscure parts of the code
 - [x] change the statistics table in the database so that `statistics.id == user.id`
 - [x] fix the scoring system
 - [x] limit the time that the user can answer a question (server side)
 - [x] show a timer of how much time is left to answer the question
    when time is up, go to the next question
 - [x] add [correctly answered, questions left] while playing the game
 - [x] fix score being NaN when the user doesn't answer any question
 - [x] remove `is_*` from the Request/Result and use the `matches!` macro instead
 - [ ] refactor some of the trivia/db/sqlite.rs module:
    1. [ ] function that abstracts the query for a specific statistic of some user
    2. [ ] proper usage of the .iterate() and .prepare() methods of the connection

### Bonuses
 - [ ] password and email regex checking
    * [ ] additional user info:
        - phone (prefix, number)
        - address (city, street, apartment)
        - birth date [feature: date_picker](https://github.com/iced-rs/iced_aw/tree/main/examples/date_picker/src/main.rs)
 - [ ] add question page

### my own additions
 - [ ] add more room info in the Join Room page
 - [ ] show the room settings in the Room page
    * [ ] (optional) allow editing of the current room settings
 - [ ] implement an easy way to go back a page / go back to the main menu
 - [ ] cards + models for errors [features: card, model](https://github.com/iced-rs/iced_aw/tree/main/examples/model/src/main.rs)
