<script>
  // Icons
  import {
    Clock,
    Target,
    TrendingUp,
    Zap,
    Calendar,
    CheckCircle,
    Pause,
    HelpCircle,
    CreditCard,
    FileText,
  } from "lucide-svelte";

  // Props
  export let sessions = [];

  // Functions
  function formatDuration(seconds) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const remainingSeconds = seconds % 60;

    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    } else if (minutes > 0) {
      return `${minutes}m`;
    } else {
      return `${remainingSeconds}s`;
    }
  }

  function formatDate(dateString) {
    const date = new Date(dateString);
    const now = new Date();
    const diffTime = Math.abs(now.getTime() - date.getTime());
    const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));
    const diffHours = Math.floor(diffTime / (1000 * 60 * 60));
    const diffMinutes = Math.floor(diffTime / (1000 * 60));

    if (diffDays === 0) {
      if (diffHours === 0) {
        if (diffMinutes === 0) {
          return "Just now";
        } else {
          return `${diffMinutes} minute${diffMinutes > 1 ? "s" : ""} ago`;
        }
      } else {
        return `${diffHours} hour${diffHours > 1 ? "s" : ""} ago`;
      }
    } else if (diffDays === 1) {
      return "Yesterday";
    } else if (diffDays < 7) {
      return `${diffDays} day${diffDays > 1 ? "s" : ""} ago`;
    } else {
      return date.toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
        year: date.getFullYear() !== now.getFullYear() ? "numeric" : undefined,
      });
    }
  }

  function getAccuracyLevel(accuracy) {
    const normalizedAccuracy = accuracy >= 1 ? accuracy : accuracy * 100;

    if (normalizedAccuracy >= 90) return "excellent";
    if (normalizedAccuracy >= 70) return "good";
    if (normalizedAccuracy >= 50) return "fair";
    return "needs-improvement";
  }

  function getAccuracyColor(accuracy) {
    const normalizedAccuracy = accuracy >= 1 ? accuracy : accuracy * 100;

    if (normalizedAccuracy >= 90) return "#10b981"; // green
    if (normalizedAccuracy >= 70) return "#3b82f6"; // blue
    if (normalizedAccuracy >= 50) return "#f59e0b"; // yellow
    return "#ef4444"; // red
  }

  function getAccuracyValue(session) {
    if (session.material_type === "flashcard_deck") {
      // For flashcards, calculate from response counts if available
      const totalResponses =
        (session.again_count || 0) +
        (session.hard_count || 0) +
        (session.good_count || 0) +
        (session.easy_count || 0);
      if (totalResponses > 0) {
        const correctResponses =
          (session.good_count || 0) + (session.easy_count || 0);
        return Math.round((correctResponses / totalResponses) * 100);
      }
      // Fallback to retention_rate if available
      return session.retention_rate
        ? Math.round(session.retention_rate * 100)
        : 0;
    } else if (session.material_type === "test") {
      // For tests, use score_percentage
      return session.score_percentage
        ? Math.round(session.score_percentage)
        : 0;
    }
    // Generic fallback
    return session.accuracy_rate ? Math.round(session.accuracy_rate) : 0;
  }

  function formatTime(dateString) {
    const date = new Date(dateString);
    return date.toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      hour12: true,
    });
  }

  function getItemsLabel(session) {
    if (session.material_type === "flashcard_deck") {
      return "Cards";
    } else if (session.material_type === "test") {
      return "Questions";
    }
    return "Items";
  }

  function getItemsStudied(session) {
    if (session.material_type === "flashcard_deck") {
      return session.cards_studied || session.items_studied || 0;
    } else if (session.material_type === "test") {
      return session.questions_answered || session.items_studied || 0;
    }
    return session.items_studied || 0;
  }

  function getAccuracyLabel(session) {
    if (session.material_type === "test") {
      return "Score";
    }
    return "Accuracy";
  }

  function hasFlashcardDetails(session) {
    return (
      session.material_type === "flashcard_deck" &&
      (session.again_count ||
        session.hard_count ||
        session.good_count ||
        session.easy_count)
    );
  }

  function hasTestDetails(session) {
    return (
      session.material_type === "test" &&
      (session.correct_answers ||
        session.incorrect_answers ||
        session.skipped_answers)
    );
  }

  function hasCardTypeBreakdown(session) {
    return (
      session.material_type === "flashcard_deck" &&
      (session.new_cards_count ||
        session.learning_cards_count ||
        session.review_cards_count)
    );
  }

  function getAverageTime(session) {
    if (session.material_type === "flashcard_deck") {
      return session.average_response_time_seconds;
    } else if (session.material_type === "test") {
      return session.time_per_question_seconds;
    }
    return null;
  }

  function getAverageTimeLabel(session) {
    if (session.material_type === "flashcard_deck") {
      return "Avg Time";
    } else if (session.material_type === "test") {
      return "Per Question";
    }
    return "Avg Time";
  }
</script>

<div class="sv-sessions-list">
  {#if sessions.length > 0}
    {#each sessions as session, index}
      <div class="sv-session-item" style="animation-delay: {index * 100}ms;">
        <div class="sv-session-main">
          <div class="sv-session-header">
            <div class="sv-session-title-group">
              <h4 class="sv-session-title">
                {session.material_name || "Study Session"}
              </h4>
              <div class="sv-session-meta">
                <span class="sv-session-type">
                  {#if session.material_type === "flashcard_deck"}
                    <CreditCard
                      size={12}
                      style="display: inline; margin-right: 4px; margin-top: 4px"
                    />
                    Flashcards
                  {:else if session.material_type === "test"}
                    <FileText
                      size={12}
                      style="display: inline; margin-right: 4px;"
                    />
                    Test
                  {:else}
                    {session.material_type}
                  {/if}
                </span>
                <span class="sv-session-time"
                  >{formatTime(session.session_start)}</span
                >
              </div>
            </div>
            <span class="sv-session-date"
              >{formatDate(session.session_start)}</span
            >
          </div>

          <div class="sv-session-stats">
            <div class="sv-session-stat">
              <Clock size={16} class="sv-session-stat-icon" />
              <div class="sv-session-stat-content">
                <span class="sv-session-stat-value"
                  >{formatDuration(session.total_duration_seconds)}</span
                >
                <span class="sv-session-stat-label">Duration</span>
              </div>
            </div>

            <div class="sv-session-stat">
              <Target size={16} class="sv-session-stat-icon" />
              <div class="sv-session-stat-content">
                <span class="sv-session-stat-value"
                  >{getItemsStudied(session)}</span
                >
                <span class="sv-session-stat-label"
                  >{getItemsLabel(session)}</span
                >
              </div>
            </div>

            <div class="sv-session-stat">
              <TrendingUp size={16} class="sv-session-stat-icon" />
              <div class="sv-session-stat-content">
                <span
                  class="sv-session-stat-value sv-accuracy-{getAccuracyLevel(
                    getAccuracyValue(session),
                  )}"
                  style="color: {getAccuracyColor(getAccuracyValue(session))}"
                >
                  {getAccuracyValue(session)}%
                </span>
                <span class="sv-session-stat-label"
                  >{getAccuracyLabel(session)}</span
                >
              </div>
            </div>

            <!-- Show different additional metrics based on session type -->
            {#if getAverageTime(session) && getAverageTime(session) > 0}
              <div class="sv-session-stat">
                <Zap size={16} class="sv-session-stat-icon" />
                <div class="sv-session-stat-content">
                  <span class="sv-session-stat-value"
                    >{getAverageTime(session).toFixed(1)}s</span
                  >
                  <span class="sv-session-stat-label"
                    >{getAverageTimeLabel(session)}</span
                  >
                </div>
              </div>
            {/if}
          </div>
        </div>

        <div class="sv-session-progress">
          <div class="sv-session-progress-bar">
            <div
              class="sv-session-progress-fill"
              style="width: {getAccuracyValue(
                session,
              )}%; background-color: {getAccuracyColor(
                getAccuracyValue(session),
              )}"
            ></div>
          </div>
          <div class="sv-session-progress-label">
            {getAccuracyValue(session)}% {getAccuracyLabel(
              session,
            ).toLowerCase()}
          </div>
        </div>

        <!-- Flashcard-specific breakdown -->
        {#if hasFlashcardDetails(session)}
          <div class="sv-session-breakdown">
            <div class="sv-breakdown-title">Response Breakdown</div>
            <div class="sv-breakdown-stats">
              {#if session.again_count > 0}
                <div class="sv-breakdown-item sv-again">
                  <span class="sv-breakdown-count">{session.again_count}</span>
                  <span class="sv-breakdown-label">Again</span>
                </div>
              {/if}
              {#if session.hard_count > 0}
                <div class="sv-breakdown-item sv-hard">
                  <span class="sv-breakdown-count">{session.hard_count}</span>
                  <span class="sv-breakdown-label">Hard</span>
                </div>
              {/if}
              {#if session.good_count > 0}
                <div class="sv-breakdown-item sv-good">
                  <span class="sv-breakdown-count">{session.good_count}</span>
                  <span class="sv-breakdown-label">Good</span>
                </div>
              {/if}
              {#if session.easy_count > 0}
                <div class="sv-breakdown-item sv-easy">
                  <span class="sv-breakdown-count">{session.easy_count}</span>
                  <span class="sv-breakdown-label">Easy</span>
                </div>
              {/if}
            </div>

            <!-- Card type breakdown if available -->
            {#if hasCardTypeBreakdown(session)}
              <div class="sv-card-types">
                <div class="sv-card-types-title">Card Types</div>
                <div class="sv-card-types-stats">
                  {#if session.new_cards_count > 0}
                    <div class="sv-card-type-item sv-new">
                      <span class="sv-card-type-count"
                        >{session.new_cards_count}</span
                      >
                      <span class="sv-card-type-label">New</span>
                    </div>
                  {/if}
                  {#if session.learning_cards_count > 0}
                    <div class="sv-card-type-item sv-learning">
                      <span class="sv-card-type-count"
                        >{session.learning_cards_count}</span
                      >
                      <span class="sv-card-type-label">Learning</span>
                    </div>
                  {/if}
                  {#if session.review_cards_count > 0}
                    <div class="sv-card-type-item sv-review">
                      <span class="sv-card-type-count"
                        >{session.review_cards_count}</span
                      >
                      <span class="sv-card-type-label">Review</span>
                    </div>
                  {/if}
                </div>
              </div>
            {/if}
          </div>
        {/if}

        <!-- Test-specific breakdown -->
        {#if hasTestDetails(session)}
          <div class="sv-session-breakdown">
            <div class="sv-breakdown-title">Answer Breakdown</div>
            <div class="sv-breakdown-stats">
              {#if session.correct_answers > 0}
                <div class="sv-breakdown-item sv-correct">
                  <span class="sv-breakdown-count"
                    >{session.correct_answers}</span
                  >
                  <span class="sv-breakdown-label">Correct</span>
                </div>
              {/if}
              {#if session.incorrect_answers > 0}
                <div class="sv-breakdown-item sv-incorrect">
                  <span class="sv-breakdown-count"
                    >{session.incorrect_answers}</span
                  >
                  <span class="sv-breakdown-label">Incorrect</span>
                </div>
              {/if}
              {#if session.skipped_answers > 0}
                <div class="sv-breakdown-item sv-skipped">
                  <span class="sv-breakdown-count"
                    >{session.skipped_answers}</span
                  >
                  <span class="sv-breakdown-label">Skipped</span>
                </div>
              {/if}
            </div>
          </div>
        {/if}

        <!-- Session status indicator -->
        <div class="sv-session-footer">
          <div class="sv-session-status">
            {#if session.completed}
              <span class="sv-status-badge sv-completed">
                <CheckCircle size={14} class="sv-status-icon" />
                Completed
              </span>
            {:else}
              <span class="sv-status-badge sv-incomplete">
                <Pause size={14} class="sv-status-icon" />
                Incomplete
              </span>
            {/if}
          </div>

          <div class="sv-session-duration-detail">
            Started at {formatTime(session.session_start)} • Ended at {formatTime(
              session.session_end,
            )}
          </div>
        </div>
      </div>
    {/each}
  {:else}
    <div class="sv-sessions-empty">
      <div class="sv-sessions-empty-icon">
        <Calendar size={48} />
      </div>
      <h4>No Recent Sessions</h4>
      <p>Start studying to see your session history here!</p>
      <div class="sv-empty-suggestion">
        Try creating some flashcard decks or tests and reviewing them to
        generate session data.
      </div>
    </div>
  {/if}
</div>

<style>
  .sv-sessions-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .sv-session-item {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--border-radius);
    padding: var(--space-md);
    transition: all var(--transition-speed) ease;
    opacity: 0;
    animation: sv-session-slide-in 0.5s ease-out forwards;
    position: relative;
    overflow: hidden;
  }

  @keyframes sv-session-slide-in {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .sv-session-item:hover {
    transform: translateX(4px);
    border-color: color-mix(in srgb, var(--accent) 30%, var(--border));
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
  }

  .sv-session-item::before {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    background: linear-gradient(
      90deg,
      var(--accent),
      var(--accent-light, var(--accent))
    );
    opacity: 0;
    transition: opacity var(--transition-speed) ease;
  }

  .sv-session-item:hover::before {
    opacity: 1;
  }

  .sv-session-main {
    margin-bottom: var(--space-md);
  }

  .sv-session-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: var(--space-md);
  }

  .sv-session-title-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
    flex: 1;
  }

  .sv-session-title {
    margin: 0;
    font-size: 1rem;
    color: var(--text);
    font-family: var(--font-heading);
    font-weight: 600;
    line-height: 1.3;
  }

  .sv-session-meta {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex-wrap: wrap;
  }

  .sv-session-type {
    font-size: 0.75rem;
    color: var(--text-secondary);
    background: color-mix(in srgb, var(--accent) 10%, var(--surface));
    padding: 4px 10px;
    border-radius: 12px;
    text-transform: capitalize;
    font-weight: 500;
    border: 1px solid color-mix(in srgb, var(--accent) 20%, var(--border));
  }

  .sv-session-type {
    background: color-mix(in srgb, #3b82f6 10%, var(--surface));
    border: 1px solid color-mix(in srgb, #3b82f6 20%, var(--border));
    color: #3b82f6;
  }

  .sv-session-time {
    font-size: 0.8rem;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .sv-session-date {
    font-size: 0.9rem;
    color: var(--text-secondary);
    flex-shrink: 0;
    font-weight: 500;
    text-align: right;
  }

  .sv-session-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
    gap: var(--space-sm);
    align-items: center;
  }

  .sv-session-stat {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-sm);
    background: color-mix(in srgb, var(--accent) 3%, var(--surface));
    border-radius: var(--border-radius);
    border: 1px solid color-mix(in srgb, var(--accent) 10%, var(--border));
    transition: all var(--transition-speed) ease;
  }

  .sv-session-stat:hover {
    background: color-mix(in srgb, var(--accent) 8%, var(--surface));
    border-color: color-mix(in srgb, var(--accent) 20%, var(--border));
  }

  .sv-session-stat-icon {
    font-size: 1.1rem;
    flex-shrink: 0;
  }

  .sv-session-stat-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .sv-session-stat-value {
    font-size: 0.9rem;
    font-weight: 700;
    color: var(--text);
    line-height: 1;
  }

  .sv-session-stat-label {
    font-size: 0.65rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 500;
  }

  .sv-session-progress {
    margin-bottom: var(--space-lg);
  }

  .sv-session-progress-bar {
    height: 8px;
    background: var(--border);
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: var(--space-xs);
  }

  .sv-session-progress-fill {
    height: 100%;
    border-radius: 4px;
    transition: width 1.5s ease-out;
    background: linear-gradient(
      90deg,
      currentColor,
      color-mix(in srgb, currentColor 80%, white)
    );
    position: relative;
  }

  .sv-session-progress-fill::after {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(
      90deg,
      transparent,
      rgba(255, 255, 255, 0.3),
      transparent
    );
    animation: sv-progress-shine 2s ease-out;
  }

  @keyframes sv-progress-shine {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(100%);
    }
  }

  .sv-session-progress-label {
    font-size: 0.8rem;
    color: var(--text-secondary);
    text-align: center;
    font-weight: 500;
  }

  .sv-session-breakdown {
    border-top: 1px solid var(--border);
    padding-top: var(--space-lg);
    margin-bottom: var(--space-lg);
  }

  .sv-breakdown-title {
    font-size: 0.9rem;
    color: var(--text);
    margin-bottom: var(--space-md);
    font-weight: 600;
    font-family: var(--font-heading);
  }

  .sv-breakdown-stats {
    display: flex;
    gap: var(--space-md);
    flex-wrap: wrap;
    margin-bottom: var(--space-lg);
  }

  .sv-breakdown-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-md);
    border-radius: var(--border-radius);
    min-width: 70px;
    transition: transform var(--transition-speed) ease;
  }

  .sv-breakdown-item:hover {
    transform: translateY(-2px);
  }

  .sv-breakdown-item.sv-again {
    background: color-mix(in srgb, #ef4444 8%, var(--surface));
    border: 1px solid color-mix(in srgb, #ef4444 20%, var(--border));
  }

  .sv-breakdown-item.sv-hard {
    background: color-mix(in srgb, #f59e0b 8%, var(--surface));
    border: 1px solid color-mix(in srgb, #f59e0b 20%, var(--border));
  }

  .sv-breakdown-item.sv-good {
    background: color-mix(in srgb, #3b82f6 8%, var(--surface));
    border: 1px solid color-mix(in srgb, #3b82f6 20%, var(--border));
  }

  .sv-breakdown-item.sv-easy {
    background: color-mix(in srgb, #10b981 8%, var(--surface));
    border: 1px solid color-mix(in srgb, #10b981 20%, var(--border));
  }

  .sv-breakdown-item.sv-correct {
    background: color-mix(in srgb, #10b981 8%, var(--surface));
    border: 1px solid color-mix(in srgb, #10b981 20%, var(--border));
  }

  .sv-breakdown-item.sv-incorrect {
    background: color-mix(in srgb, #ef4444 8%, var(--surface));
    border: 1px solid color-mix(in srgb, #ef4444 20%, var(--border));
  }

  .sv-breakdown-item.sv-skipped {
    background: color-mix(in srgb, #6b7280 8%, var(--surface));
    border: 1px solid color-mix(in srgb, #6b7280 20%, var(--border));
  }

  .sv-breakdown-count {
    font-weight: 700;
    font-size: 1.2rem;
    color: var(--text);
  }

  .sv-breakdown-label {
    font-size: 0.7rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 500;
  }

  /* Card Types */
  .sv-card-types {
    border-top: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
    padding-top: var(--space-md);
  }

  .sv-card-types-title {
    font-size: 0.85rem;
    color: var(--text);
    margin-bottom: var(--space-sm);
    font-weight: 600;
  }

  .sv-card-types-stats {
    display: flex;
    gap: var(--space-sm);
    flex-wrap: wrap;
  }

  .sv-card-type-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-sm);
    border-radius: var(--border-radius);
    min-width: 60px;
    font-size: 0.8rem;
  }

  .sv-card-type-item.sv-new {
    background: color-mix(in srgb, #8b5cf6 8%, var(--surface));
    border: 1px solid color-mix(in srgb, #8b5cf6 20%, var(--border));
  }

  .sv-card-type-item.sv-learning {
    background: color-mix(in srgb, #06b6d4 8%, var(--surface));
    border: 1px solid color-mix(in srgb, #06b6d4 20%, var(--border));
  }

  .sv-card-type-item.sv-review {
    background: color-mix(in srgb, #10b981 8%, var(--surface));
    border: 1px solid color-mix(in srgb, #10b981 20%, var(--border));
  }

  .sv-card-type-count {
    font-weight: 700;
    color: var(--text);
  }

  .sv-card-type-label {
    font-size: 0.65rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 500;
  }

  .sv-session-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-top: 1px solid var(--border);
    padding-top: var(--space-md);
  }

  .sv-status-badge {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-sm);
    border-radius: 20px;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .sv-status-badge.sv-completed {
    background: color-mix(in srgb, #10b981 10%, var(--surface));
    color: #10b981;
    border: 1px solid color-mix(in srgb, #10b981 20%, var(--border));
  }

  .sv-status-badge.sv-incomplete {
    background: color-mix(in srgb, #f59e0b 10%, var(--surface));
    color: #f59e0b;
    border: 1px solid color-mix(in srgb, #f59e0b 20%, var(--border));
  }

  .sv-status-icon {
    font-size: 0.8rem;
  }

  .sv-session-duration-detail {
    font-size: 0.75rem;
    color: var(--text-secondary);
    font-weight: 400;
  }

  /* Empty State */
  .sv-sessions-empty {
    text-align: center;
    padding: var(--space-xl);
    color: var(--text-secondary);
    background: var(--surface);
    border: 2px dashed var(--border);
    border-radius: var(--border-radius);
  }

  .sv-sessions-empty-icon {
    font-size: 4rem;
    margin-bottom: var(--space-lg);
    opacity: 0.6;
  }

  .sv-sessions-empty h4 {
    margin: 0 0 var(--space-md) 0;
    color: var(--text);
    font-family: var(--font-heading);
    font-size: 1.3rem;
  }

  .sv-sessions-empty p {
    margin: 0 0 var(--space-md) 0;
    font-size: 1rem;
  }

  .sv-empty-suggestion {
    font-size: 0.9rem;
    color: var(--text-secondary);
    font-style: italic;
    max-width: 400px;
    margin: 0 auto;
    line-height: 1.5;
  }

  .sv-breakdown-item.sv-correct {
    background: color-mix(in srgb, #10b981 8%, var(--surface));
    border: 1px solid color-mix(in srgb, #10b981 20%, var(--border));
  }

  .sv-breakdown-item.sv-incorrect {
    background: color-mix(in srgb, #ef4444 8%, var(--surface));
    border: 1px solid color-mix(in srgb, #ef4444 20%, var(--border));
  }

  .sv-breakdown-item.sv-skipped {
    background: color-mix(in srgb, #6b7280 8%, var(--surface));
    border: 1px solid color-mix(in srgb, #6b7280 20%, var(--border));
  }

  @media (max-width: 768px) {
    .sv-session-header {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--space-md);
    }

    .sv-session-date {
      text-align: left;
    }

    .sv-session-stats {
      grid-template-columns: repeat(2, 1fr);
      gap: var(--space-sm);
    }

    .sv-breakdown-stats {
      justify-content: center;
    }

    .sv-session-footer {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--space-sm);
    }
  }

  @media (max-width: 480px) {
    .sv-sessions-list {
      gap: var(--space-md);
    }

    .sv-session-item {
      padding: var(--space-md);
    }

    .sv-session-stats {
      grid-template-columns: 1fr;
    }

    .sv-breakdown-stats {
      flex-direction: column;
      align-items: center;
      gap: var(--space-sm);
    }

    .sv-breakdown-item {
      width: 100%;
      max-width: 200px;
    }

    .sv-card-types-stats {
      justify-content: center;
    }
  }
</style>
