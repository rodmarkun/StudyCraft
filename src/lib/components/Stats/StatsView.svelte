<script lang="ts">
  import { onMount } from 'svelte';

  // Icons
  import { Clock, Target, TrendingUp, BookOpen, Flame, RotateCcw, AlertTriangle, CreditCard, FileText } from 'lucide-svelte';
  
  // Manager
  import { StatsManager } from '../../logic/Stats/statsManager';

  // Components
  import StudyTimeChart from './StudyTimeChart.svelte';

  // State
  let statsManager = new StatsManager();
  let isLoading = true;
  let error = null;
  let overviewStats = {
    total_sessions: 0,
    total_study_time_seconds: 0,
    flashcard_sessions: 0,
    test_sessions: 0,
    average_cards_per_session: null,
    average_flashcard_retention: null,
    average_test_score: null,
    materials_studied: 0,
    last_session: null
  };
  let recentSessions = [];
  let studyTimeData = [];
  let currentStreak = 0;
  let thisWeekStats = {
    sessionsCount: 0,
    studyTime: 0,
    flashcardSessions: 0,
    testSessions: 0
  };

  // Functions
  function formatDuration(seconds) {
    if (!seconds || seconds === 0) return '0s';
    
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
    if (!dateString) return 'Never';
    try {
      const date = new Date(dateString);
      return date.toLocaleDateString();
    } catch (e) {
      return 'Invalid Date';
    }
  }

  function formatPercentage(value) {
    if (!value || isNaN(value)) return '0%';
    
    // Check if the value is already a percentage (> 1) or a decimal (0-1)
    if (value > 1) {
      // Already a percentage, just round it
      return `${Math.round(value)}%`;
    } else {
      // It's a decimal, multiply by 100
      return `${Math.round(value * 100)}%`;
    }
  }

  function getItemsLabel(session) {
    if (session.material_type === 'flashcard_deck') {
      return 'Cards';
    } else if (session.material_type === 'test') {
      return 'Questions';
    }
    return 'Items';
  }

  function getItemsStudied(session) {
    return statsManager.getItemsStudied(session);
  }

  function getSessionAccuracy(session) {
    return Math.round(statsManager.getAccuracyRate(session));
  }

  function getAccuracyLabel(session) {
    if (session.material_type === 'test') {
      return 'Score';
    }
    return 'Accuracy';
  }

  async function loadAllStats() {
    try {
      isLoading = true;
      error = null;

      // Load all stats in parallel
      const [
        sessionStats,
        recentSessionsData,
        allSessions
      ] = await Promise.all([
        statsManager.getReviewSessionStats(),
        statsManager.getRecentReviewSessions(10),
        statsManager.getAllReviewSessions()
      ]);

      // Update overview stats with separated counts
      overviewStats = {
        total_sessions: sessionStats.total_sessions || 0,
        total_study_time_seconds: sessionStats.total_study_time_seconds || 0,
        flashcard_sessions: allSessions.filter(s => s.material_type === 'flashcard_deck').length,
        test_sessions: allSessions.filter(s => s.material_type === 'test').length,
        average_cards_per_session: sessionStats.average_cards_per_session ?? null,
        average_flashcard_retention: sessionStats.average_flashcard_retention ?? null,
        average_test_score: sessionStats.average_test_score ?? null,
        materials_studied: sessionStats.materials_studied || 0,
        last_session: sessionStats.last_session ?? null
        };

      // Update recent sessions - limit to 5
      recentSessions = recentSessionsData.slice(0, 5);

      if (allSessions.length > 0) {
        console.log('Processing', allSessions.length, 'sessions for charts');
        
        currentStreak = statsManager.calculateCurrentStreak(allSessions);
        console.log('Current streak:', currentStreak);
        
        // Calculate this week stats with separation
        const oneWeekAgo = new Date();
        oneWeekAgo.setDate(oneWeekAgo.getDate() - 7);
        
        const thisWeekSessions = allSessions.filter(session => 
          new Date(session.session_start) >= oneWeekAgo
        );
        
        thisWeekStats = {
          sessionsCount: thisWeekSessions.length,
          studyTime: thisWeekSessions.reduce((total, session) => total + session.total_duration_seconds, 0),
          flashcardSessions: thisWeekSessions.filter(s => s.material_type === 'flashcard_deck').length,
          testSessions: thisWeekSessions.filter(s => s.material_type === 'test').length
        };
        
        console.log('This week stats:', thisWeekStats);
        
        studyTimeData = statsManager.prepareStudyTimeChartData(allSessions);
        console.log('Study time data:', studyTimeData);
      } else {
        console.log('No sessions available for chart data');
        thisWeekStats = {
          sessionsCount: 0,
          studyTime: 0,
          flashcardSessions: 0,
          testSessions: 0
        };
      }

    } catch (err) {
      console.error('Failed to load stats:', err);
      error = err.message || 'Failed to load statistics';
    } finally {
      isLoading = false;
    }
  }

  onMount(() => {
    loadAllStats();
  });
</script>

<section class="sv-stats-section">
  {#if isLoading}
    <div class="sv-loading">
      <div class="sv-loading-spinner"></div>
      <h3>Loading Statistics...</h3>
    </div>
  {:else if error}
    <div class="sv-error">
      <div class="sv-error-icon">
        <AlertTriangle size={48} />
      </div>
      <h3>Error Loading Stats</h3>
      <p>{error}</p>
      <button class="sv-retry-button" on:click={loadAllStats}>
        Try Again
      </button>
    </div>
  {:else}
    <div class="sv-container">
      <!-- Header -->
      <div class="sv-header">
        <h2>Statistics</h2>
      </div>

      <!-- Main Stats Display -->
      <div class="sv-stats-overview">
        <!-- Primary Stats Row -->
        <div class="sv-primary-stats">
          <div class="sv-stat-item">
            <div class="sv-stat-value">{overviewStats.total_sessions}</div>
            <div class="sv-stat-label">Total Sessions</div>
          </div>
          <div class="sv-stat-item">
            <div class="sv-stat-value">{formatDuration(overviewStats.total_study_time_seconds)}</div>
            <div class="sv-stat-label">Study Time</div>
          </div>
          <div class="sv-stat-item">
            <div class="sv-stat-value">{overviewStats.flashcard_sessions}</div>
            <div class="sv-stat-label">Flashcard Sessions</div>
          </div>
          <div class="sv-stat-item">
            <div class="sv-stat-value">{overviewStats.test_sessions}</div>
            <div class="sv-stat-label">Test Sessions</div>
          </div>
        </div>
        
        <!-- Secondary Info Bar -->
        <div class="sv-info-bar">
          <div class="sv-streak-info">
            <Flame size={18} class="sv-streak-icon" />
            <span class="sv-streak-value">{currentStreak}</span>
            <span class="sv-streak-label">day streak</span>
          </div>
          
          <div class="sv-week-info">
            <span class="sv-week-label">This week:</span>
            <span class="sv-week-stat">{thisWeekStats.sessionsCount} sessions</span>
            <span class="sv-week-stat">{formatDuration(thisWeekStats.studyTime)}</span>
            <span class="sv-week-stat">{thisWeekStats.flashcardSessions} flashcard sessions</span>
            <span class="sv-week-stat">{thisWeekStats.testSessions} test sessions</span>
          </div>
        </div>
      </div>

      <!-- Study Time Chart -->
      <div class="sv-chart-section">
        <div class="sv-chart-header">
          <h3>Study Time Trend</h3>
          <p>Daily study time over the last 30 days</p>
        </div>
        <StudyTimeChart data={studyTimeData} />
      </div>

      <!-- Recent Sessions with corrected stats -->
      <div class="sv-sessions-section">
        <div class="sv-section-header">
          <h3>Recent Sessions</h3>
          <p>Your latest study sessions</p>
        </div>
        
        <div class="sv-sessions-grid">
          {#if recentSessions.length > 0}
            {#each recentSessions as session, index}
              <div class="sv-session-row" style="animation-delay: {index * 50}ms;">
                <div class="sv-session-info">
                  <div class="sv-session-title">
                    {session.material_name || 'Study Session'}
                  </div>
                  <div class="sv-session-meta">
                    <span class="sv-session-date">{formatDate(session.session_start)}</span>
                    <span class="sv-session-type">
                      {#if session.material_type === 'flashcard_deck'}
                        <CreditCard size={12} style="display: inline; margin-right: 4px;" />
                        Flashcards
                      {:else if session.material_type === 'test'}
                        <FileText size={12} style="display: inline; margin-right: 4px;" />
                        Test
                      {:else}
                        {session.material_type}
                      {/if}
                    </span>
                  </div>
                </div>
                
                <div class="sv-session-stats">
                  <div class="sv-session-metric">
                    <Clock size={14} class="sv-metric-icon" />
                    <span>{formatDuration(session.total_duration_seconds)}</span>
                  </div>
                  <div class="sv-session-metric">
                    <Target size={14} class="sv-metric-icon" />
                    <span>{getItemsStudied(session)}</span>
                    <span class="sv-metric-label">{getItemsLabel(session)}</span>
                  </div>
                  <div class="sv-session-metric">
                    <TrendingUp size={14} class="sv-metric-icon" />
                    <span class="sv-accuracy" style="color: {getSessionAccuracy(session) >= 90 ? '#10b981' : getSessionAccuracy(session) >= 70 ? '#3b82f6' : getSessionAccuracy(session) >= 50 ? '#f59e0b' : '#ef4444'}">
                      {getSessionAccuracy(session)}%
                    </span>
                    <span class="sv-metric-label">{getAccuracyLabel(session)}</span>
                  </div>
                </div>
              </div>
            {/each}
          {:else}
            <div class="sv-sessions-empty">
              <div class="sv-empty-icon">
                <BookOpen size={48} />
              </div>
              <p>No recent sessions found</p>
            </div>
          {/if}
        </div>
      </div>

      <!-- Refresh Action -->
      <div class="sv-actions">
        <button class="sv-refresh-button" on:click={loadAllStats}>
          <RotateCcw size={16} />
          Refresh Stats
        </button>
      </div>
    </div>
  {/if}
</section>

<style>
  .sv-stats-section {
    padding: 1.5rem 0;
    min-height: 60vh;
    width: 100%;
  }

  .sv-container {
    padding: 0 var(--space-lg);
    max-width: 1200px;
    margin: 0 auto;
  }

  .sv-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    text-align: center;
    min-height: 300px;
  }

  .sv-loading-spinner {
    width: 32px;
    height: 32px;
    border: 2px solid var(--border);
    border-top: 2px solid #10b981;
    border-radius: 50%;
    animation: sv-spin 1s linear infinite;
    margin-bottom: 1rem;
  }

  @keyframes sv-spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .sv-error {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    text-align: center;
    min-height: 300px;
    background: color-mix(in srgb, #ef4444 5%, var(--surface));
    border: 1px solid color-mix(in srgb, #ef4444 20%, var(--border));
    border-radius: 12px;
  }

  .sv-error-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
  }

  .sv-retry-button {
    background: #ef4444;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 8px;
    cursor: pointer;
    margin-top: 1rem;
    transition: all 0.2s ease;
  }

  .sv-retry-button:hover {
    background: #dc2626;
    transform: translateY(-1px);
  }

  .sv-header {
    text-align: center;
    margin-bottom: 2rem;
  }

  .sv-header h2 {
    font-size: 3.2rem;
    font-weight: 400;
    letter-spacing: 0.02em;
    color: var(--text);
    font-family: var(--font-heading);
    margin: 0;
  }

  .sv-stats-overview {
    margin-bottom: 2rem;
  }

  .sv-primary-stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1.5rem;
    margin-bottom: 1rem;
  }

  .sv-stat-item {
    text-align: center;
    padding: 1.25rem 1rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    transition: all 0.2s ease;
  }

  .sv-stat-item:hover {
    border-color: #10b981;
    transform: translateY(-2px);
  }

  .sv-stat-value {
    display: block;
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--text);
    line-height: 1;
    margin-bottom: 0.375rem;
  }

  .sv-stat-label {
    font-size: 0.8rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 500;
  }

  .sv-info-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  .sv-streak-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .sv-streak-value {
    font-size: 1.5rem;
    font-weight: 700;
    color: #10b981;
  }

  .sv-streak-label {
    font-size: 1.1rem;
    color: var(--text-secondary);
    font-weight: 500;
    margin-top: 4px;
  }

  .sv-week-info {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .sv-week-label {
    font-size: 0.875rem;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .sv-week-stat {
    font-size: 0.8rem;
    color: var(--text);
    font-weight: 500;
    padding: 0.25rem 0.5rem;
    background: color-mix(in srgb, var(--border) 20%, transparent);
    border-radius: 4px;
  }

  .sv-chart-section {
    margin-bottom: 2rem;
  }

  .sv-chart-header {
    margin-bottom: 1.5rem;
  }

  .sv-chart-header h3 {
    font-size: 1.5rem;
    font-weight: 500;
    color: var(--text);
    margin: 0 0 0.25rem 0;
  }

  .sv-chart-header p {
    color: var(--text-secondary);
    font-size: 0.875rem;
    margin: 0;
  }

  .sv-sessions-section {
    margin-bottom: 2rem;
  }

  .sv-section-header {
    margin-bottom: 1.5rem;
  }

  .sv-section-header h3 {
    font-size: 1.5rem;
    font-weight: 500;
    color: var(--text);
    margin: 0 0 0.25rem 0;
  }

  .sv-section-header p {
    color: var(--text-secondary);
    font-size: 0.875rem;
    margin: 0;
  }

  .sv-sessions-grid {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .sv-session-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    transition: all 0.2s ease;
    opacity: 0;
    animation: sv-fade-in 0.4s ease-out forwards;
  }

  @keyframes sv-fade-in {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .sv-session-row:hover {
    border-color: #10b981;
    transform: translateX(4px);
  }

  .sv-session-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .sv-session-title {
    font-weight: 500;
    color: var(--text);
    font-size: 0.875rem;
  }

  .sv-session-meta {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .sv-session-date {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .sv-session-type {
    font-size: 0.75rem;
    color: #10b981;
    background: color-mix(in srgb, #10b981 10%, var(--surface));
    padding: 2px 8px;
    border-radius: 12px;
    text-transform: capitalize;
  }

  .sv-session-stats {
    display: flex;
    align-items: center;
    gap: 1.5rem;
  }

  .sv-session-metric {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.875rem;
    color: var(--text);
    font-weight: 500;
  }

  .sv-metric-label {
    font-size: 0.7rem;
    color: var(--text-secondary);
    margin-left: 0.25rem;
  }

  .sv-accuracy {
    font-weight: 600;
  }

  .sv-sessions-empty {
    text-align: center;
    padding: 3rem;
    color: var(--text-secondary);
  }

  .sv-empty-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
    opacity: 0.6;
  }

  .sv-actions {
    display: flex;
    justify-content: center;
  }

  .sv-refresh-button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: #10b981;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 8px;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .sv-refresh-button:hover {
    background: #059669;
    transform: translateY(-1px);
  }

  @media (max-width: 1024px) {
    .sv-primary-stats {
      grid-template-columns: repeat(2, 1fr);
    }

    .sv-info-bar {
      flex-direction: column;
      gap: 1rem;
      align-items: flex-start;
    }

    .sv-week-info {
      flex-wrap: wrap;
    }
  }

  @media (max-width: 768px) {
    .sv-container {
      padding: 0 1rem;
    }

    .sv-header h2 {
      font-size: 2rem;
    }

    .sv-primary-stats {
      grid-template-columns: 1fr;
      gap: 1rem;
    }

    .sv-stat-item {
      padding: 1rem;
    }

    .sv-session-row {
      flex-direction: column;
      align-items: flex-start;
      gap: 1rem;
    }

    .sv-session-stats {
      gap: 1rem;
    }
  }

  @media (max-width: 480px) {
    .sv-session-stats {
      flex-direction: column;
      align-items: flex-start;
      gap: 0.5rem;
    }
    
    .sv-week-info {
      flex-direction: column;
      align-items: flex-start;
      gap: 0.5rem;
    }
  }
</style>