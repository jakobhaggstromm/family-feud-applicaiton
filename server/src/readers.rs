use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::models::{Answer, Question, QuestionInput};

pub fn read_questions_from_file(path: &str) -> Result<Vec<Question>, Box<dyn std::error::Error>> {
    let file = File::open(Path::new(path))?;
    let reader = BufReader::new(file);
    let inputs: Vec<QuestionInput> = serde_json::from_reader(reader)?;

    let questions = inputs
        .into_iter()
        .map(|qi| Question {
            text: qi.text,
            answers: qi
                .answers
                .into_iter()
                .map(|ai| Answer {
                    text: ai.text,
                    points: ai.points,
                    revealed: false,
                })
                .collect(),
        })
        .collect();

    Ok(questions)
}