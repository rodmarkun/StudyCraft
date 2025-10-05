import { invoke } from '@tauri-apps/api/core';
import type { StudyMaterial, GeneratedQuestion } from './testCreator';

export interface ReviewMaterial {
  id: string;
  name: string;
  type: 'flashcard_deck' | 'test';
  cards_count?: number;
  questions_count?: number;
  last_review?: string;
  tags: string[];
}

export interface EditableTestAnswer {
  id?: number | null;
  answer_text: string;
  is_correct: boolean;
  position: number;
  isNew?: boolean;
  isModified?: boolean;
  isDeleted?: boolean;
}

export interface EditableTestQuestion {
  id?: number | null;
  question: string;
  answers: EditableTestAnswer[];
  position?: number;
  isNew?: boolean;
  isModified?: boolean;
  isDeleted?: boolean;
  tempId?: string;
}

export interface TestEditFormData {
  testId: string;
  name: string;
  language: string;
  selectedMaterials: StudyMaterial[];
  concept: string;
}

export interface ExistingTestQuestion {
  question: {
    id: number;
    test_id: string;
    question: string;
    position: number;
  };
  answers: Array<{
    id: number;
    question_id: number;
    answer_text: string;
    is_correct: boolean;
    position: number;
  }>;
}

export interface TestUpdateRequest {
  test_id: string;
  name?: string;
  questions_to_add?: Array<{
    question: string;
    position: number;
    answers: Array<{
      answer_text: string;
      is_correct: boolean;
      position: number;
    }>;
  }>;
  questions_to_update?: Array<{
    id: number;
    question: string;
    position: number;
    answers: Array<{
      id?: number;
      answer_text: string;
      is_correct: boolean;
      position: number;
    }>;
  }>;
  questions_to_delete?: number[];
}

export class TestEditorLogic {
  private questions: EditableTestQuestion[] = [];
  private originalQuestions: EditableTestQuestion[] = [];
  private isLoading: boolean = false;
  private isSaving: boolean = false;
  private isGenerating: boolean = false;
  private error: string = '';
  private generationStatus: string = '';
  private hasUnsavedChanges: boolean = false;
  private originalTestName: string = '';
  private nextTempId: number = 1;
  public currentStep: number = 2;

  get getQuestions(): EditableTestQuestion[] {
    return [...this.questions];
  }

  get getOriginalQuestions(): EditableTestQuestion[] {
    return [...this.originalQuestions];
  }

  get getIsLoading(): boolean {
    return this.isLoading;
  }

  get getIsSaving(): boolean {
    return this.isSaving;
  }

  get getIsGenerating(): boolean {
    return this.isGenerating;
  }

  get getError(): string {
    return this.error;
  }

  get getGenerationStatus(): string {
    return this.generationStatus;
  }

  get getHasUnsavedChanges(): boolean {
    return this.hasUnsavedChanges;
  }

  moveToNextStep(): void {
    this.currentStep = Math.min(this.currentStep + 1, 3);
  }

  moveToPreviousStep(): void {
    this.currentStep = Math.max(this.currentStep - 1, 2); 
  }

  getCurrentStep(): number {
    return this.currentStep;
  }

  // Helper function to check if a question has changed
  private hasQuestionChanged(originalQuestion: EditableTestQuestion, currentQuestion: EditableTestQuestion): boolean {
    // Check if question text changed
    if (originalQuestion.question !== currentQuestion.question) {
      return true;
    }
    
    // Check if answers changed
    if (originalQuestion.answers.length !== currentQuestion.answers.length) {
      return true;
    }
    
    // Check each answer for changes
    for (let i = 0; i < originalQuestion.answers.length; i++) {
      const originalAnswer = originalQuestion.answers[i];
      const currentAnswer = currentQuestion.answers[i];
      
      if (originalAnswer.answer_text !== currentAnswer.answer_text ||
          originalAnswer.is_correct !== currentAnswer.is_correct) {
        return true;
      }
    }
    
    return false;
  }

  // Build update request with proper change detection
  buildUpdateRequest(formData: TestEditFormData): TestUpdateRequest {
    const questionsToUpdate = [];
    const questionsToDelete = [];
    const questionsToAdd = [];

    console.log('DEBUG: Building update request...');
    console.log('DEBUG: Original questions count:', this.originalQuestions.length);
    console.log('DEBUG: Current questions count:', this.questions.length);

    // Find changed existing questions
    this.questions.forEach((currentQuestion, index) => {
      console.log(`DEBUG: Processing question ${index} (ID: ${currentQuestion.id})`);
      
      if (currentQuestion.isNew) {
        // This is a new question
        console.log(`DEBUG: Question ${index} is new, adding to questionsToAdd`);
        questionsToAdd.push({
          question: currentQuestion.question,
          position: index,
          answers: currentQuestion.answers.map((answer, answerIndex) => ({
            answer_text: answer.answer_text,
            is_correct: answer.is_correct,
            position: answerIndex
          }))
        });
      } else if (currentQuestion.isDeleted) {
        // This question should be deleted
        console.log(`DEBUG: Question ${index} is deleted, adding to questionsToDelete`);
        questionsToDelete.push(currentQuestion.id);
      } else {
        // Check if existing question changed
        const originalQuestion = this.originalQuestions.find(q => q.id === currentQuestion.id);
        console.log(`DEBUG: Found original question for ID ${currentQuestion.id}:`, !!originalQuestion);
        
        if (originalQuestion) {
          const hasChanged = this.hasQuestionChanged(originalQuestion, currentQuestion);
          console.log(`DEBUG: Question ${currentQuestion.id} has changes: ${hasChanged}`);
          
          if (hasChanged) {
            console.log(`DEBUG: Adding question ${currentQuestion.id} to update list`);
            console.log('DEBUG: Original question:', originalQuestion.question);
            console.log('DEBUG: Current question:', currentQuestion.question);
            console.log('DEBUG: Original answers:', originalQuestion.answers);
            console.log('DEBUG: Current answers:', currentQuestion.answers);
            
            questionsToUpdate.push({
              id: currentQuestion.id,
              question: currentQuestion.question,
              position: index,
              answers: currentQuestion.answers.map((answer, answerIndex) => ({
                id: answer.id, // Preserve answer ID for updates
                answer_text: answer.answer_text,
                is_correct: answer.is_correct,
                position: answerIndex
              }))
            });
          }
        }
      }
    });

    const updateRequest = {
      test_id: formData.testId,
      name: formData.name,
      questions_to_update: questionsToUpdate.length > 0 ? questionsToUpdate : undefined,
      questions_to_delete: questionsToDelete.length > 0 ? questionsToDelete : undefined,
      questions_to_add: questionsToAdd.length > 0 ? questionsToAdd : undefined
    };

    console.log('DEBUG: Final update request:', updateRequest);
    return updateRequest;
  }

  async loadExistingQuestions(testId: string): Promise<void> {
    this.isLoading = true;
    this.error = '';

    try {
      console.log('Loading existing questions for test:', testId);
      
      const existingQuestions = await invoke('get_test_questions', {
        testId: testId
      }) as ExistingTestQuestion[];

      console.log('Loaded existing questions:', existingQuestions);

      this.questions = existingQuestions
        .sort((a, b) => a.question.position - b.question.position)
        .map((questionData, index) => ({
          id: questionData.question.id,
          question: questionData.question.question,
          answers: questionData.answers
            .sort((a, b) => a.position - b.position)
            .map(answer => ({
              id: answer.id,
              answer_text: answer.answer_text,
              is_correct: answer.is_correct,
              position: answer.position,
              isNew: false,
              isModified: false,
              isDeleted: false
            })),
          position: questionData.question.position,
          isNew: false,
          isModified: false,
          isDeleted: false,
          tempId: `existing_${questionData.question.id}_${index}`
        }));

      // Store original state for change detection
      this.originalQuestions = JSON.parse(JSON.stringify(this.questions));
      this.hasUnsavedChanges = false;
      this.nextTempId = 1000; // Start temp IDs high to avoid conflicts

      console.log('Processed questions:', this.questions);
    } catch (err) {
      console.error('Failed to load existing questions:', err);
      this.error = `Failed to load existing questions: ${err}`;
    } finally {
      this.isLoading = false;
    }
  }

  isFormValid(formData: TestEditFormData): boolean {
    return formData.name.trim() !== '' && this.questions.some(question => !question.isDeleted) && 
           this.questions.filter(question => !question.isDeleted).every(question => 
             question.question.trim() !== '' && 
             question.answers.filter(answer => !answer.isDeleted).length >= 2 &&
             question.answers.filter(answer => !answer.isDeleted).some(answer => answer.is_correct) &&
             question.answers.filter(answer => !answer.isDeleted).every(answer => answer.answer_text.trim() !== '')
           );
  }

  addQuestion(): void {
    const newQuestion: EditableTestQuestion = {
      id: null,
      question: '',
      answers: [
        {
          id: null,
          answer_text: '',
          is_correct: true,
          position: 0,
          isNew: true,
          isModified: false,
          isDeleted: false
        },
        {
          id: null,
          answer_text: '',
          is_correct: false,
          position: 1,
          isNew: true,
          isModified: false,
          isDeleted: false
        }
      ],
      position: this.questions.filter(question => !question.isDeleted).length,
      isNew: true,
      isModified: false,
      isDeleted: false,
      tempId: `temp_${this.nextTempId++}`
    };
    
    this.questions = [...this.questions, newQuestion];
    this.updateQuestionPositions();
    this.checkForChanges();
  }

  addQuestionWithContent(question: string, answers: Array<{answer_text: string; is_correct: boolean; position: number}>): void {
    const newQuestion: EditableTestQuestion = {
      id: null,
      question,
      answers: answers.map((answer, index) => ({
        id: null,
        answer_text: answer.answer_text,
        is_correct: answer.is_correct,
        position: index,
        isNew: true,
        isModified: false,
        isDeleted: false
      })),
      position: this.questions.filter(question => !question.isDeleted).length,
      isNew: true,
      isModified: false,
      isDeleted: false,
      tempId: `temp_${this.nextTempId++}`
    };
    
    this.questions = [...this.questions, newQuestion];
    this.updateQuestionPositions();
    this.checkForChanges();
  }

  removeQuestion(index: number): void {
    const visibleQuestions = this.questions.filter(question => !question.isDeleted);
    const questionToRemove = visibleQuestions[index];
    
    if (!questionToRemove) return;
    
    const actualIndex = this.questions.findIndex(question => question === questionToRemove);
    
    if (questionToRemove.isNew) {
      this.questions = this.questions.filter((_, i) => i !== actualIndex);
    } else {
      this.questions = this.questions.map((question, i) => 
        i === actualIndex ? { ...questionToRemove, isDeleted: true } : question
      );
    }
    
    this.updateQuestionPositions();
    this.checkForChanges();
  }

  updateQuestion(index: number, value: string): void {
    const visibleQuestions = this.questions.filter(question => !question.isDeleted);
    const questionToUpdate = visibleQuestions[index];
    
    if (!questionToUpdate) return;
    
    const actualIndex = this.questions.findIndex(question => question === questionToUpdate);
    
    this.questions = this.questions.map((question, i) => 
      i === actualIndex ? {
        ...questionToUpdate,
        question: value,
        isModified: !questionToUpdate.isNew
      } : question
    );
    
    this.checkForChanges();
  }

  updateAnswer(questionIndex: number, answerIndex: number, field: 'answer_text' | 'is_correct', value: string | boolean): void {
    const visibleQuestions = this.questions.filter(question => !question.isDeleted);
    const questionToUpdate = visibleQuestions[questionIndex];
    
    if (!questionToUpdate) return;
    
    const actualQuestionIndex = this.questions.findIndex(question => question === questionToUpdate);
    const visibleAnswers = questionToUpdate.answers.filter(answer => !answer.isDeleted);
    const answerToUpdate = visibleAnswers[answerIndex];
    
    if (!answerToUpdate) return;
    
    const actualAnswerIndex = questionToUpdate.answers.findIndex(answer => answer === answerToUpdate);
    
    if (field === 'is_correct' && value === true) {
      // Set all others to false when marking one as correct
      this.questions = this.questions.map((question, qIndex) => 
        qIndex === actualQuestionIndex ? {
          ...question,
          answers: question.answers.map((answer, aIndex) => ({
            ...answer,
            is_correct: aIndex === actualAnswerIndex,
            isModified: !answer.isNew
          }))
        } : question
      );
    } else {
      this.questions = this.questions.map((question, qIndex) => 
        qIndex === actualQuestionIndex ? {
          ...question,
          answers: question.answers.map((answer, aIndex) => 
            aIndex === actualAnswerIndex ? {
              ...answer,
              [field]: value,
              isModified: !answer.isNew
            } : answer
          )
        } : question
      );
    }
    
    this.checkForChanges();
  }

  addAnswer(questionIndex: number): void {
    const visibleQuestions = this.questions.filter(question => !question.isDeleted);
    const questionToUpdate = visibleQuestions[questionIndex];
    
    if (!questionToUpdate) return;
    
    const actualIndex = this.questions.findIndex(question => question === questionToUpdate);
    const visibleAnswers = questionToUpdate.answers.filter(answer => !answer.isDeleted);
    
    const newAnswer: EditableTestAnswer = {
      id: null,
      answer_text: '',
      is_correct: false,
      position: visibleAnswers.length,
      isNew: true,
      isModified: false,
      isDeleted: false
    };
    
    this.questions = this.questions.map((question, i) => 
      i === actualIndex ? {
        ...question,
        answers: [...question.answers, newAnswer]
      } : question
    );
    
    this.checkForChanges();
  }

  removeAnswer(questionIndex: number, answerIndex: number): void {
    const visibleQuestions = this.questions.filter(question => !question.isDeleted);
    const questionToUpdate = visibleQuestions[questionIndex];
    
    if (!questionToUpdate) return;
    
    const actualQuestionIndex = this.questions.findIndex(question => question === questionToUpdate);
    const visibleAnswers = questionToUpdate.answers.filter(answer => !answer.isDeleted);
    
    if (visibleAnswers.length <= 2) return; // Don't allow removing if only 2 answers left
    
    const answerToRemove = visibleAnswers[answerIndex];
    if (!answerToRemove) return;
    
    const actualAnswerIndex = questionToUpdate.answers.findIndex(answer => answer === answerToRemove);
    
    if (answerToRemove.isNew) {
      // Remove completely if it's new
      this.questions = this.questions.map((question, qIndex) => 
        qIndex === actualQuestionIndex ? {
          ...question,
          answers: question.answers.filter((_, aIndex) => aIndex !== actualAnswerIndex)
        } : question
      );
    } else {
      // Mark as deleted if it exists in database
      this.questions = this.questions.map((question, qIndex) => 
        qIndex === actualQuestionIndex ? {
          ...question,
          answers: question.answers.map((answer, aIndex) => 
            aIndex === actualAnswerIndex ? { ...answer, isDeleted: true } : answer
          )
        } : question
      );
    }
    
    // Update positions for remaining answers
    this.questions = this.questions.map((question, qIndex) => 
      qIndex === actualQuestionIndex ? {
        ...question,
        answers: question.answers
          .filter(answer => !answer.isDeleted)
          .map((answer, index) => ({ ...answer, position: index }))
          .concat(question.answers.filter(answer => answer.isDeleted))
      } : question
    );
    
    // If we removed the correct answer, make the first remaining one correct
    const remainingAnswers = this.questions[actualQuestionIndex].answers.filter(answer => !answer.isDeleted);
    if (!remainingAnswers.some(answer => answer.is_correct)) {
      this.questions = this.questions.map((question, qIndex) => 
        qIndex === actualQuestionIndex ? {
          ...question,
          answers: question.answers.map((answer, aIndex) => 
            aIndex === question.answers.findIndex(a => !a.isDeleted) ? 
              { ...answer, is_correct: true } : answer
          )
        } : question
      );
    }
    
    this.checkForChanges();
  }

  updateTestName(name: string): void {
    if (name !== this.originalTestName) {
      this.hasUnsavedChanges = true;
    } else {
      this.checkForChanges();
    }
  }

  private updateQuestionPositions(): void {
    let position = 0;
    this.questions = this.questions.map(question => {
      if (!question.isDeleted) {
        return { ...question, position: position++ };
      }
      return question;
    });
  }

  private checkForChanges(): void {
    // Check if there are changes compared to original questions
    let hasChanges = false;

    // Check for new questions
    if (this.questions.some(q => q.isNew)) {
      hasChanges = true;
    }

    // Check for deleted questions
    if (this.questions.some(q => q.isDeleted)) {
      hasChanges = true;
    }

    // Check for modified existing questions
    for (const currentQuestion of this.questions) {
      if (!currentQuestion.isNew && !currentQuestion.isDeleted) {
        const originalQuestion = this.originalQuestions.find(q => q.id === currentQuestion.id);
        if (originalQuestion && this.hasQuestionChanged(originalQuestion, currentQuestion)) {
          hasChanges = true;
          break;
        }
      }
    }
    
    this.hasUnsavedChanges = hasChanges;
  }

  removeMaterial(materialToRemove: StudyMaterial, currentMaterials: StudyMaterial[]): StudyMaterial[] {
    return currentMaterials.filter(material => material.id !== materialToRemove.id);
  }

  updateMaterialQuestionCount(materialId: string, count: number, materials: StudyMaterial[]): StudyMaterial[] {
    return materials.map(material => 
      material.id === materialId ? { ...material, questionsToGenerate: count } : material
    );
  }

  getTotalQuestionsToGenerate(materials: StudyMaterial[]): number {
    return materials.reduce((sum, material) => sum + (material.questionsToGenerate || 5), 0);
  }

  async generateAdditionalQuestions(formData: TestEditFormData): Promise<void> {
    if (formData.selectedMaterials.length === 0) {
      throw new Error('No materials selected for generation');
    }

    if (!formData.concept.trim()) {
      throw new Error('No concept specified for generation');
    }

    this.isGenerating = true;
    this.error = '';
    this.generationStatus = 'Preparing study materials...';

    try {
      console.log('Generating additional questions for:', formData.selectedMaterials);
      
      const materialsRequest = formData.selectedMaterials.map(material => {
        if (!material.id) {
          console.warn('Missing material ID for material:', material);
          return null;
        }
        
        return {
          material_id: material.id,
          questions_to_generate: material.questionsToGenerate || 5
        };
      }).filter(req => req !== null);

      if (materialsRequest.length === 0) {
        throw new Error('No valid materials with IDs found');
      }

      this.generationStatus = `Generating test questions focused on "${formData.concept}"...`;
      
      const generatedQuestions = await invoke('generate_test_questions', {
        request: {
          materials: materialsRequest,
          language: formData.language || 'English',
          concept: formData.concept
        }
      }) as GeneratedQuestion[];

      console.log('Generated additional questions:', generatedQuestions);
      
      if (generatedQuestions && Array.isArray(generatedQuestions) && generatedQuestions.length > 0) {
        this.generationStatus = `Successfully generated ${generatedQuestions.length} additional test questions`;
        
        const currentVisibleQuestions = this.questions.filter(question => !question.isDeleted);
        const newQuestions: EditableTestQuestion[] = generatedQuestions.map((question: GeneratedQuestion, index: number) => ({
          id: null,
          question: question.question,
          answers: question.answers.map((answer, answerIndex) => ({
            id: null,
            answer_text: answer.answer_text,
            is_correct: answer.is_correct,
            position: answerIndex,
            isNew: true,
            isModified: false,
            isDeleted: false
          })),
          position: currentVisibleQuestions.length + index,
          isNew: true,
          isModified: false,
          isDeleted: false,
          tempId: `temp_${this.nextTempId++}`
        }));
        
        this.questions = [...this.questions, ...newQuestions];
        this.checkForChanges();
        
        setTimeout(() => {
          this.generationStatus = '';
          this.isGenerating = false;
        }, 2000);
      } else {
        throw new Error('No additional test questions were generated. Please try again or add questions manually.');
      }
    } catch (err) {
      console.error('Failed to generate additional test questions:', err);
      this.error = `Failed to generate additional test questions: ${err}`;
      this.isGenerating = false;
      throw err;
    }
  }

  async saveChanges(formData: TestEditFormData): Promise<void> {
    if (!this.isFormValid(formData)) {
      throw new Error('Form is not valid - check that all questions have content and at least 2 answers with one correct answer');
    }

    this.isSaving = true;
    this.error = '';

    try {
      const updateRequest = this.buildUpdateRequest(formData);
      
      console.log('Sending update request:', updateRequest);
      await invoke('update_test', {
        request: updateRequest
      });
      console.log('Successfully updated test');
      
      this.hasUnsavedChanges = false;
      
      // Update original questions to reflect the new state
      this.originalQuestions = JSON.parse(JSON.stringify(this.questions.filter(q => !q.isDeleted)));
      this.originalTestName = formData.name;

    } catch (err) {
      console.error('Failed to save changes:', err);
      this.error = `Failed to save changes: ${err}`;
      throw err;
    } finally {
      this.isSaving = false;
    }
  }

  setOriginalTestName(name: string): void {
    this.originalTestName = name;
  }

  reset(): void {
    this.questions = [];
    this.originalQuestions = [];
    this.isLoading = false;
    this.isSaving = false;
    this.error = '';
    this.hasUnsavedChanges = false;
    this.originalTestName = '';
    this.currentStep = 2;
  }
}