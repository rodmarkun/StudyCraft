pub const CREATE_STUDY_MATERIALS_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS study_materials (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        original_path TEXT NOT NULL,
        markdown_path TEXT NOT NULL,
        cover_path TEXT,
        is_processing BOOLEAN DEFAULT 0,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        extra_instructions TEXT DEFAULT '',
        is_indexed BOOLEAN DEFAULT 0,
        chunk_count INTEGER,
        last_indexed TIMESTAMP,
        url TEXT DEFAULT ''
    )";

pub const CREATE_REVIEW_MATERIALS_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS review_materials (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        type TEXT NOT NULL,
        last_review TIMESTAMP,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        deleted BOOLEAN DEFAULT 0
    )";

pub const CREATE_FLASHCARDS_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS flashcards (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        deck_id TEXT NOT NULL,
        front TEXT NOT NULL,
        back TEXT NOT NULL,
        position INTEGER NOT NULL,
        
        -- Spaced repetition fields
        due_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        interval_days INTEGER DEFAULT 0,
        ease_factor REAL DEFAULT 2.5,
        repetitions INTEGER DEFAULT 0,
        status TEXT DEFAULT 'new',  -- 'new', 'learning', 'review', 'suspended'
        last_reviewed TIMESTAMP,
        learning_step INTEGER DEFAULT 0,  -- Current step in learning phase
        
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (deck_id) REFERENCES review_materials (id) ON DELETE CASCADE
    )";

pub const CREATE_TAGS_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS tags (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        is_available BOOLEAN DEFAULT 1
    )";

pub const CREATE_MATERIAL_TAGS_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS material_tags (
        material_id TEXT NOT NULL,
        tag_id INTEGER NOT NULL,
        PRIMARY KEY (material_id, tag_id),
        FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE
    )";

pub const CREATE_REVIEW_SESSIONS_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS review_sessions (
        id TEXT PRIMARY KEY,
        material_id TEXT NOT NULL,
        material_type TEXT NOT NULL,
        session_start TIMESTAMP NOT NULL,
        session_end TIMESTAMP NOT NULL,
        total_duration_seconds INTEGER NOT NULL,
        completed BOOLEAN DEFAULT 1,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (material_id) REFERENCES review_materials (id) ON DELETE CASCADE
    )";

pub const CREATE_FLASHCARD_REVIEW_SESSIONS_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS flashcard_review_sessions (
        session_id TEXT PRIMARY KEY,
        cards_studied INTEGER NOT NULL,
        again_count INTEGER DEFAULT 0,
        hard_count INTEGER DEFAULT 0,
        good_count INTEGER DEFAULT 0,
        easy_count INTEGER DEFAULT 0,
        new_cards_count INTEGER DEFAULT 0,
        learning_cards_count INTEGER DEFAULT 0,
        review_cards_count INTEGER DEFAULT 0,
        retention_rate REAL DEFAULT 0.0,
        average_response_time_seconds REAL DEFAULT 0.0,
        FOREIGN KEY (session_id) REFERENCES review_sessions (id) ON DELETE CASCADE
    )";

pub const CREATE_TEST_REVIEW_SESSIONS_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS test_review_sessions (
        session_id TEXT PRIMARY KEY,
        questions_answered INTEGER NOT NULL,
        correct_answers INTEGER DEFAULT 0,
        incorrect_answers INTEGER DEFAULT 0,
        skipped_answers INTEGER DEFAULT 0,
        score_percentage REAL DEFAULT 0.0,
        time_per_question_seconds REAL DEFAULT 0.0,
        FOREIGN KEY (session_id) REFERENCES review_sessions (id) ON DELETE CASCADE
    )";

// Study material queries
pub const INSERT_STUDY_MATERIAL: &str =
    "
    INSERT INTO study_materials (id, name, original_path, markdown_path, cover_path, is_processing, extra_instructions, url) 
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)";

pub const GET_ALL_STUDY_MATERIALS: &str = "
    SELECT sm.id, sm.name, sm.original_path, sm.markdown_path, sm.cover_path, 
           sm.is_processing, sm.created_at, sm.extra_instructions,
           COALESCE(sm.is_indexed, 0) as is_indexed,
           sm.chunk_count, sm.last_indexed, sm.url
    FROM study_materials sm
    ORDER BY sm.created_at DESC";

pub const GET_STUDY_MATERIAL_BY_ID: &str =
    "
    SELECT id, name, original_path, markdown_path, cover_path, is_processing, created_at, extra_instructions,
           COALESCE(is_indexed, 0) as is_indexed, chunk_count, last_indexed, url
    FROM study_materials
    WHERE id = ?1";

pub const GET_STUDY_MATERIAL_BY_FILENAME: &str =
    "
    SELECT id, name, original_path, markdown_path, cover_path, is_processing, created_at, extra_instructions, url
    FROM study_materials
    WHERE original_path LIKE ?1";

pub const GET_STUDY_MATERIALS_INDEXED: &str =
    "SELECT id, name, original_path, markdown_path, cover_path, is_processing, created_at, extra_instructions, chunk_count, last_indexed, url
             FROM study_materials
             WHERE is_indexed = 1";

pub const GET_STUDY_MATERIALS_UNINDEXED: &str =
    "SELECT id, name, original_path, markdown_path, cover_path, is_processing, created_at, extra_instructions, url
             FROM study_materials
             WHERE is_indexed = 0 OR is_indexed IS NULL";

pub const UPDATE_STUDY_MATERIAL_PROCESSING: &str = "
    UPDATE study_materials 
    SET is_processing = ?1, updated_at = CURRENT_TIMESTAMP 
    WHERE id = ?2";

pub const UPDATE_STUDY_MATERIAL_NAME: &str = "
    UPDATE study_materials 
    SET name = ?1, updated_at = CURRENT_TIMESTAMP 
    WHERE id = ?2";

pub const UPDATE_STUDY_MATERIAL_INSTRUCTIONS: &str = "
    UPDATE study_materials 
    SET extra_instructions = ?1, updated_at = CURRENT_TIMESTAMP 
    WHERE id = ?2";

pub const UPDATE_STUDY_MATERIAL_INDEXED_STATE: &str =
    "
    UPDATE study_materials 
    SET is_indexed = ?1, chunk_count = ?2, last_indexed = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP 
    WHERE id = ?3";

pub const DELETE_STUDY_MATERIAL: &str = "
    DELETE FROM study_materials WHERE id = ?1";

// Review material queries
pub const INSERT_REVIEW_MATERIAL: &str = "
    INSERT INTO review_materials (id, name, type, last_review, created_at, updated_at) 
    VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)";

pub const GET_ALL_REVIEW_MATERIALS: &str = "
    SELECT id, name, type, last_review, created_at, updated_at, COALESCE(deleted, 0) as deleted 
    FROM review_materials 
    WHERE (deleted = 0 OR deleted IS NULL) 
    ORDER BY updated_at DESC";

pub const GET_REVIEW_MATERIAL_BY_ID: &str = "
    SELECT id, name, type, last_review, created_at, updated_at, COALESCE(deleted, 0) as deleted 
    FROM review_materials 
    WHERE id = ?1 AND (deleted = 0 OR deleted IS NULL)";

pub const UPDATE_REVIEW_MATERIAL_NAME: &str = "
    UPDATE review_materials 
    SET name = ?1, updated_at = CURRENT_TIMESTAMP 
    WHERE id = ?2";

pub const DELETE_REVIEW_MATERIAL: &str = "
    UPDATE review_materials 
    SET deleted = 1, updated_at = CURRENT_TIMESTAMP 
    WHERE id = ?1";

pub const GET_FLASHCARD_DECKS: &str = "
    SELECT id, name, last_review, created_at, updated_at 
            FROM review_materials
            WHERE type = 'flashcard_deck' AND (deleted = 0 OR deleted IS NULL)
            ORDER BY updated_at DESC";

pub const GET_FLASHCARD_DECK_BY_ID: &str =
    "
    SELECT name, last_review, created_at, updated_at FROM review_materials WHERE id = ?1 AND type = 'flashcard_deck' AND (deleted = 0 OR deleted IS NULL)";

pub const GET_RANDOM_FLASHCARD_DECK_ID: &str = "
    SELECT id FROM review_materials 
    WHERE type = 'flashcard_deck' AND (deleted = 0 OR deleted IS NULL)
    ORDER BY RANDOM() 
    LIMIT 1";

pub const GET_RANDOM_FLASHCARDS_FROM_DECK: &str = "
    SELECT id, deck_id, front, back, position, due_date, interval_days, 
           ease_factor, repetitions, status, last_reviewed, learning_step,
           created_at, updated_at
    FROM flashcards 
    WHERE deck_id = ?1 
    ORDER BY RANDOM() 
    LIMIT ?2";

// Flashcard queries
pub const INSERT_FLASHCARD: &str =
    "
    INSERT INTO flashcards (deck_id, front, back, position, due_date, status, created_at, updated_at) 
    VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP, 'new', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)";

pub const DELETE_FLASHCARD: &str = "
DELETE FROM flashcards WHERE id = ?1 AND deck_id = ?2";

pub const GET_FLASHCARDS_BY_DECK: &str = "
    SELECT id, deck_id, front, back, position FROM flashcards 
    WHERE deck_id = ?1 
    ORDER BY position";

pub const GET_FLASHCARD_BY_ID: &str =
    "
SELECT id, deck_id, front, back, position, due_date, interval_days, ease_factor, repetitions, status, last_reviewed, learning_step, created_at, updated_at FROM flashcards WHERE id = ?1";

pub const GET_FLASHCARDS_WITH_SCHEDULING: &str = "
    SELECT id, deck_id, front, back, position, due_date, interval_days, 
           ease_factor, repetitions, status, last_reviewed, learning_step,
           created_at, updated_at
    FROM flashcards 
    WHERE deck_id = ?1 
    ORDER BY position";

pub const UPDATE_FLASHCARD_SCHEDULING: &str = "
    UPDATE flashcards SET 
                due_date = ?1, 
                interval_days = ?2, 
                ease_factor = ?3, 
                repetitions = ?4,
                status = ?5, 
                last_reviewed = ?6, 
                learning_step = ?7, 
                updated_at = ?8
             WHERE id = ?9";

pub const UPDATE_EXISTING_FLASHCARD: &str = "
UPDATE flashcards SET front = ?1, back = ?2, position = ?3, updated_at = CURRENT_TIMESTAMP 
                     WHERE id = ?4 AND deck_id = ?5";

pub const GET_PENDING_FLASHCARDS: &str = "
    SELECT id, deck_id, front, back, position, due_date, interval_days, 
           ease_factor, repetitions, status, last_reviewed, learning_step,
           created_at, updated_at
    FROM flashcards 
    WHERE deck_id = ?1 AND due_date <= CURRENT_TIMESTAMP
    ORDER BY due_date ASC";

pub const GET_FLASHCARD_COUNTS: &str = "
    SELECT 
        COUNT(CASE WHEN status = 'new' THEN 1 END) as new_count,
        COUNT(CASE WHEN status = 'learning' THEN 1 END) as learning_count,
        COUNT(CASE WHEN status = 'review' THEN 1 END) as review_count,
        COUNT(CASE WHEN due_date <= CURRENT_TIMESTAMP THEN 1 END) as pending_count,
        COUNT(*) as total_count
    FROM flashcards 
    WHERE deck_id = ?1";

pub const UPDATE_FLASHCARD_DECK_NAME: &str = "
    UPDATE review_materials SET name = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2";

pub const UPDATE_LAST_REVIEW: &str =
    "
    UPDATE review_materials SET last_review = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = ?1";

pub const UPDATE_REVIEW_MATERIAL_TIMESTAMP: &str = "
    UPDATE review_materials SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1";

// Tag queries
pub const INSERT_TAG: &str = "
    INSERT OR IGNORE INTO tags (name) VALUES (?1)";

pub const GET_TAG_ID: &str = "
    SELECT id FROM tags WHERE name = ?1";

pub const INSERT_MATERIAL_TAG: &str = "
    INSERT INTO material_tags (material_id, tag_id) VALUES (?1, ?2)";

pub const GET_TAGS_BY_MATERIAL: &str = "
    SELECT t.name FROM tags t
    JOIN material_tags mt ON t.id = mt.tag_id
    WHERE mt.material_id = ?1";

pub const DELETE_MATERIAL_TAGS: &str = "
    DELETE FROM material_tags WHERE material_id = ?1";

pub const GET_ALL_TAGS: &str = "
    SELECT name FROM tags
    ORDER BY name";

pub const GET_AVAILABLE_TAGS: &str = "
    SELECT name FROM tags 
    WHERE is_available = 1
    ORDER BY name";

pub const INSERT_AVAILABLE_TAG: &str = "
    INSERT OR IGNORE INTO tags (name, is_available) VALUES (?1, 1)";

pub const DELETE_AVAILABLE_TAG: &str = "
    UPDATE tags SET is_available = 0 WHERE name = ?1";

pub const CHECK_MATERIAL_HAS_TAG: &str = "
    SELECT 1 FROM material_tags 
    WHERE material_id = ?1 AND tag_id = ?2
    LIMIT 1";

pub const DELETE_MATERIAL_TAG: &str = "
    DELETE FROM material_tags 
    WHERE material_id = ?1 AND tag_id = ?2";

// Review session queries
pub const INSERT_REVIEW_SESSION: &str = "
    INSERT INTO review_sessions (
        id, material_id, material_type, session_start, session_end, 
        total_duration_seconds, completed
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)";

pub const INSERT_FLASHCARD_REVIEW_SESSION: &str = "
    INSERT INTO flashcard_review_sessions (
        session_id, cards_studied, again_count, hard_count, 
        good_count, easy_count, new_cards_count, learning_cards_count, 
        review_cards_count, retention_rate, average_response_time_seconds
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)";

pub const INSERT_TEST_REVIEW_SESSION: &str = "
    INSERT INTO test_review_sessions (
        session_id, questions_answered, correct_answers, incorrect_answers,
        skipped_answers, score_percentage, time_per_question_seconds
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)";

pub const GET_FLASHCARD_REVIEW_SESSIONS_BY_MATERIAL: &str = "
    SELECT rs.id, rs.material_id, rs.material_type, rs.session_start, rs.session_end,
           rs.total_duration_seconds, rs.completed, rs.created_at,
           frs.cards_studied, frs.again_count, frs.hard_count, frs.good_count,
           frs.easy_count, frs.new_cards_count, frs.learning_cards_count,
           frs.review_cards_count, frs.retention_rate, frs.average_response_time_seconds,
           rm.name as material_name
    FROM review_sessions rs
    JOIN flashcard_review_sessions frs ON rs.id = frs.session_id
    LEFT JOIN review_materials rm ON rs.material_id = rm.id
    WHERE rs.material_id = ?1 AND rs.material_type = 'flashcard_deck'
    ORDER BY rs.session_start DESC";

pub const GET_TEST_REVIEW_SESSIONS_BY_MATERIAL: &str = "
    SELECT rs.id, rs.material_id, rs.material_type, rs.session_start, rs.session_end,
           rs.total_duration_seconds, rs.completed, rs.created_at,
           trs.questions_answered, trs.correct_answers, trs.incorrect_answers,
           trs.skipped_answers, trs.score_percentage, trs.time_per_question_seconds,
           rm.name as material_name
    FROM review_sessions rs
    JOIN test_review_sessions trs ON rs.id = trs.session_id
    LEFT JOIN review_materials rm ON rs.material_id = rm.id
    WHERE rs.material_id = ?1 AND rs.material_type = 'test'
    ORDER BY rs.session_start DESC";

pub const GET_ALL_REVIEW_SESSIONS: &str = "
    SELECT rs.id, rs.material_id, rs.material_type, rs.session_start, rs.session_end,
           rs.total_duration_seconds, rs.completed, rs.created_at,
           rm.name as material_name
    FROM review_sessions rs
    LEFT JOIN review_materials rm ON rs.material_id = rm.id
    ORDER BY rs.session_start DESC";

pub const GET_REVIEW_SESSION_STATS: &str = "
    SELECT 
        COUNT(*) as total_sessions,
        SUM(rs.total_duration_seconds) as total_study_time,
        AVG(CASE WHEN rs.material_type = 'flashcard_deck' THEN frs.cards_studied END) as avg_cards_per_session,
        AVG(CASE WHEN rs.material_type = 'flashcard_deck' THEN frs.retention_rate END) as avg_flashcard_retention,
        AVG(CASE WHEN rs.material_type = 'test' THEN trs.score_percentage END) as avg_test_score,
        MAX(rs.session_start) as last_session,
        COUNT(DISTINCT rs.material_id) as materials_studied,
        COUNT(CASE WHEN rs.material_type = 'flashcard_deck' THEN 1 END) as flashcard_sessions,
        COUNT(CASE WHEN rs.material_type = 'test' THEN 1 END) as test_sessions
    FROM review_sessions rs
    LEFT JOIN flashcard_review_sessions frs ON rs.id = frs.session_id AND rs.material_type = 'flashcard_deck'
    LEFT JOIN test_review_sessions trs ON rs.id = trs.session_id AND rs.material_type = 'test'
    WHERE rs.completed = 1";

pub const GET_RECENT_REVIEW_SESSIONS: &str = "
    SELECT 
        rs.id, 
        rs.material_id, 
        rs.material_type, 
        rs.session_start, 
        rs.session_end,
        rs.total_duration_seconds, 
        rm.name as material_name,
        rs.completed,
        
        -- Flashcard-specific fields (NULL for test sessions)
        frs.cards_studied,
        frs.again_count,
        frs.hard_count,
        frs.good_count,
        frs.easy_count,
        frs.new_cards_count,
        frs.learning_cards_count,
        frs.review_cards_count,
        frs.retention_rate,
        frs.average_response_time_seconds,
        
        -- Test-specific fields (NULL for flashcard sessions)
        trs.questions_answered,
        trs.correct_answers,
        trs.incorrect_answers,
        trs.skipped_answers,
        trs.score_percentage,
        trs.time_per_question_seconds
        
    FROM review_sessions rs
    LEFT JOIN review_materials rm ON rs.material_id = rm.id
    LEFT JOIN flashcard_review_sessions frs ON rs.id = frs.session_id AND rs.material_type = 'flashcard_deck'
    LEFT JOIN test_review_sessions trs ON rs.id = trs.session_id AND rs.material_type = 'test'
    WHERE rs.completed = 1
    ORDER BY rs.session_start DESC
    LIMIT ?1";

// Test-related queries
pub const CREATE_TEST_QUESTIONS_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS test_questions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        test_id TEXT NOT NULL,
        question TEXT NOT NULL,
        position INTEGER NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (test_id) REFERENCES review_materials (id) ON DELETE CASCADE
    )";

pub const CREATE_TEST_ANSWERS_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS test_answers (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        question_id INTEGER NOT NULL,
        answer_text TEXT NOT NULL,
        is_correct BOOLEAN NOT NULL DEFAULT 0,
        position INTEGER NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (question_id) REFERENCES test_questions (id) ON DELETE CASCADE
    )";

pub const GET_RANDOM_TEST_ID: &str = "
    SELECT id FROM review_materials 
    WHERE type = 'test' AND (deleted = 0 OR deleted IS NULL)
    ORDER BY RANDOM() 
    LIMIT 1";

pub const GET_RANDOM_TEST_QUESTIONS: &str = "
    SELECT id, test_id, question, position, created_at, updated_at
    FROM test_questions 
    WHERE test_id = ?1 
    ORDER BY RANDOM() 
    LIMIT ?2";

// Test operation queries
pub const INSERT_TEST_QUESTION: &str = "
    INSERT INTO test_questions (test_id, question, position) 
    VALUES (?1, ?2, ?3)";

pub const INSERT_TEST_ANSWER: &str = "
    INSERT INTO test_answers (question_id, answer_text, is_correct, position) 
    VALUES (?1, ?2, ?3, ?4)";

pub const GET_TEST_QUESTIONS: &str = "
    SELECT id, test_id, question, position, created_at, updated_at
    FROM test_questions 
    WHERE test_id = ?1 
    ORDER BY position";

pub const GET_TEST_ANSWERS_BY_QUESTION: &str = "
    SELECT id, question_id, answer_text, is_correct, position, created_at, updated_at
    FROM test_answers 
    WHERE question_id = ?1 
    ORDER BY position";

pub const DELETE_TEST_QUESTION: &str = "
    DELETE FROM test_questions WHERE id = ?1 AND test_id = ?2";

pub const DELETE_TEST_ANSWER: &str = "
    DELETE FROM test_answers WHERE id = ?1";

pub const UPDATE_TEST_QUESTION: &str = "
    UPDATE test_questions SET question = ?1, position = ?2, updated_at = CURRENT_TIMESTAMP 
    WHERE id = ?3 AND test_id = ?4";

pub const UPDATE_TEST_ANSWER: &str = "
    UPDATE test_answers SET answer_text = ?1, is_correct = ?2, position = ?3, updated_at = CURRENT_TIMESTAMP 
    WHERE id = ?4";

pub const GET_TEST_BY_ID: &str = "
    SELECT name, last_review, created_at, updated_at 
    FROM review_materials 
    WHERE id = ?1 AND type = 'test' AND (deleted = 0 OR deleted IS NULL)";

pub const GET_ALL_TESTS: &str = "
    SELECT id, name, last_review, created_at, updated_at 
    FROM review_materials
    WHERE type = 'test' AND (deleted = 0 OR deleted IS NULL)
    ORDER BY updated_at DESC";

pub const UPDATE_TEST_NAME: &str = "
    UPDATE review_materials SET name = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2 AND type = 'test'";
