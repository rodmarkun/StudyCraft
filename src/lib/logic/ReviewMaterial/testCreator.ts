import { invoke } from '@tauri-apps/api/core';

export interface StudyMaterial {
  id: string;
  name?: string;
  display_name?: string;
  file_name?: string;
  questionsToGenerate?: number;
  study_material_original?: string;
  markdown_path?: string;
}

export interface Test {
  id: string;
  name: string;
  type: 'test';
  questions_count: number;
  last_review: string;
  tags: string[];
  progress: number;
}

export interface TestAnswer {
  answer_text: string;
  is_correct: boolean;
  position: number;
}

export interface TestQuestion {
  question: string;
  answers: TestAnswer[];
  position?: number;
}

export interface TestFormData {
  name: string;
  language: string;
  tags: string[];
  selectedMaterials: StudyMaterial[];
  concept: string;
}

export interface GeneratedQuestion {
  question: string;
  answers: Array<{
    answer_text: string;
    is_correct: boolean;
    position: number;
  }>;
}

export interface TestCreationResponse {
  review_material_id: string;
  review_material_name: string;
  questions_count: number;
}

export class TestCreatorLogic {
  private questions: TestQuestion[] = [];
  private currentStep: number = 1;
  private isCreating: boolean = false;
  private isGenerating: boolean = false;
  private error: string = '';
  private generationStatus: string = '';

  get getCurrentStep(): number {
    return this.currentStep;
  }

  get getQuestions(): TestQuestion[] {
    return this.questions;
  }

  get getIsCreating(): boolean {
    return this.isCreating;
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

  isFormValid(formData: TestFormData): boolean {
    return formData.name.trim() !== '' && this.questions.every(question => 
      question.question.trim() !== '' && 
      question.answers.length >= 2 &&
      question.answers.some(answer => answer.is_correct) &&
      question.answers.every(answer => answer.answer_text.trim() !== '')
    );
  }

  isStepComplete(step: number, formData: TestFormData): boolean {
    switch (step) {
      case 1:
        return formData.name.trim() !== '';
      case 2:
        return this.questions.length > 0 && this.questions.every(question => 
          question.question.trim() !== '' && 
          question.answers.length >= 2 &&
          question.answers.some(answer => answer.is_correct) &&
          question.answers.every(answer => answer.answer_text.trim() !== '')
        );
      default:
        return false;
    }
  }

  addQuestion(): void {
    const newQuestion: TestQuestion = {
      question: '',
      answers: [
        { answer_text: '', is_correct: true, position: 0 },
        { answer_text: '', is_correct: false, position: 1 }
      ],
      position: this.questions.length
    };
    this.questions = [...this.questions, newQuestion];
  }

  addQuestionWithContent(question: string, answers: TestAnswer[]): void {
    const newQuestion: TestQuestion = {
      question,
      answers: answers.map((answer, index) => ({
        ...answer,
        position: index
      })),
      position: this.questions.length
    };
    this.questions = [...this.questions, newQuestion];
  }

  removeQuestion(index: number): void {
    this.questions = this.questions.filter((_, i) => i !== index);
    // Update positions
    this.questions = this.questions.map((question, i) => ({
      ...question,
      position: i
    }));
  }

  updateQuestion(index: number, value: string): void {
    if (this.questions[index]) {
      this.questions[index].question = value;
    }
  }

  updateAnswer(questionIndex: number, answerIndex: number, field: 'answer_text' | 'is_correct', value: string | boolean): void {
    if (this.questions[questionIndex] && this.questions[questionIndex].answers[answerIndex]) {
      if (field === 'is_correct' && value === true) {
        // Set all others to false when marking one as correct
        this.questions[questionIndex].answers = this.questions[questionIndex].answers.map((answer, i) => ({
          ...answer,
          is_correct: i === answerIndex
        }));
      } else {
        this.questions[questionIndex].answers[answerIndex] = {
          ...this.questions[questionIndex].answers[answerIndex],
          [field]: value
        };
      }
    }
  }

  addAnswer(questionIndex: number): void {
    if (this.questions[questionIndex]) {
      const newAnswer: TestAnswer = {
        answer_text: '',
        is_correct: false,
        position: this.questions[questionIndex].answers.length
      };
      this.questions[questionIndex].answers.push(newAnswer);
    }
  }

  removeAnswer(questionIndex: number, answerIndex: number): void {
    if (this.questions[questionIndex] && this.questions[questionIndex].answers.length > 2) {
      this.questions[questionIndex].answers = this.questions[questionIndex].answers.filter((_, i) => i !== answerIndex);
      
      // Update positions
      this.questions[questionIndex].answers = this.questions[questionIndex].answers.map((answer, i) => ({
        ...answer,
        position: i
      }));
      
      // If we removed the correct answer, make the first one correct
      if (!this.questions[questionIndex].answers.some(answer => answer.is_correct)) {
        this.questions[questionIndex].answers[0].is_correct = true;
      }
    }
  }

  async moveToNextStep(formData: TestFormData): Promise<void> {
    if (this.currentStep < 3) {
      if (this.currentStep === 1 && formData.selectedMaterials.length > 0) {
        await this.generateTestQuestions(formData);
      } else {
        this.currentStep++;
      }
    }
  }

  moveToPreviousStep(): void {
    if (this.currentStep > 1) {
      this.currentStep--;
    }
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

  async generateTestQuestions(formData: TestFormData): Promise<void> {
    if (formData.selectedMaterials.length === 0) {
      this.currentStep++;
      return;
    }

    this.isGenerating = true;
    this.error = '';
    this.generationStatus = 'Preparing study materials...';

    try {
      console.log('Preparing to generate test questions for:', formData.selectedMaterials);
      
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

      console.log('Sending generation request:', materialsRequest);
      this.generationStatus = formData.concept 
        ? `Generating test questions focused on "${formData.concept}"...`
        : 'Sending request to AI model...';
      
      const generatedQuestions = await invoke('generate_test_questions', {
        request: {
          materials: materialsRequest,
          language: formData.language || 'English',
          concept: formData.concept || null 
        }
      }) as GeneratedQuestion[];

      console.log('Generated questions:', generatedQuestions);
      this.generationStatus = `Successfully generated ${generatedQuestions.length} test questions`;
      
      if (generatedQuestions && Array.isArray(generatedQuestions) && generatedQuestions.length > 0) {
        this.questions = generatedQuestions.map((question: GeneratedQuestion, index: number) => ({
          question: question.question,
          answers: question.answers.map((answer, answerIndex) => ({
            answer_text: answer.answer_text,
            is_correct: answer.is_correct,
            position: answerIndex
          })),
          position: index
        }));
        
        setTimeout(() => {
          this.currentStep = 2;
          this.isGenerating = false;
          console.log('Advanced to step 2, questions:', this.questions);
        }, 1000);
      } else {
        this.error = 'No test questions were generated. Please try again or create questions manually.';
        this.isGenerating = false;
        this.currentStep = 2;
      }
    } catch (err) {
      console.error('Failed to generate test questions:', err);
      this.error = `Failed to generate test questions: ${err}. Please check the material format and try again.`;
      this.isGenerating = false;
      this.currentStep = 2;
    }
  }

  async createTest(formData: TestFormData, deckId: string): Promise<Test> {
    if (!this.isFormValid(formData)) {
      throw new Error('Form is not valid');
    }

    this.isCreating = true;
    this.error = '';

    try {
      const questionData = this.questions.map((question, index) => ({
        question: question.question,
        position: index,
        answers: question.answers.map((answer, answerIndex) => ({
          answer_text: answer.answer_text,
          is_correct: answer.is_correct,
          position: answerIndex
        }))
      }));
      
      const result = await invoke('create_test', {
        id: deckId,
        name: formData.name,
        tags: formData.tags,
        questions: questionData,
        language: formData.language
      }) as TestCreationResponse;

      console.log('Created test:', result);
      
      const newTest: Test = {
        id: result.review_material_id,
        name: result.review_material_name,
        type: 'test',
        questions_count: this.questions.length,
        last_review: new Date().toISOString().split('T')[0],
        tags: formData.tags,
        progress: 0
      };

      return newTest;
    } catch (err) {
      console.error('Failed to create test:', err);
      this.error = `Failed to create test: ${err}`;
      throw err;
    } finally {
      this.isCreating = false;
    }
  }

  reset(): void {
    this.questions = [];
    this.currentStep = 1;
    this.isCreating = false;
    this.isGenerating = false;
    this.error = '';
    this.generationStatus = '';
  }
}