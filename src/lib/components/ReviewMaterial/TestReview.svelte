<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { fade, fly, scale, slide } from "svelte/transition";
  import { elasticOut, cubicOut } from "svelte/easing";
  import { invoke } from "@tauri-apps/api/core";

  // Icons
  import { X as XIcon } from "lucide-svelte";

  // Components
  import Button from "../Shared/Button.svelte";

  // Logic
  import {
    TestReviewLogic,
    type TestQuestionWithAnswers,
  } from "../../logic/ReviewMaterial/testReview";

  // Stores
  import { llmStore } from "../../stores/llmStore";

  // Props
  export let test = null;
  export let isOpen = false;

  // Callback Props
  export let onClose: () => void = () => {};
  export let onReviewCompleted: (data: {
    testId: string;
    statistics: any;
    reviewSession?: any;
    sessionId?: string;
    error?: string;
  }) => void = () => {};

  // State
  let reviewLogic = null;
  let currentQuestion = null;
  let isLoading = true;
  let testComplete = false;
  let error = "";
  let confetti = false;
  let selectedAnswers = [];
  let isExplaining = false;
  let explanation = "";
  let reviewSettings = null;
  let showSettings = false;
  let tempSettings = {
    shuffle_questions: true,
    shuffle_answers: true,
  };
  let testStats = {
    total_questions: 0,
  };
  let testTimer;
  let testStartTime = null;
  let testTime = 0;
  let sessionStartTime = null;
  let questionStartTime = null;
  let questionResponseTimes = [];
  let progress = { current: 0, total: 0, percentage: 0 };
  let session = null;
  let isAnimating = false;
  let questionDirection = 0; // -1 for left, 1 for right, 0 for none
  let keyboardEnabled = true;
  let showQuestionReview = false;
  let detailedResults = [];

  // Reactivity
  $: llmStatus = $llmStore;
  $: canUseAI = llmStatus.hasConfiguredProvider && !llmStatus.isLoading;
  $: testName = test && test.name ? test.name : "Test Review";
  $: if (test && isOpen && !reviewSettings) {
    showSettings = true;
    loadTestStats();
  }
  $: if (test && isOpen && reviewSettings) {
    loadQuestions();
  }

  // Functions
  async function loadTestStats() {
    if (!test?.id) return;

    try {
      const stats: [] = await invoke("get_test_questions", { testId: test.id });

      testStats = {
        total_questions: stats.length || 0,
      };
    } catch (error) {
      console.error("Failed to load test stats:", error);
      console.error("Error details:", error.message);
    }
  }

  async function loadQuestions() {
    isLoading = true;
    testComplete = false;
    error = "";

    try {
      if (test && test.id && reviewSettings) {
        const loadedQuestions: TestQuestionWithAnswers[] = await invoke(
          "get_test_questions",
          {
            testId: test.id,
          },
        );

        // Validate each question
        if (loadedQuestions && loadedQuestions.length > 0) {
          loadedQuestions.forEach((question, index) => {
            if (
              question.question.id == null ||
              question.question.id === undefined
            ) {
              console.error(
                `ERROR: Question ${index} has null/undefined ID:`,
                question,
              );
            }
          });

          reviewLogic = new TestReviewLogic(loadedQuestions, reviewSettings);
          updateDisplayState();
          startTestSession();
        } else {
          error = "No questions are available for this test.";
        }
      } else {
        throw new Error("Invalid test information or missing settings");
      }
    } catch (err) {
      console.error("Failed to load test questions:", err);
      console.error("Error stack:", err.stack);
      error = `Failed to load test questions: ${err.message || "Unknown error"}`;
    } finally {
      isLoading = false;
    }
  }

  function handleSettingsConfirm(settings) {
    reviewSettings = settings;
    showSettings = false;
  }

  function handleSettingsClose() {
    showSettings = false;
    if (!reviewSettings) {
      handleClose();
    }
  }

  function updateDisplayState() {
    if (reviewLogic) {
      const newQuestion = reviewLogic.getCurrentQuestion();
      const newProgress = reviewLogic.getProgress();
      const newSession = reviewLogic.getSession();

      currentQuestion = newQuestion;
      progress = newProgress;
      session = newSession;

      if (currentQuestion) {
        selectedAnswers = [];
      }

      if (currentQuestion && currentQuestion.question.id == null) {
        console.error(
          "ERROR: Current question has invalid ID:",
          currentQuestion,
        );
      }
    }
  }

  function startTestSession() {
    sessionStartTime = new Date();
    testStartTime = Date.now();
    testTime = 0;
    questionResponseTimes = [];
    startQuestionTimer();

    testTimer = setInterval(() => {
      testTime = Math.floor((Date.now() - testStartTime) / 1000);
      if (reviewLogic) {
        reviewLogic.updateSessionTime(testTime);
      }
    }, 1000);
  }

  function stopTestSession() {
    if (testTimer) {
      clearInterval(testTimer);
      testTimer = null;
    }
  }

  function startQuestionTimer() {
    questionStartTime = Date.now();
  }

  function recordQuestionResponseTime() {
    if (questionStartTime) {
      const responseTime = (Date.now() - questionStartTime) / 1000;
      questionResponseTimes.push(responseTime);
      startQuestionTimer();
    }
  }

  function handleAnswerSelect(answerId) {
    if (isAnimating || !currentQuestion) return;

    const answerIndex = currentQuestion.answers.findIndex(
      (a) => a.id === answerId,
    );
    if (answerIndex === -1) return;

    // For single-select questions (most common), replace selection
    selectedAnswers = [answerId];
  }

  async function handleExplain() {
    if (!currentQuestion || isExplaining) return;

    if (!canUseAI) {
      explanation =
        "No AI provider configured. Please configure an AI provider in Settings to use the explanation feature.";
      return;
    }

    isExplaining = true;
    explanation = "";

    try {
      const correctAnswer = currentQuestion.answers.find((a) => a.is_correct);
      const result: string = await invoke("explain_test_question", {
        question: currentQuestion.question.question,
        correctAnswer: correctAnswer?.answer_text || "",
        allAnswers: currentQuestion.answers.map((a) => a.answer_text),
      });
      explanation = result;
    } catch (err) {
      console.error("Failed to get explanation:", err);
      explanation =
        "Sorry, I could not generate an explanation at this time. Please try again later.";
    } finally {
      isExplaining = false;
    }
  }

  async function handleSubmitAnswer() {
    if (
      !reviewLogic ||
      selectedAnswers.length === 0 ||
      isAnimating ||
      !currentQuestion
    )
      return;

    recordQuestionResponseTime();

    keyboardEnabled = false;
    isAnimating = true;
    questionDirection = 1;

    try {
      const isCurrentlyLastQuestion = !reviewLogic.canMoveNext();
      const isCorrect = reviewLogic.submitAnswer(selectedAnswers);

      setTimeout(() => {
        if (isCurrentlyLastQuestion) {
          completeTest();
        } else if (reviewLogic.canMoveNext()) {
          reviewLogic.moveNext();
          updateDisplayState();
          resetQuestionState();
        } else {
          completeTest();
        }

        setTimeout(() => {
          keyboardEnabled = true;
          isAnimating = false;
          questionDirection = 0;
        }, 100);
      }, 300);
    } catch (err) {
      console.error("Failed to submit answer:", err);
      console.error("Error details:", err.message);
      console.error("Error stack:", err.stack);

      setTimeout(() => {
        const isCurrentlyLastQuestion = !reviewLogic.canMoveNext();

        if (isCurrentlyLastQuestion) {
          completeTest();
        } else if (reviewLogic.canMoveNext()) {
          reviewLogic.moveNext();
          updateDisplayState();
          resetQuestionState();
        } else {
          completeTest();
        }

        setTimeout(() => {
          keyboardEnabled = true;
          isAnimating = false;
          questionDirection = 0;
        }, 100);
      }, 300);
    }
  }

  function resetQuestionState() {
    selectedAnswers = [];
    explanation = "";
    isExplaining = false;
  }

  function handlePrevious() {
    if (reviewLogic && reviewLogic.canMovePrevious() && !isAnimating) {
      isAnimating = true;
      questionDirection = -1;

      setTimeout(() => {
        reviewLogic.movePrevious();
        updateDisplayState();
        resetQuestionState();

        setTimeout(() => {
          isAnimating = false;
          questionDirection = 0;
        }, 100);
      }, 150);
    }
  }

  function handleKeyboard(event) {
    const target = event.target;
    const isTypingInInput =
      target.tagName === "INPUT" ||
      target.tagName === "TEXTAREA" ||
      target.hasAttribute("contenteditable") ||
      target.isContentEditable;

    // If typing in input, don't handle test shortcuts
    if (isTypingInInput) {
      return;
    }

    if (!keyboardEnabled || isAnimating || !currentQuestion) return;

    const handledKeys = [
      "Enter",
      "1",
      "2",
      "3",
      "4",
      "5",
      "e",
      "E",
      "ArrowLeft",
    ];
    if (handledKeys.includes(event.key)) {
      event.preventDefault();
      event.stopPropagation();
      console.log("Prevented default for:", event.key);
    }

    switch (event.key) {
      case "Enter":
        if (selectedAnswers.length > 0) {
          handleSubmitAnswer();
        }
        break;
      case "1":
      case "2":
      case "3":
      case "4":
      case "5":
        const answerIndex = parseInt(event.key) - 1;
        if (currentQuestion.answers[answerIndex]) {
          handleAnswerSelect(currentQuestion.answers[answerIndex].id);
        }
        break;
      case "e":
      case "E":
        if (canUseAI) {
          handleExplain();
        }
        break;
      case "ArrowLeft":
        handlePrevious();
        break;
    }
  }

  async function completeTest() {
    stopTestSession();

    if (reviewLogic) {
      detailedResults = reviewLogic.getDetailedResults();
    }

    if (test && test.id && reviewLogic && sessionStartTime) {
      try {
        const sessionStats = reviewLogic.getSession();
        session = sessionStats;

        const sessionEndTime = new Date();
        const averageResponseTime =
          questionResponseTimes.length > 0
            ? questionResponseTimes.reduce((a, b) => a + b, 0) /
              questionResponseTimes.length
            : 0;

        // Create review session request with the new unified structure
        const reviewSessionRequest = {
          material_type: "test",
          material_id: test.id,
          session_start: sessionStartTime.toISOString(),
          session_end: sessionEndTime.toISOString(),
          total_duration_seconds: testTime,
          questions_answered: sessionStats.questionsAnswered,
          correct_answers: sessionStats.correctAnswers,
          incorrect_answers: sessionStats.incorrectAnswers,
          skipped_answers: sessionStats.skippedAnswers || 0,
          score_percentage: reviewLogic.getScorePercentage(),
          time_per_question_seconds: averageResponseTime,
          completed: true,
        };

        console.log("Saving review session:", reviewSessionRequest);

        const sessionId: string = await invoke("create_review_session", {
          request: reviewSessionRequest,
        });

        await invoke("update_last_review", {
          materialId: test.id,
        });

        onReviewCompleted({
          testId: test.id,
          statistics: sessionStats,
          reviewSession: reviewSessionRequest,
          sessionId: sessionId,
        });

        // Show confetti animation
        confetti = true;
        setTimeout(() => {
          confetti = false;
        }, 5000);

        testComplete = true;
      } catch (err) {
        console.error("Failed to save review session:", err);
        console.error("Error stack:", err.stack);

        const sessionStats = reviewLogic.getSession();
        session = sessionStats;
        testComplete = true;

        onReviewCompleted({
          testId: test.id,
          statistics: sessionStats,
          error: err.message,
        });
      }
    } else {
      console.log("Completing test without saving session - missing data:", {
        testExists: !!test,
        testId: test?.id,
        reviewLogicExists: !!reviewLogic,
        sessionStartExists: !!sessionStartTime,
      });

      if (reviewLogic) {
        const sessionStats = reviewLogic.getSession();
        session = sessionStats;
      }
      testComplete = true;

      onReviewCompleted({
        testId: test?.id,
        statistics: session,
      });
    }
  }

  function handleClose() {
    stopTestSession();
    reviewSettings = null;
    showSettings = false;
    onClose();
  }

  function restartTest() {
    if (reviewLogic) {
      reviewLogic.reset();
      updateDisplayState();
      resetQuestionState();
      testComplete = false;
      reviewSettings = null;
      showSettings = true;
      showQuestionReview = false;
      detailedResults = [];
      loadTestStats();
    }
  }

  function toggleQuestionReview() {
    showQuestionReview = !showQuestionReview;
  }

  function getAnswerLetter(index) {
    return String.fromCharCode(65 + index);
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeyboard, { capture: true });

    return () => {
      window.removeEventListener("keydown", handleKeyboard, { capture: true });
      stopTestSession();
    };
  });

  onDestroy(() => {
    stopTestSession();
  });
</script>

{#if isOpen}
  <div class="tr-review-overlay" transition:fade={{ duration: 200 }}>
    <div
      class="tr-review-container"
      transition:slide={{ duration: 300, easing: cubicOut }}
    >
      <!-- Settings Modal -->
      {#if showSettings}
        <div class="tr-settings-overlay">
          <div class="tr-settings-modal">
            <div class="tr-settings-header">
              <h3>Test Settings</h3>
              <button
                class="tr-close-button"
                aria-label="Close"
                on:click={handleSettingsClose}
              >
                <XIcon/>
              </button>
            </div>

            <div class="tr-settings-content">
              <p class="tr-test-name">Test: {testName}</p>

              <!-- Test Statistics -->
              <div class="tr-test-stats">
                <div class="tr-stat-box">
                  <span class="tr-stat-number">{testStats.total_questions}</span
                  >
                  <span class="tr-stat-label">Questions</span>
                </div>
              </div>

              <div class="tr-settings-options">
                <label class="tr-setting-item">
                  <input
                    type="checkbox"
                    bind:checked={tempSettings.shuffle_questions}
                  />
                  <span class="tr-setting-label">Shuffle questions</span>
                  <span class="tr-setting-description"
                    >Randomize question order during test</span
                  >
                </label>

                <label class="tr-setting-item">
                  <input
                    type="checkbox"
                    bind:checked={tempSettings.shuffle_answers}
                  />
                  <span class="tr-setting-label">Shuffle answer options</span>
                  <span class="tr-setting-description"
                    >Randomize answer order for each question</span
                  >
                </label>
              </div>

              <div class="tr-settings-actions">
                <button
                  class="tr-settings-cancel"
                  on:click={handleSettingsClose}
                >
                  Cancel
                </button>
                <button
                  class="tr-settings-confirm"
                  on:click={() => handleSettingsConfirm(tempSettings)}
                  disabled={testStats.total_questions === 0}
                >
                  Start Test
                </button>
              </div>
            </div>
          </div>
        </div>
      {:else}
        <!-- Header -->
        <div class="tr-review-header">
          <div class="tr-test-info">
            <h2>{testName}</h2>
            <div class="tr-test-meta">
              <span>{progress.total} questions</span>
              {#if !testComplete && session}
                <span>Time: {TestReviewLogic.formatTestTime(testTime)}</span>
              {/if}
            </div>
          </div>

          <button
            class="tr-close-button"
            aria-label="Close"
            on:click={handleClose}
          >
            <XIcon/>
          </button>
        </div>

        {#if !testComplete}
          <!-- Progress Bar -->
          <div class="tr-progress-bar-container">
            <div
              class="tr-progress-bar"
              style="width: {progress.percentage}%"
            ></div>
          </div>

          <!-- Progress Info -->
          <div class="tr-progress-info">
            <div class="tr-progress-text">
              <span>Question {progress.current} of {progress.total}</span>
            </div>

            {#if session}
              <div class="tr-session-stats">
                <span class="tr-stat-item tr-correct"
                  >{session.correctAnswers}</span
                >
                <span class="tr-stat-item tr-incorrect"
                  >{session.incorrectAnswers}</span
                >
              </div>
            {/if}
          </div>
        {/if}

        <!-- Main Content -->
        <div class="tr-review-content">
          {#if isLoading}
            <div class="tr-loading-state" in:fade>
              <div class="tr-loading-spinner"></div>
              <span>Loading test questions...</span>
            </div>
          {:else if error}
            <div class="tr-error-state" in:fade>
              <div class="tr-error-icon">⚠️</div>
              <h3>Error</h3>
              <p>{error}</p>
              <Button variant="primary" onClick={handleClose} text="Close" />
            </div>
          {:else if testComplete}
            <div class="tr-complete-state" in:fade>
              <div class="tr-complete-icon">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="64"
                  height="64"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                  <polyline points="22 4 12 14.01 9 11.01"></polyline>
                </svg>
              </div>
              <h3>Test Complete!</h3>

              {#if session}
                <div class="tr-completion-stats">
                  <div class="tr-main-stats">
                    <div class="tr-stat-card">
                      <span class="tr-stat-value"
                        >{session.questionsAnswered}</span
                      >
                      <span class="tr-stat-label">Questions Answered</span>
                    </div>
                    <div class="tr-stat-card">
                      <span class="tr-stat-value"
                        >{TestReviewLogic.formatTestTime(testTime)}</span
                      >
                      <span class="tr-stat-label">Test Time</span>
                    </div>
                    <div class="tr-stat-card">
                      <span class="tr-stat-value"
                        >{reviewLogic
                          ? reviewLogic.getScorePercentage()
                          : 0}%</span
                      >
                      <span class="tr-stat-label">Score</span>
                    </div>
                  </div>

                  <div class="tr-score-breakdown">
                    <div class="tr-score-stat tr-correct">
                      <span class="tr-score-count"
                        >{session.correctAnswers}</span
                      >
                      <span class="tr-score-label">Correct</span>
                    </div>
                    <div class="tr-score-stat tr-incorrect">
                      <span class="tr-score-count"
                        >{session.incorrectAnswers}</span
                      >
                      <span class="tr-score-label">Incorrect</span>
                    </div>
                  </div>
                </div>
              {/if}

              <div class="tr-complete-actions">
                <Button
                  variant="secondary"
                  onClick={handleClose}
                  text="Close"
                />
                <Button
                  variant="primary"
                  onClick={restartTest}
                  text="Take Again"
                />
              </div>

              <!-- Question Review Section -->
              {#if detailedResults.length > 0}
                <div class="tr-question-review-section">
                  <button
                    class="tr-review-toggle"
                    on:click={toggleQuestionReview}
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      width="20"
                      height="20"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      class:tr-rotated={showQuestionReview}
                    >
                      <polyline points="6 9 12 15 18 9"></polyline>
                    </svg>
                    <span
                      >{showQuestionReview ? "Hide" : "Show"} Question Review</span
                    >
                  </button>

                  {#if showQuestionReview}
                    <div
                      class="tr-question-review-list"
                      in:slide={{ duration: 300 }}
                    >
                      {#each detailedResults as result, index}
                        <div
                          class="tr-review-question"
                          class:tr-correct={result.isCorrect}
                          class:tr-incorrect={!result.isCorrect &&
                            !result.wasSkipped}
                          class:tr-skipped={result.wasSkipped}
                        >
                          <div class="tr-review-question-header">
                            <div class="tr-review-question-number">
                              Question {index + 1}
                            </div>
                            <div class="tr-review-question-status">
                              {#if result.wasSkipped}
                                <span class="tr-status-badge tr-status-skipped"
                                  >Skipped</span
                                >
                              {:else if result.isCorrect}
                                <span class="tr-status-badge tr-status-correct"
                                  >Correct</span
                                >
                              {:else}
                                <span
                                  class="tr-status-badge tr-status-incorrect"
                                  >Incorrect</span
                                >
                              {/if}
                            </div>
                          </div>

                          <div class="tr-review-question-text">
                            {result.question.question.question}
                          </div>

                          <div class="tr-review-answers">
                            {#each result.question.answers as answer, answerIndex}
                              <div
                                class="tr-review-answer"
                                class:tr-user-selected={result.userAnswerIds.includes(
                                  answer.id,
                                )}
                                class:tr-correct-answer={answer.is_correct}
                                class:tr-wrong-selection={result.userAnswerIds.includes(
                                  answer.id,
                                ) && !answer.is_correct}
                              >
                                <div class="tr-review-answer-marker">
                                  {getAnswerLetter(answerIndex)}
                                </div>
                                <div class="tr-review-answer-text">
                                  {answer.answer_text}
                                </div>
                                <div class="tr-review-answer-indicators">
                                  {#if answer.is_correct}
                                    <span
                                      class="tr-answer-indicator tr-correct-indicator"
                                      title="Correct answer"
                                    >
                                      <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        width="16"
                                        height="16"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                      >
                                        <polyline points="20 6 9 17 4 12"
                                        ></polyline>
                                      </svg>
                                    </span>
                                  {/if}
                                  {#if result.userAnswerIds.includes(answer.id)}
                                    <span
                                      class="tr-answer-indicator tr-user-indicator"
                                      title="Your answer"
                                    >
                                      <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        width="16"
                                        height="16"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                      >
                                        <path
                                          d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"
                                        ></path>
                                        <circle cx="12" cy="7" r="4"></circle>
                                      </svg>
                                    </span>
                                  {/if}
                                </div>
                              </div>
                            {/each}
                          </div>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/if}

              {#if confetti}
                <div class="tr-confetti-container">
                  {#each Array(50) as _, i}
                    <div
                      class="tr-confetti"
                      style="
                      --fall-delay: {Math.random() * 3}s; 
                      --fall-distance: {50 + Math.random() * 40}vh; 
                      --fall-speed: {2 + Math.random() * 3}s; 
                      --left-pos: {Math.random() * 100}%; 
                      --size: {4 + Math.random() * 8}px; 
                      --hue: {Math.random() * 360}deg;
                    "
                      in:scale={{
                        duration: 300,
                        delay: Math.random() * 500,
                        easing: elasticOut,
                      }}
                    ></div>
                  {/each}
                </div>
              {/if}
            </div>
          {:else if currentQuestion}
            <!-- Test Question Interface -->
            <div class="tr-test-interface">
              <!-- Main Question Display -->
              <div class="tr-question-section">
                <div
                  class="tr-question-card"
                  class:tr-animating={isAnimating}
                  in:fly={{
                    x: questionDirection * 300,
                    duration: isAnimating ? 300 : 0,
                    easing: cubicOut,
                  }}
                >
                  <div class="tr-question-content">
                    <div class="tr-question-text">
                      {currentQuestion.question.question}
                    </div>

                    <div class="tr-answers-section">
                      <div class="tr-answers-list">
                        {#each currentQuestion.answers as answer, index (answer.id)}
                          <button
                            class="tr-answer-option"
                            class:tr-selected={selectedAnswers.includes(
                              answer.id,
                            )}
                            on:click={() => handleAnswerSelect(answer.id)}
                            disabled={isAnimating}
                          >
                            <div class="tr-answer-marker">
                              {String.fromCharCode(65 + index)}
                            </div>
                            <div class="tr-answer-text">
                              {answer.answer_text}
                            </div>
                            <div class="tr-answer-number">
                              {index + 1}
                            </div>
                          </button>
                        {/each}
                      </div>
                    </div>
                  </div>

                  <div class="tr-question-footer">
                    <span class="tr-question-instructions"
                      >Select an answer</span
                    >
                    <span class="tr-keyboard-hint"
                      >1-{currentQuestion.answers.length} or Enter</span
                    >
                  </div>
                </div>
              </div>

              <!-- Explanation Section (when available) -->
              {#if explanation}
                <div class="tr-explanation-panel" in:fade={{ duration: 300 }}>
                  <div class="tr-explanation-header">
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      width="18"
                      height="18"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    >
                      <circle cx="12" cy="12" r="10"></circle>
                      <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
                      <path d="m12 17l.01 0"></path>
                    </svg>
                    <span>AI Explanation</span>
                  </div>
                  <div class="tr-explanation-content">
                    {explanation}
                  </div>
                </div>
              {/if}

              <!-- Control Section -->
              <div class="tr-control-section">
                <!-- Navigation Controls -->
                <div class="tr-navigation-controls">
                  <button
                    class="tr-nav-button tr-previous"
                    on:click|stopPropagation={handlePrevious}
                    disabled={!reviewLogic ||
                      !reviewLogic.canMovePrevious() ||
                      isAnimating}
                    title="Previous question (←)"
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      width="18"
                      height="18"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    >
                      <line x1="19" y1="12" x2="5" y2="12"></line>
                      <polyline points="12 19 5 12 12 5"></polyline>
                    </svg>
                    <span>Previous</span>
                  </button>

                  <!-- <div
                    class="tr-explain-button-wrapper"
                    title={!canUseAI
                      ? "Configure an AI provider in Settings to use explanations"
                      : ""}
                  >
                    <button
                      class="tr-nav-button tr-explain"
                      on:click|stopPropagation={handleExplain}
                      disabled={isExplaining || !canUseAI}
                      title="Get AI explanation (E)"
                    >
                      {#if isExplaining}
                        <div class="tr-mini-spinner"></div>
                      {:else}
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="18"
                          height="18"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                        >
                          <circle cx="12" cy="12" r="10"></circle>
                          <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
                          <line x1="12" y1="17" x2="12.01" y2="17"></line>
                        </svg>
                      {/if}
                      <span>{isExplaining ? "Explaining..." : "Explain"}</span>
                    </button>
                  </div> -->

                  <button
                    class="tr-nav-button tr-submit"
                    on:click|stopPropagation={handleSubmitAnswer}
                    disabled={selectedAnswers.length === 0 || isAnimating}
                    title="Submit answer (Enter)"
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      width="18"
                      height="18"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    >
                      <polyline points="20 6 9 17 4 12"></polyline>
                    </svg>
                    <span>Submit</span>
                  </button>
                </div>

                <!-- Keyboard Shortcuts Hint -->
                <div class="tr-keyboard-shortcuts">
                  <span class="tr-shortcut-hint">
                    1-{currentQuestion.answers.length} (select) • Enter (submit)
                    • {canUseAI ? "E (explain) • " : ""}← (previous)
                    {#if !canUseAI}
                      <br /><small
                        style="color: var(--text-secondary); font-style: italic;"
                        >Configure AI provider in Settings to enable
                        explanations</small
                      >
                    {/if}
                  </span>
                </div>
              </div>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .tr-review-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(4px);
    padding: var(--space-md);
  }

  .tr-review-container {
    background: var(--surface);
    width: 100%;
    max-width: 900px;
    height: 100%;
    max-height: 95vh;
    border-radius: var(--border-radius);
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 1px solid var(--border);
  }

  .tr-settings-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
  }

  .tr-settings-modal {
    background: var(--surface);
    border-radius: var(--border-radius);
    padding: 0;
    max-width: 500px;
    width: 90%;
    box-shadow:
      0 20px 25px -5px rgba(0, 0, 0, 0.1),
      0 10px 10px -5px rgba(0, 0, 0, 0.04);
    border: 1px solid var(--border);
  }

  .tr-settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md) var(--space-md) 0 var(--space-md);
    border-bottom: 1px solid var(--border);
    margin-bottom: var(--space-md);
  }

  .tr-settings-header h3 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text);
    font-family: var(--font-heading);
  }

  .tr-settings-content {
    padding: 0 var(--space-lg) var(--space-lg) var(--space-lg);
  }

  .tr-test-name {
    font-size: 1rem;
    font-weight: 500;
    color: var(--text-secondary);
    margin-bottom: var(--space-lg);
    text-align: center;
  }

  .tr-test-stats {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--space-sm);
    margin-bottom: var(--space-lg);
    justify-items: center;
  }

  .tr-stat-box {
    background: var(--background);
    border-radius: var(--border-radius);
    padding: var(--space-lg) var(--space-md);
    text-align: center;
    border: 2px solid var(--border);
    transition: all 0.2s ease;
    min-width: 120px;
  }

  .tr-stat-number {
    display: block;
    font-size: 2rem;
    font-weight: 700;
    color: var(--accent);
    margin-bottom: var(--space-xs);
  }

  .tr-stat-label {
    display: block;
    font-size: 0.875rem;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .tr-settings-options {
    margin-bottom: var(--space-lg);
  }

  .tr-setting-item {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
    margin-bottom: var(--space-md);
    cursor: pointer;
    padding: var(--space-md);
    border-radius: var(--border-radius);
    transition: background-color 0.2s ease;
  }

  .tr-setting-item:hover {
    background: color-mix(in srgb, var(--accent) 5%, var(--background));
  }

  .tr-setting-item input[type="checkbox"] {
    margin-right: var(--space-sm);
    width: 18px;
    height: 18px;
    accent-color: var(--accent);
  }

  .tr-setting-label {
    font-weight: 500;
    color: var(--text);
    display: flex;
    align-items: center;
    font-size: 0.95rem;
  }

  .tr-setting-description {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-left: 30px;
    line-height: 1.4;
  }

  .tr-settings-actions {
    display: flex;
    gap: var(--space-sm);
    justify-content: flex-end;
  }

  .tr-settings-cancel {
    padding: var(--space-sm) var(--space-md);
    border: 1px solid var(--border);
    background: var(--background);
    color: var(--text);
    border-radius: var(--border-radius);
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s ease;
    font-family: var(--font-body);
  }

  .tr-settings-cancel:hover {
    background: color-mix(in srgb, var(--text) 5%, var(--background));
    border-color: var(--text-secondary);
  }

  .tr-settings-confirm {
    padding: var(--space-sm) var(--space-md);
    border: none;
    background: var(--accent);
    color: white;
    border-radius: var(--border-radius);
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s ease;
    font-family: var(--font-body);
  }

  .tr-settings-confirm:hover:not(:disabled) {
    background: color-mix(in srgb, var(--accent) 85%, black);
  }

  .tr-settings-confirm:disabled {
    background: var(--text-secondary);
    cursor: not-allowed;
    opacity: 0.6;
  }

  .tr-review-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-lg) var(--space-xl);
    background: var(--background);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .tr-test-info h2 {
    margin: 0;
    font-family: var(--font-heading);
    font-weight: 600;
    font-size: 1.5rem;
    color: var(--text);
  }

  .tr-test-meta {
    color: var(--text-secondary);
    font-size: 0.9rem;
    display: flex;
    gap: var(--space-lg);
    margin-top: var(--space-xs);
    flex-wrap: wrap;
  }

  .tr-close-button {
    width: 58px;
    height: 58px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .tr-close-button:hover {
    color: var(--error);
    transform: scale(1.05);
  }

  /* ===== Progress ===== */
  .tr-progress-bar-container {
    height: 4px;
    background: var(--background);
    width: 100%;
    overflow: hidden;
    flex-shrink: 0;
  }

  .tr-progress-bar {
    height: 100%;
    background: linear-gradient(
      90deg,
      var(--accent),
      color-mix(in srgb, var(--accent) 80%, var(--success))
    );
    transition: width 0.4s ease;
    border-radius: 0 3px 3px 0;
  }

  .tr-progress-info {
    padding: var(--space-md) var(--space-xl);
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: var(--background);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .tr-progress-text {
    display: flex;
    gap: var(--space-lg);
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .tr-session-stats {
    display: flex;
    gap: var(--space-md);
  }

  .tr-stat-item {
    display: flex;
    align-items: center;
    font-weight: 500;
    font-size: 0.9rem;
    padding: 4px 8px;
    border-radius: 8px;
  }

  .tr-stat-item.tr-correct {
    color: var(--success);
    background: color-mix(in srgb, var(--success) 10%, transparent);
  }

  .tr-stat-item.tr-incorrect {
    color: var(--error);
    background: color-mix(in srgb, var(--error) 10%, transparent);
  }

  .tr-review-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
    height: 0;
  }

  .tr-loading-state,
  .tr-error-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-lg);
    color: var(--text-secondary);
    padding: var(--space-xl);
  }

  .tr-loading-spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--border);
    border-top: 3px solid var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .tr-error-icon {
    font-size: 3rem;
  }

  .tr-test-interface {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: var(--space-lg);
    gap: var(--space-lg);
    min-height: 0;
    overflow-y: auto;
  }

  .tr-question-section {
    flex: 0 0 auto;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .tr-question-card {
    width: 100%;
    max-width: 700px;
    background: var(--background);
    border-radius: var(--border-radius);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
    border: 1px solid var(--border);
    overflow: hidden;
    transition: transform 0.3s ease;
  }

  .tr-question-card.tr-animating {
    pointer-events: none;
  }

  .tr-question-content {
    padding: var(--space-lg);
  }

  .tr-question-text {
    font-size: 1.3rem;
    line-height: 1.6;
    color: var(--text);
    text-align: center;
    margin-bottom: var(--space-xl);
    word-wrap: break-word;
    word-break: break-word;
    hyphens: none;
    overflow-wrap: break-word;
  }

  .tr-answers-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .tr-answers-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .tr-answer-option {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md) var(--space-lg);
    background: var(--surface);
    border: 2px solid var(--border);
    border-radius: var(--border-radius);
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: left;
    width: 100%;
    position: relative;
  }

  .tr-answer-option:hover {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 5%, var(--surface));
    transform: translateY(-1px);
  }

  .tr-answer-option.tr-selected {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 12%, var(--surface));
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 20%, transparent);
  }

  .tr-answer-option:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    transform: none;
  }

  .tr-answer-marker {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: var(--background);
    border: 2px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
    color: var(--text-secondary);
    flex-shrink: 0;
    transition: all 0.2s ease;
  }

  .tr-answer-option.tr-selected .tr-answer-marker {
    background: var(--accent);
    border-color: var(--accent);
    color: white;
  }

  .tr-answer-text {
    flex: 1;
    font-size: 1rem;
    line-height: 1.5;
    color: var(--text);
  }

  .tr-answer-number {
    position: absolute;
    top: 8px;
    right: 12px;
    width: 20px;
    height: 20px;
    background: var(--background);
    border: 1px solid var(--border);
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    color: var(--text-secondary);
    font-weight: 600;
  }

  .tr-question-footer {
    padding: var(--space-md) var(--space-xl);
    background: color-mix(in srgb, var(--accent) 5%, var(--background));
    border-top: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
  }

  .tr-question-instructions {
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .tr-keyboard-hint {
    color: var(--text-secondary);
    font-size: 0.8rem;
    background: var(--surface);
    padding: 4px 8px;
    border-radius: 4px;
    border: 1px solid var(--border);
  }

  .tr-explanation-panel {
    flex: 0 0 auto;
    background: color-mix(in srgb, var(--accent) 5%, var(--background));
    border-radius: var(--border-radius);
    border: 1px solid color-mix(in srgb, var(--accent) 20%, var(--border));
    overflow: hidden;
  }

  .tr-explanation-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-md) var(--space-lg);
    background: color-mix(in srgb, var(--accent) 10%, var(--background));
    border-bottom: 1px solid
      color-mix(in srgb, var(--accent) 20%, var(--border));
    color: var(--accent);
    font-weight: 600;
    font-size: 0.95rem;
  }

  .tr-explanation-content {
    padding: var(--space-lg);
    color: var(--text);
    line-height: 1.6;
    font-size: 0.95rem;
    max-height: 200px;
    overflow-y: auto;
  }

  .tr-control-section {
    flex: 0 0 auto;
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .tr-navigation-controls {
    display: flex;
    gap: var(--space-md);
    align-items: center;
    justify-content: center;
    flex-wrap: wrap;
  }

  .tr-nav-button {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-xs);
    padding: var(--space-md) var(--space-lg);
    border: none;
    border-radius: var(--border-radius);
    background: var(--background);
    color: var(--text);
    cursor: pointer;
    font-family: var(--font-body);
    font-weight: 500;
    transition: all 0.2s ease;
    border: 1px solid var(--border);
    min-width: 120px;
    font-size: 0.9rem;
  }

  .tr-nav-button:hover:not(:disabled) {
    background: color-mix(in srgb, var(--accent) 10%, var(--background));
    transform: translateY(-1px);
  }

  .tr-nav-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }

  .tr-nav-button.tr-submit {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }

  .tr-nav-button.tr-submit:hover:not(:disabled) {
    background: color-mix(in srgb, var(--accent) 85%, black);
  }

  .tr-nav-button.tr-explain {
    background: color-mix(in srgb, var(--accent) 12%, var(--background));
    border-color: color-mix(in srgb, var(--accent) 30%, var(--border));
    color: var(--accent);
    font-weight: 600;
  }

  .tr-nav-button.tr-explain:hover:not(:disabled) {
    background: color-mix(in srgb, var(--accent) 20%, var(--background));
  }

  .tr-explain-button-wrapper {
    display: inline-block;
  }

  .tr-nav-button.tr-explain:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tr-nav-button.tr-explain:disabled:hover {
    transform: none;
    background: color-mix(in srgb, var(--accent) 12%, var(--background));
  }

  .tr-mini-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid var(--border);
    border-top: 2px solid var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .tr-keyboard-shortcuts {
    text-align: center;
    padding: var(--space-md);
  }

  .tr-shortcut-hint {
    color: var(--text-secondary);
    font-size: 0.8rem;
    font-style: italic;
  }

  .tr-complete-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    gap: var(--space-md);
    padding: var(--space-md);
    text-align: center;
    overflow-y: auto;
    min-height: 0;
    height: 100%;
    box-sizing: border-box;
  }

  .tr-complete-icon {
    color: var(--success);
    margin-bottom: var(--space-md);
    flex-shrink: 0;
    margin-top: var(--space-lg);
  }

  .tr-complete-state h3 {
    font-family: var(--font-heading);
    font-weight: 600;
    font-size: 2rem;
    margin: 0;
    color: var(--text);
    flex-shrink: 0;
  }

  .tr-completion-stats {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    width: 100%;
    max-width: 600px;
    flex-shrink: 0;
  }

  .tr-main-stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-lg);
    flex-shrink: 0;
  }

  .tr-stat-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--space-lg);
    background: var(--background);
    border-radius: var(--border-radius);
    border: 1px solid var(--border);
    min-height: 100px;
  }

  .tr-stat-value {
    font-size: 2.5rem;
    font-weight: 600;
    color: var(--accent);
    margin-bottom: var(--space-xs);
    line-height: 1.2;
  }

  .tr-stat-label {
    color: var(--text-secondary);
    font-size: 0.9rem;
    font-weight: 500;
  }

  .tr-score-breakdown {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-md);
    flex-shrink: 0;
  }

  .tr-score-stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--space-md);
    background: var(--background);
    border-radius: var(--border-radius);
    border: 1px solid var(--border);
    min-height: 80px;
  }

  .tr-score-count {
    font-size: 1.8rem;
    font-weight: 600;
    margin-bottom: var(--space-xs);
    line-height: 1.2;
  }

  .tr-score-stat.tr-correct .tr-score-count {
    color: var(--success);
  }
  .tr-score-stat.tr-incorrect .tr-score-count {
    color: var(--error);
  }

  .tr-complete-actions {
    display: flex;
    gap: var(--space-md);
    margin-top: var(--space-lg);
    flex-shrink: 0;
    padding-bottom: var(--space-lg);
  }

  .tr-question-review-section {
    width: 100%;
    max-width: 800px;
    margin-top: var(--space-md);
    flex-shrink: 0;
  }

  .tr-review-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    background: var(--background);
    border: 1px solid var(--border);
    border-radius: var(--border-radius);
    padding: var(--space-md) var(--space-lg);
    color: var(--text);
    cursor: pointer;
    font-family: var(--font-body);
    font-weight: 500;
    transition: all 0.2s ease;
    width: 100%;
    justify-content: center;
    margin-bottom: var(--space-lg);
  }

  .tr-review-toggle:hover {
    background: color-mix(in srgb, var(--accent) 5%, var(--background));
    border-color: var(--accent);
  }

  .tr-review-toggle svg {
    transition: transform 0.2s ease;
  }

  .tr-review-toggle svg.tr-rotated {
    transform: rotate(180deg);
  }

  .tr-question-review-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .tr-review-question {
    background: var(--background);
    border-radius: var(--border-radius);
    border: 1px solid var(--border);
    padding: var(--space-md);
    transition: all 0.2s ease;
  }

  .tr-review-question.tr-correct {
    border-left: 4px solid var(--success);
    background: color-mix(in srgb, var(--success) 3%, var(--background));
  }

  .tr-review-question.tr-incorrect {
    border-left: 4px solid var(--error);
    background: color-mix(in srgb, var(--error) 3%, var(--background));
  }

  .tr-review-question.tr-skipped {
    border-left: 4px solid var(--text-secondary);
    background: color-mix(in srgb, var(--text-secondary) 3%, var(--background));
  }

  .tr-review-question-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-md);
  }

  .tr-review-question-number {
    font-weight: 600;
    color: var(--text);
    font-size: 0.9rem;
  }

  .tr-review-question-status {
    display: flex;
    align-items: center;
  }

  .tr-status-badge {
    padding: 4px 8px;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .tr-status-badge.tr-status-correct {
    background: color-mix(in srgb, var(--success) 15%, transparent);
    color: var(--success);
  }

  .tr-status-badge.tr-status-incorrect {
    background: color-mix(in srgb, var(--error) 15%, transparent);
    color: var(--error);
  }

  .tr-status-badge.tr-status-skipped {
    background: color-mix(in srgb, var(--text-secondary) 15%, transparent);
    color: var(--text-secondary);
  }

  .tr-review-question-text {
    font-size: 1rem;
    line-height: 1.5;
    color: var(--text);
    margin-bottom: var(--space-lg);
    font-weight: 500;
  }

  .tr-review-answers {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .tr-review-answer {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--border-radius);
    border: 1px solid var(--border);
    transition: all 0.2s ease;
    position: relative;
  }

  .tr-review-answer.tr-correct-answer {
    background: color-mix(in srgb, var(--success) 8%, var(--surface));
    border-color: color-mix(in srgb, var(--success) 30%, var(--border));
  }

  .tr-review-answer.tr-user-selected {
    background: color-mix(in srgb, var(--accent) 8%, var(--surface));
    border-color: color-mix(in srgb, var(--accent) 30%, var(--border));
  }

  .tr-review-answer.tr-wrong-selection {
    background: color-mix(in srgb, var(--error) 8%, var(--surface));
    border-color: color-mix(in srgb, var(--error) 30%, var(--border));
  }

  .tr-review-answer-marker {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--surface);
    border: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
    color: var(--text-secondary);
    flex-shrink: 0;
    font-size: 0.875rem;
  }

  .tr-review-answer.tr-correct-answer .tr-review-answer-marker {
    background: var(--success);
    border-color: var(--success);
    color: white;
  }

  .tr-review-answer.tr-user-selected .tr-review-answer-marker {
    background: var(--accent);
    border-color: var(--accent);
    color: white;
  }

  .tr-review-answer.tr-wrong-selection .tr-review-answer-marker {
    background: var(--error);
    border-color: var(--error);
    color: white;
  }

  .tr-review-answer-text {
    flex: 1;
    font-size: 0.9rem;
    line-height: 1.4;
    color: var(--text);
  }

  .tr-review-answer-indicators {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }

  .tr-answer-indicator {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .tr-answer-indicator.tr-correct-indicator {
    background: var(--success);
    color: white;
  }

  .tr-answer-indicator.tr-user-indicator {
    background: var(--accent);
    color: white;
  }

  .tr-confetti-container {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    pointer-events: none;
    z-index: 9999;
    overflow: hidden;
  }

  .tr-confetti {
    position: absolute;
    top: -10px;
    left: var(--left-pos);
    width: var(--size);
    height: var(--size);
    background-color: hsl(var(--hue), 70%, 60%);
    opacity: 0.9;
    border-radius: 2px;
    animation: confetti-fall var(--fall-speed) var(--fall-delay) linear forwards;
    transition: opacity 2s ease-out;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @keyframes confetti-fall {
    0% {
      transform: translateY(-10px) rotate(0deg);
      opacity: 1;
    }
    100% {
      transform: translateY(var(--fall-distance)) rotate(720deg);
      opacity: 0;
    }
  }

  @media (max-width: 768px) {
    .tr-review-overlay {
      padding: var(--space-sm);
    }

    .tr-review-container {
      max-height: 100vh;
    }

    .tr-settings-modal {
      width: 95%;
      max-width: none;
    }

    .tr-test-stats {
      grid-template-columns: 1fr;
    }

    .tr-stat-box {
      padding: var(--space-md);
    }

    .tr-stat-number {
      font-size: 1.5rem;
    }

    .tr-review-header {
      padding: var(--space-md) var(--space-lg);
    }

    .tr-test-info h2 {
      font-size: 1.3rem;
    }

    .tr-progress-info {
      padding: var(--space-sm) var(--space-lg);
      flex-direction: column;
      gap: var(--space-sm);
      align-items: flex-start;
    }

    .tr-test-interface {
      padding: var(--space-md);
      gap: var(--space-md);
    }

    .tr-question-card {
      margin: 0;
    }

    .tr-question-content {
      padding: var(--space-lg);
    }

    .tr-question-text {
      font-size: 1.1rem;
      margin-bottom: var(--space-lg);
    }

    .tr-answer-option {
      padding: var(--space-sm) var(--space-md);
      gap: var(--space-sm);
    }

    .tr-answer-marker {
      width: 28px;
      height: 28px;
    }

    .tr-answer-text {
      font-size: 0.9rem;
    }

    .tr-navigation-controls {
      justify-content: center;
      gap: var(--space-sm);
    }

    .tr-nav-button {
      min-width: 100px;
      font-size: 0.85rem;
      padding: var(--space-sm) var(--space-md);
    }

    .tr-explanation-content {
      max-height: 150px;
    }

    .tr-main-stats {
      grid-template-columns: 1fr;
      gap: var(--space-md);
    }

    .tr-score-breakdown {
      grid-template-columns: 1fr;
    }

    .tr-complete-actions {
      flex-direction: column;
      width: 100%;
    }

    .tr-complete-state {
      padding: var(--space-lg) var(--space-md);
      justify-content: flex-start;
    }

    .tr-complete-icon {
      margin-top: var(--space-md);
    }

    .tr-complete-state h3 {
      font-size: 1.8rem;
    }

    .tr-stat-card {
      min-height: 80px;
      padding: var(--space-md);
    }

    .tr-stat-value {
      font-size: 2rem;
    }

    .tr-score-stat {
      min-height: 70px;
    }

    .tr-score-count {
      font-size: 1.5rem;
    }

    .tr-question-review-section {
      margin-top: var(--space-lg);
    }

    .tr-review-question {
      padding: var(--space-md);
    }

    .tr-review-question-header {
      flex-direction: column;
      gap: var(--space-sm);
      align-items: flex-start;
    }

    .tr-review-answer {
      padding: var(--space-sm);
      gap: var(--space-sm);
    }

    .tr-review-answer-marker {
      width: 24px;
      height: 24px;
      font-size: 0.8rem;
    }

    .tr-review-answer-text {
      font-size: 0.85rem;
    }

    .tr-answer-indicator {
      width: 18px;
      height: 18px;
    }
  }

  @media (max-width: 480px) {
    .tr-settings-modal {
      width: 98%;
      margin: var(--space-sm);
    }

    .tr-settings-actions {
      flex-direction: column;
      gap: var(--space-xs);
    }

    .tr-settings-cancel,
    .tr-settings-confirm {
      width: 100%;
    }

    .tr-question-content {
      padding: var(--space-md);
    }

    .tr-question-text {
      font-size: 1rem;
    }

    .tr-answer-option {
      padding: var(--space-xs) var(--space-sm);
      flex-direction: column;
      align-items: flex-start;
      gap: var(--space-xs);
      text-align: left;
    }

    .tr-answer-marker {
      width: 24px;
      height: 24px;
      align-self: flex-end;
    }

    .tr-answer-number {
      position: static;
      margin-left: auto;
    }

    .tr-session-stats {
      flex-wrap: wrap;
      justify-content: center;
      gap: var(--space-sm);
    }

    .tr-navigation-controls {
      flex-direction: column;
      gap: var(--space-sm);
    }

    .tr-nav-button {
      width: 100%;
      min-width: auto;
    }

    .tr-explanation-content {
      max-height: 120px;
    }

    .tr-complete-state {
      padding: var(--space-md) var(--space-sm);
      gap: var(--space-md);
    }

    .tr-complete-icon {
      margin-top: var(--space-sm);
      margin-bottom: var(--space-sm);
    }

    .tr-complete-state h3 {
      font-size: 1.5rem;
    }

    .tr-stat-value {
      font-size: 1.8rem;
    }

    .tr-score-count {
      font-size: 1.3rem;
    }

    .tr-complete-actions {
      padding-bottom: var(--space-xl);
    }

    .tr-review-question {
      padding: var(--space-sm);
    }

    .tr-review-question-text {
      font-size: 0.9rem;
    }

    .tr-review-answer {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--space-xs);
    }

    .tr-review-answer-marker {
      align-self: flex-end;
    }

    .tr-review-answer-indicators {
      align-self: flex-end;
    }
  }

  @media (max-width: 360px) {
    .tr-settings-header {
      padding: var(--space-md);
      margin-bottom: var(--space-md);
    }

    .tr-settings-content {
      padding: 0 var(--space-md) var(--space-md) var(--space-md);
    }

    .tr-test-interface {
      padding: var(--space-sm);
    }

    .tr-question-text {
      font-size: 0.95rem;
    }

    .tr-nav-button {
      padding: var(--space-sm) var(--space-md);
      font-size: 0.8rem;
    }

    .tr-complete-state {
      padding: var(--space-sm);
    }

    .tr-complete-state h3 {
      font-size: 1.3rem;
    }

    .tr-stat-card {
      min-height: 70px;
      padding: var(--space-sm);
    }

    .tr-stat-value {
      font-size: 1.6rem;
    }

    .tr-score-stat {
      min-height: 60px;
      padding: var(--space-sm);
    }

    .tr-score-count {
      font-size: 1.2rem;
    }

    .tr-review-toggle {
      padding: var(--space-sm) var(--space-md);
      font-size: 0.9rem;
    }

    .tr-review-question-text {
      font-size: 0.85rem;
    }

    .tr-review-answer-text {
      font-size: 0.8rem;
    }

    .tr-status-badge {
      font-size: 0.7rem;
      padding: 2px 6px;
    }
  }

  .tr-close-button:focus,
  .tr-nav-button:focus,
  .tr-answer-option:focus,
  .tr-settings-cancel:focus,
  .tr-settings-confirm:focus,
  .tr-review-toggle:focus {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .tr-setting-item:focus-within {
    background: color-mix(in srgb, var(--accent) 8%, var(--background));
    outline: 2px solid var(--accent);
    outline-offset: 2px;
    border-radius: var(--border-radius);
  }

  @media (prefers-contrast: high) {
    .tr-question-card,
    .tr-answer-option,
    .tr-nav-button,
    .tr-settings-modal,
    .tr-stat-box,
    .tr-review-question,
    .tr-review-answer {
      border-width: 2px;
    }

    .tr-stat-box {
      border-width: 3px;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .tr-question-card,
    .tr-confetti,
    .tr-loading-spinner,
    .tr-mini-spinner {
      transition: none;
      animation: none;
    }

    .tr-loading-spinner,
    .tr-mini-spinner {
      border-top-color: var(--accent);
    }

    .tr-nav-button:hover:not(:disabled),
    .tr-answer-option:hover,
    .tr-settings-cancel:hover,
    .tr-settings-confirm:hover:not(:disabled),
    .tr-review-toggle:hover {
      transform: none;
    }

    .tr-review-toggle svg {
      transition: none;
    }
  }

  @media print {
    .tr-review-overlay {
      background: white;
      position: static;
    }

    .tr-review-container {
      box-shadow: none;
      border: 1px solid black;
      max-height: none;
      height: auto;
    }

    .tr-settings-overlay,
    .tr-control-section,
    .tr-close-button,
    .tr-confetti-container {
      display: none;
    }

    .tr-question-card {
      transform: none !important;
    }

    .tr-question-review-section {
      page-break-before: always;
    }

    .tr-review-question {
      page-break-inside: avoid;
      margin-bottom: var(--space-lg);
    }
  }
</style>
