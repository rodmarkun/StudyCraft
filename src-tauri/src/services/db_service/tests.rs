use super::connection::DbService;
use super::queries;
use super::review_materials::ReviewMaterialOperations;
use crate::commands::structure::requests;
use crate::errors::{AppError, AppResult};
use crate::materials::review_material::{ReviewMaterial, ReviewMaterialType};
use crate::materials::test::{NewTestQuestion, TestAnswer, TestQuestion, TestQuestionWithAnswers};
use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use uuid::Uuid;

#[derive(Debug)]
struct TestQuestionRow {
    id: i64,
    test_id: String,
    question: String,
    position: i32,
    created_at: String,
    updated_at: String,
}

impl TestQuestionRow {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(TestQuestionRow {
            id: row.get("id")?,
            test_id: row.get("test_id")?,
            question: row.get("question")?,
            position: row.get("position")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }

    fn into_test_question(self) -> AppResult<TestQuestion> {
        let created_at =
            chrono::NaiveDateTime::parse_from_str(&self.created_at, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| AppError::DbError(format!("Failed to parse created_at: {}", e)))?
                .and_utc();

        let updated_at =
            chrono::NaiveDateTime::parse_from_str(&self.updated_at, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| AppError::DbError(format!("Failed to parse updated_at: {}", e)))?
                .and_utc();

        Ok(TestQuestion {
            id: Some(self.id),
            test_id: self.test_id,
            question: self.question,
            position: self.position,
            created_at,
            updated_at,
        })
    }
}

#[derive(Debug)]
struct TestAnswerRow {
    id: i64,
    question_id: i64,
    answer_text: String,
    is_correct: bool,
    position: i32,
    created_at: String,
    updated_at: String,
}

impl TestAnswerRow {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(TestAnswerRow {
            id: row.get("id")?,
            question_id: row.get("question_id")?,
            answer_text: row.get("answer_text")?,
            is_correct: row.get("is_correct")?,
            position: row.get("position")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }

    fn into_test_answer(self) -> AppResult<TestAnswer> {
        let created_at =
            chrono::NaiveDateTime::parse_from_str(&self.created_at, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| AppError::DbError(format!("Failed to parse created_at: {}", e)))?
                .and_utc();

        let updated_at =
            chrono::NaiveDateTime::parse_from_str(&self.updated_at, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| AppError::DbError(format!("Failed to parse updated_at: {}", e)))?
                .and_utc();

        Ok(TestAnswer {
            id: Some(self.id),
            question_id: self.question_id,
            answer_text: self.answer_text,
            is_correct: self.is_correct,
            position: self.position,
            created_at,
            updated_at,
        })
    }
}

pub trait TestOperations {
    fn create_test(
        &self,
        test_id_str: &str,
        name: &str,
        tags: &[String],
        questions: &[NewTestQuestion],
    ) -> AppResult<ReviewMaterial>;
    fn get_tests(&self) -> AppResult<Vec<ReviewMaterial>>;
    fn get_test(&self, id: &str) -> AppResult<Option<ReviewMaterial>>;
    fn get_test_questions(&self, test_id: &str) -> AppResult<Vec<TestQuestionWithAnswers>>;
    fn get_content_from_random_test(&self, nitems: u32) -> AppResult<Vec<TestQuestionWithAnswers>>;
    fn update_test(&self, request: requests::TestUpdateRequest) -> AppResult<()>;
    fn delete_test(&self, id: &str) -> AppResult<()>;
    fn update_test_name(&self, id: &str, name: &str) -> AppResult<()>;
}

impl TestOperations for DbService {
    fn create_test(
        &self,
        test_id_str: &str,
        name: &str,
        tags: &[String],
        questions: &[NewTestQuestion],
    ) -> AppResult<ReviewMaterial> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn
            .transaction()
            .map_err(|e| AppError::DbError(format!("Failed to start transaction: {}", e)))?;

        let now = Utc::now();

        let review_material = ReviewMaterial {
            id: String::from(test_id_str),
            display_name: name.to_string(),
            rm_type: ReviewMaterialType::Test,
            last_review: None,
            created_at: now,
            updated_at: now,
        };

        self.add_review_material_tx(&tx, &review_material)?;

        for tag in tags {
            if tag.trim().is_empty() {
                continue;
            }

            tx.execute(queries::INSERT_TAG, params![tag.trim()])
                .map_err(|e| AppError::DbError(format!("Failed to insert tag: {}", e)))?;

            let tag_id: i64 = tx
                .query_row(queries::GET_TAG_ID, params![tag.trim()], |row| row.get(0))
                .map_err(|e| AppError::DbError(format!("Failed to get tag ID: {}", e)))?;

            tx.execute(queries::INSERT_MATERIAL_TAG, params![test_id_str, tag_id])
                .map_err(|e| AppError::DbError(format!("Failed to link test to tag: {}", e)))?;
        }

        for (i, question) in questions.iter().enumerate() {
            if question.question.trim().is_empty() {
                continue;
            }

            tx.execute(
                queries::INSERT_TEST_QUESTION,
                params![
                    test_id_str,
                    question.question.trim(),
                    question.position.max(i as i32)
                ],
            )
            .map_err(|e| AppError::DbError(format!("Failed to insert test question: {}", e)))?;

            // Get the ID of the inserted question
            let question_id = tx.last_insert_rowid();

            for (j, answer) in question.answers.iter().enumerate() {
                if answer.answer_text.trim().is_empty() {
                    continue;
                }

                tx.execute(
                    queries::INSERT_TEST_ANSWER,
                    params![
                        question_id,
                        answer.answer_text.trim(),
                        answer.is_correct,
                        answer.position.max(j as i32)
                    ],
                )
                .map_err(|e| AppError::DbError(format!("Failed to insert test answer: {}", e)))?;
            }
        }

        tx.commit()
            .map_err(|e| AppError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(review_material)
    }

    fn get_tests(&self) -> AppResult<Vec<ReviewMaterial>> {
        Ok(self
            .get_review_materials()?
            .into_iter()
            .filter(|rm| matches!(rm.rm_type, ReviewMaterialType::Test))
            .collect())
    }

    fn get_test(&self, id: &str) -> AppResult<Option<ReviewMaterial>> {
        self.get_review_material(id)
    }

    fn get_test_questions(&self, test_id: &str) -> AppResult<Vec<TestQuestionWithAnswers>> {
        let conn = self.conn.borrow();
        let mut stmt = conn
            .prepare(queries::GET_TEST_QUESTIONS)
            .map_err(|e| AppError::DbError(format!("Failed to prepare statement: {}", e)))?;

        let question_iter = stmt
            .query_map(params![test_id], |row| TestQuestionRow::from_row(row))
            .map_err(|e| AppError::DbError(format!("Failed to query test questions: {}", e)))?;

        let mut questions_with_answers = Vec::new();

        for question_result in question_iter {
            let row = question_result.map_err(|e| {
                AppError::DbError(format!("Failed to process test question: {}", e))
            })?;
            let question = row.into_test_question()?;

            let mut answer_stmt = conn
                .prepare(queries::GET_TEST_ANSWERS_BY_QUESTION)
                .map_err(|e| {
                    AppError::DbError(format!("Failed to prepare answer statement: {}", e))
                })?;

            let answer_iter = answer_stmt
                .query_map(params![question.id.unwrap()], |row| {
                    TestAnswerRow::from_row(row)
                })
                .map_err(|e| AppError::DbError(format!("Failed to query test answers: {}", e)))?;

            let mut answers = Vec::new();
            for answer_result in answer_iter {
                let row = answer_result.map_err(|e| {
                    AppError::DbError(format!("Failed to process test answer: {}", e))
                })?;
                answers.push(row.into_test_answer()?);
            }

            questions_with_answers.push(TestQuestionWithAnswers { question, answers });
        }

        Ok(questions_with_answers)
    }

    fn get_content_from_random_test(&self, nitems: u32) -> AppResult<Vec<TestQuestionWithAnswers>> {
        let random_test_id: String =
            match self
                .conn
                .borrow()
                .query_row(queries::GET_RANDOM_TEST_ID, [], |row| row.get(0))
            {
                Ok(id) => id,
                Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(Vec::new()),
                Err(e) => {
                    return Err(AppError::DbError(format!(
                        "Failed to get random test: {}",
                        e
                    )))
                }
            };

        let conn = self.conn.borrow();
        // Get random questions from that test using SQL randomization
        let mut stmt = conn
            .prepare(queries::GET_RANDOM_TEST_QUESTIONS)
            .map_err(|e| AppError::DbError(format!("Failed to prepare statement: {}", e)))?;

        let question_iter = stmt
            .query_map(params![random_test_id, nitems], |row| {
                TestQuestionRow::from_row(row)
            })
            .map_err(|e| {
                AppError::DbError(format!("Failed to query random test questions: {}", e))
            })?;

        let mut questions_with_answers = Vec::new();

        for question_result in question_iter {
            let row = question_result.map_err(|e| {
                AppError::DbError(format!("Failed to process test question: {}", e))
            })?;
            let question = row.into_test_question()?;

            let mut answer_stmt = conn
                .prepare(queries::GET_TEST_ANSWERS_BY_QUESTION)
                .map_err(|e| {
                    AppError::DbError(format!("Failed to prepare answer statement: {}", e))
                })?;

            let answer_iter = answer_stmt
                .query_map(params![question.id.unwrap()], |row| {
                    TestAnswerRow::from_row(row)
                })
                .map_err(|e| AppError::DbError(format!("Failed to query test answers: {}", e)))?;

            let mut answers = Vec::new();
            for answer_result in answer_iter {
                let row = answer_result.map_err(|e| {
                    AppError::DbError(format!("Failed to process test answer: {}", e))
                })?;
                answers.push(row.into_test_answer()?);
            }

            questions_with_answers.push(TestQuestionWithAnswers { question, answers });
        }

        Ok(questions_with_answers)
    }

    fn update_test(&self, request: requests::TestUpdateRequest) -> AppResult<()> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn
            .transaction()
            .map_err(|e| AppError::DbError(format!("Failed to start transaction: {}", e)))?;

        // Update test name if provided
        if let Some(new_name) = &request.name {
            self.update_review_material_name_tx(&tx, &request.test_id, new_name)?;
        }

        // Delete questions if specified
        if let Some(questions_to_delete) = &request.questions_to_delete {
            for question_id in questions_to_delete {
                tx.execute(
                    queries::DELETE_TEST_QUESTION,
                    params![question_id, request.test_id],
                )
                .map_err(|e| {
                    AppError::DbError(format!("Failed to delete question {}: {}", question_id, e))
                })?;
            }
        }

        // Update existing questions if specified
        if let Some(questions_to_update) = &request.questions_to_update {
            for (q_index, question) in questions_to_update.iter().enumerate() {
                tx.execute(
                    queries::UPDATE_TEST_QUESTION,
                    params![
                        question.question,
                        question.position,
                        question.id,
                        request.test_id
                    ],
                )
                .map_err(|e| {
                    AppError::DbError(format!("Failed to update question {}: {}", question.id, e))
                })?;

                for (a_index, answer) in question.answers.iter().enumerate() {
                    if let Some(answer_id) = answer.id {
                        let rows_affected = tx
                            .execute(
                                queries::UPDATE_TEST_ANSWER,
                                params![
                                    answer.answer_text,
                                    answer.is_correct,
                                    answer.position,
                                    answer_id
                                ],
                            )
                            .map_err(|e| {
                                AppError::DbError(format!(
                                    "Failed to update answer {}: {}",
                                    answer_id, e
                                ))
                            })?;
                    } else {
                        tx.execute(
                            queries::INSERT_TEST_ANSWER,
                            params![
                                question.id,
                                answer.answer_text,
                                answer.is_correct,
                                answer.position
                            ],
                        )
                        .map_err(|e| {
                            AppError::DbError(format!("Failed to add new answer: {}", e))
                        })?;
                    }
                }
            }
        }

        if let Some(questions_to_add) = &request.questions_to_add {
            for question in questions_to_add {
                tx.execute(
                    queries::INSERT_TEST_QUESTION,
                    params![request.test_id, question.question, question.position],
                )
                .map_err(|e| AppError::DbError(format!("Failed to add new question: {}", e)))?;

                // Get the ID of the inserted question
                let question_id = tx.last_insert_rowid();
                for answer in &question.answers {
                    tx.execute(
                        queries::INSERT_TEST_ANSWER,
                        params![
                            question_id,
                            answer.answer_text,
                            answer.is_correct,
                            answer.position
                        ],
                    )
                    .map_err(|e| {
                        AppError::DbError(format!("Failed to add answer for new question: {}", e))
                    })?;
                }
            }
        }

        self.update_review_material_tx(&tx, &request.test_id)?;

        tx.commit()
            .map_err(|e| AppError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    fn delete_test(&self, id: &str) -> AppResult<()> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn
            .transaction()
            .map_err(|e| AppError::DbError(format!("Failed to start transaction: {}", e)))?;

        self.delete_review_material_tx(&tx, id)?;

        tx.commit()
            .map_err(|e| AppError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    fn update_test_name(&self, id: &str, name: &str) -> AppResult<()> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn
            .transaction()
            .map_err(|e| AppError::DbError(format!("Failed to start transaction: {}", e)))?;

        self.update_review_material_name_tx(&tx, id, name)?;

        tx.commit()
            .map_err(|e| AppError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }
}
