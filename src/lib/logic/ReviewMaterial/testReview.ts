export interface TestAnswer {
  id: number;
  question_id: number;
  answer_text: string;
  is_correct: boolean;
  position: number;
}

export interface TestQuestion {
  id: number;
  test_id: string;
  question: string;
  position: number;
}

export interface TestQuestionWithAnswers {
  question: TestQuestion;
  answers: TestAnswer[];
}

export interface TestSession {
  sessionStart: Date;
  questionsAnswered: number;
  timeSpent: number;
  correctAnswers: number;
  incorrectAnswers: number;
  skippedQuestions: number;
  userAnswers: Map<number, number[]>; // question ID -> selected answer IDs
}

export interface TestReviewSettings {
  shuffle_questions: boolean;
  shuffle_answers: boolean;
}

export class TestReviewLogic {
  private questions: TestQuestionWithAnswers[] = [];
  private originalQuestions: TestQuestionWithAnswers[] = [];
  public currentIndex: number = 0; 
  private session: TestSession;
  private settings: TestReviewSettings;
  private answeredQuestions: Set<number> = new Set();

  constructor(questions: TestQuestionWithAnswers[], settings?: Partial<TestReviewSettings>) {
    console.log('Initializing TestReviewLogic with questions:', questions);
    console.log('Settings:', settings);
    
    const validatedQuestions = questions.map((questionData, index) => {
      console.log(`Validating question ${index}:`, {
        id: questionData.question.id,
        idType: typeof questionData.question.id,
        text: questionData.question.question?.substring(0, 30) + '...',
        answersCount: questionData.answers?.length
      });
      
      if (questionData.question.id == null || questionData.question.id === undefined) {
        console.error(`ERROR: Question ${index} has null/undefined ID:`, questionData);
        throw new Error(`Question ${index} has invalid ID: ${questionData.question.id}`);
      }
      
      if (typeof questionData.question.id !== 'number') {
        console.warn(`WARNING: Question ${index} ID is not a number:`, questionData.question.id, typeof questionData.question.id);
        // Try to convert string ID to number
        if (typeof questionData.question.id === 'string' && !isNaN(Number(questionData.question.id))) {
          console.log(`Converting string ID to number for question ${index}`);
          return {
            ...questionData,
            question: { ...questionData.question, id: Number(questionData.question.id) }
          };
        } else {
          throw new Error(`Question ${index} has invalid ID type: ${typeof questionData.question.id}`);
        }
      }
      
      return questionData;
    });
    
    this.originalQuestions = [...validatedQuestions];
    this.questions = [...validatedQuestions];
    this.session = this.initializeSession();
    this.settings = {
      shuffle_questions: false,
      shuffle_answers: true,
      ...settings
    };
    
    // Apply shuffling based on settings
    if (this.settings.shuffle_questions) {
      this.shuffleQuestions();
    }
    
    if (this.settings.shuffle_answers) {
      this.shuffleAnswers();
    }
    
    console.log('TestReviewLogic initialized successfully with', this.questions.length, 'questions');
  }

  private initializeSession(): TestSession {
    return {
      sessionStart: new Date(),
      questionsAnswered: 0,
      timeSpent: 0,
      correctAnswers: 0,
      incorrectAnswers: 0,
      skippedQuestions: 0,
      userAnswers: new Map()
    };
  }

  private shuffleQuestions(): void {
    console.log('Shuffling questions');
    for (let i = this.questions.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [this.questions[i], this.questions[j]] = [this.questions[j], this.questions[i]];
    }
  }

  private shuffleAnswers(): void {
    console.log('Shuffling answers for each question');
    this.questions = this.questions.map(questionData => ({
      ...questionData,
      answers: this.shuffleArray([...questionData.answers])
    }));
  }

  private shuffleArray<T>(array: T[]): T[] {
    const shuffled = [...array];
    for (let i = shuffled.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
    }
    return shuffled;
  }

  getCurrentQuestion(): TestQuestionWithAnswers | null {
    const question = this.questions[this.currentIndex] || null;
    
    if (question) {
      console.log('Getting current question:', {
        index: this.currentIndex,
        id: question.question.id,
        idType: typeof question.question.id,
        text: question.question.question?.substring(0, 30) + '...',
        answersCount: question.answers?.length
      });
      
      // Validate the question ID
      if (question.question.id == null || question.question.id === undefined) {
        console.error('ERROR: Current question has null/undefined ID:', question);
        throw new Error('Current question has invalid ID');
      }
      
      if (typeof question.question.id !== 'number') {
        console.error('ERROR: Current question ID is not a number:', question.question.id, typeof question.question.id);
        throw new Error(`Current question has invalid ID type: ${typeof question.question.id}`);
      }
    } else {
      console.log('No current question at index:', this.currentIndex);
    }
    
    return question;
  }

  getProgress(): { current: number; total: number; percentage: number } {
    const progress = {
      current: this.currentIndex + 1,
      total: this.questions.length,
      percentage: this.questions.length > 0 ? ((this.currentIndex + 1) / this.questions.length) * 100 : 0
    };
    
    console.log('Current progress:', progress);
    return progress;
  }

  getSession(): TestSession {
    return { 
      ...this.session,
      userAnswers: new Map(this.session.userAnswers) // Return a copy
    };
  }

  canMoveNext(): boolean {
    const canMove = this.currentIndex < this.questions.length - 1;
    console.log(`Can move next: ${canMove} (current: ${this.currentIndex}, total: ${this.questions.length})`);
    return canMove;
  }

  canMovePrevious(): boolean {
    const canMove = this.currentIndex > 0;
    console.log(`Can move previous: ${canMove} (current: ${this.currentIndex})`);
    return canMove;
  }

  moveNext(): boolean {
    if (this.canMoveNext()) {
      this.currentIndex++;
      console.log(`Moved to next question. New index: ${this.currentIndex}`);
      return true;
    }
    console.log('Cannot move to next question');
    return false;
  }

  movePrevious(): boolean {
    if (this.canMovePrevious()) {
      this.currentIndex--;
      console.log(`Moved to previous question. New index: ${this.currentIndex}`);
      return true;
    }
    console.log('Cannot move to previous question');
    return false;
  }

  submitAnswer(selectedAnswerIds: number[]): boolean {
    const currentQuestion = this.getCurrentQuestion();
    if (!currentQuestion) {
      throw new Error('No current question to answer');
    }
    
    const questionId = currentQuestion.question.id;
    console.log(`Submitting answer for question ${questionId} with answers:`, selectedAnswerIds);
    
    // Store the user's answer
    this.session.userAnswers.set(questionId, [...selectedAnswerIds]);
    
    // Check if this question was already answered
    const wasAlreadyAnswered = this.answeredQuestions.has(questionId);
    if (!wasAlreadyAnswered) {
      this.session.questionsAnswered++;
      this.answeredQuestions.add(questionId);
      console.log(`New question answered. Total questions answered: ${this.session.questionsAnswered}`);
    } else {
      console.log(`Question ${questionId} was already answered, not incrementing questionsAnswered`);
    }
    
    // Determine if the answer is correct
    const correctAnswerIds = currentQuestion.answers
      .filter(answer => answer.is_correct)
      .map(answer => answer.id);
    
    const isCorrect = this.arraysEqual(selectedAnswerIds.sort(), correctAnswerIds.sort());
    
    if (!wasAlreadyAnswered) {
      if (isCorrect) {
        this.session.correctAnswers++;
        console.log(`Correct answer! Total correct: ${this.session.correctAnswers}`);
      } else {
        this.session.incorrectAnswers++;
        console.log(`Incorrect answer. Total incorrect: ${this.session.incorrectAnswers}`);
      }
    }
    
    console.log('Session after answer submission:', {
      questionsAnswered: this.session.questionsAnswered,
      correctAnswers: this.session.correctAnswers,
      incorrectAnswers: this.session.incorrectAnswers,
      answeredQuestionIds: Array.from(this.answeredQuestions)
    });
    
    return isCorrect;
  }

  private arraysEqual(a: number[], b: number[]): boolean {
    if (a.length !== b.length) return false;
    return a.every((val, index) => val === b[index]);
  }

  skipQuestion(): void {
    const currentQuestion = this.getCurrentQuestion();
    if (!currentQuestion) return;
    
    const questionId = currentQuestion.question.id;
    console.log(`Skipping question ${questionId}`);
    
    const wasAlreadyAnswered = this.answeredQuestions.has(questionId);
    if (!wasAlreadyAnswered) {
      this.session.skippedQuestions++;
      this.session.questionsAnswered++;
      this.answeredQuestions.add(questionId);
      console.log(`Question skipped. Total skipped: ${this.session.skippedQuestions}`);
    }
  }

  getUserAnswer(questionId: number): number[] {
    return this.session.userAnswers.get(questionId) || [];
  }

  isQuestionAnswered(questionId: number): boolean {
    return this.answeredQuestions.has(questionId);
  }

  getCorrectAnswerIds(questionId: number): number[] {
    const question = this.questions.find(q => q.question.id === questionId);
    if (!question) return [];
    
    return question.answers
      .filter(answer => answer.is_correct)
      .map(answer => answer.id);
  }

  isAnswerCorrect(questionId: number): boolean {
    const userAnswerIds = this.getUserAnswer(questionId);
    const correctAnswerIds = this.getCorrectAnswerIds(questionId);
    
    return this.arraysEqual(userAnswerIds.sort(), correctAnswerIds.sort());
  }

  updateSessionTime(seconds: number): void {
    this.session.timeSpent = seconds;
  }

  reset(): void {
    console.log('Resetting TestReviewLogic');
    this.currentIndex = 0;
    this.session = this.initializeSession();
    this.questions = [...this.originalQuestions];
    this.answeredQuestions.clear();
    
    // Reapply shuffling if enabled
    if (this.settings.shuffle_questions) {
      this.shuffleQuestions();
    }
    
    if (this.settings.shuffle_answers) {
      this.shuffleAnswers();
    }
    
    console.log('Reset complete. Questions:', this.questions.length);
  }

  static formatTestTime(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    
    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
    return `${minutes}:${secs.toString().padStart(2, '0')}`;
  }

  getScorePercentage(): number {
    const totalAnswered = this.session.correctAnswers + this.session.incorrectAnswers;
    if (totalAnswered === 0) return 0;
    return Math.round((this.session.correctAnswers / totalAnswered) * 100);
  }

  getAccuracyRate(): number {
    return this.getScorePercentage();
  }

  getCompletionPercentage(): number {
    if (this.questions.length === 0) return 0;
    return Math.round((this.session.questionsAnswered / this.questions.length) * 100);
  }

  // Get detailed results for review
  getDetailedResults(): Array<{
    question: TestQuestionWithAnswers;
    userAnswerIds: number[];
    correctAnswerIds: number[];
    isCorrect: boolean;
    wasSkipped: boolean;
  }> {
    return this.questions.map(questionData => {
      const questionId = questionData.question.id;
      const userAnswerIds = this.getUserAnswer(questionId);
      const correctAnswerIds = this.getCorrectAnswerIds(questionId);
      const isCorrect = this.isAnswerCorrect(questionId);
      const wasAnswered = this.isQuestionAnswered(questionId);
      const wasSkipped = wasAnswered && userAnswerIds.length === 0;
      
      return {
        question: questionData,
        userAnswerIds,
        correctAnswerIds,
        isCorrect,
        wasSkipped
      };
    });
  }

  // Helper method to get statistics
  getStatistics(): {
    totalQuestions: number;
    questionsAnswered: number;
    correctAnswers: number;
    incorrectAnswers: number;
    skippedQuestions: number;
    scorePercentage: number;
    completionPercentage: number;
    timeSpent: number;
  } {
    return {
      totalQuestions: this.questions.length,
      questionsAnswered: this.session.questionsAnswered,
      correctAnswers: this.session.correctAnswers,
      incorrectAnswers: this.session.incorrectAnswers,
      skippedQuestions: this.session.skippedQuestions,
      scorePercentage: this.getScorePercentage(),
      completionPercentage: this.getCompletionPercentage(),
      timeSpent: this.session.timeSpent
    };
  }

  // Get the number of questions remaining in the test
  getRemainingQuestionsCount(): number {
    return this.questions.length - this.currentIndex - 1;
  }

  // Check if the test is complete
  isTestComplete(): boolean {
    return this.session.questionsAnswered >= this.questions.length;
  }
}