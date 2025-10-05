import { invoke } from '@tauri-apps/api/core';

// Enhanced review session that includes all data from the backend
export interface ReviewSession {
  id: string;
  material_id: string;
  material_type: string;
  session_start: string;
  session_end: string;
  total_duration_seconds: number;
  completed: boolean;
  created_at: string;
  material_name?: string;
  
  // Flashcard-specific fields (null for test sessions)
  cards_studied?: number;
  again_count?: number;
  hard_count?: number;
  good_count?: number;
  easy_count?: number;
  new_cards_count?: number;
  learning_cards_count?: number;
  review_cards_count?: number;
  retention_rate?: number;
  average_response_time_seconds?: number;
  
  // Test-specific fields (null for flashcard sessions)
  questions_answered?: number;
  correct_answers?: number;
  incorrect_answers?: number;
  skipped_answers?: number;
  score_percentage?: number;
  time_per_question_seconds?: number;
}

// Flashcard-specific review session data
export interface FlashcardReviewSession {
  session: ReviewSession;
  cards_studied: number;
  again_count: number;
  hard_count: number;
  good_count: number;
  easy_count: number;
  new_cards_count: number;
  learning_cards_count: number;
  review_cards_count: number;
  retention_rate: number;
  average_response_time_seconds: number;
}

// Test-specific review session data
export interface TestReviewSession {
  session: ReviewSession;
  questions_answered: number;
  correct_answers: number;
  incorrect_answers: number;
  skipped_answers: number;
  score_percentage: number;
  time_per_question_seconds: number;
  questions_order?: string;
  answers_given?: string;
}

// Enhanced stats interface with separated counts
export interface ReviewSessionStats {
  total_sessions: number;
  total_study_time_seconds: number;
  average_cards_per_session?: number;
  average_flashcard_retention?: number;
  average_test_score?: number;
  last_session?: string;
  materials_studied: number;
  flashcard_sessions: number;
  test_sessions: number;
}

export interface StudyTimeDataPoint {
  date: string;
  dateLabel: string;
  studyTime: number;
}

export interface AccuracyDataPoint {
  date: string;
  dateLabel: string;
  accuracy: number;
}

export class StatsManager {
  
  async getReviewSessionStats(): Promise<ReviewSessionStats> {
    try {
      return await invoke('get_review_session_stats');
    } catch (error) {
      console.error('Failed to get review session stats:', error);
      throw new Error(`Failed to load statistics: ${error}`);
    }
  }

  async getRecentReviewSessions(limit: number = 10): Promise<ReviewSession[]> {
    try {
      if (limit <= 0 || limit > 100) {
        throw new Error('Limit must be between 1 and 100');
      }
      
      // The enhanced backend query now returns all necessary data in one call
      const sessions: ReviewSession[] = await invoke('get_recent_review_sessions', { limit });
      return sessions;
    } catch (error) {
      console.error('Failed to get recent review sessions:', error);
      throw new Error(`Failed to load recent sessions: ${error}`);
    }
  }

  async getAllReviewSessions(): Promise<ReviewSession[]> {
    try {
      const sessions: ReviewSession[] = await invoke('get_all_review_sessions');
      return sessions;
    } catch (error) {
      console.error('Failed to get all review sessions:', error);
      throw new Error(`Failed to load session data: ${error}`);
    }
  }

  // Helper methods to extract normalized data from sessions
  getItemsStudied(session: ReviewSession): number {
    if (session.material_type === 'flashcard_deck') {
      return session.cards_studied || 0;
    } else if (session.material_type === 'test') {
      return session.questions_answered || 0;
    }
    return 0;
  }

  getAccuracyRate(session: ReviewSession): number {
    if (session.material_type === 'flashcard_deck') {
      // For flashcards, calculate from response counts if available
      const totalResponses = (session.again_count || 0) + (session.hard_count || 0) + 
                           (session.good_count || 0) + (session.easy_count || 0);
      if (totalResponses > 0) {
        const correctResponses = (session.good_count || 0) + (session.easy_count || 0);
        return (correctResponses / totalResponses) * 100;
      }
      // Fallback to retention_rate
      return session.retention_rate ? session.retention_rate * 100 : 0;
    } else if (session.material_type === 'test') {
      return session.score_percentage || 0;
    }
    return 0;
  }

  getAverageTime(session: ReviewSession): number | null {
    if (session.material_type === 'flashcard_deck') {
      return session.average_response_time_seconds || null;
    } else if (session.material_type === 'test') {
      return session.time_per_question_seconds || null;
    }
    return null;
  }

  calculateCurrentStreak(sessions: ReviewSession[]): number {
    if (!sessions || sessions.length === 0) return 0;

    const sortedSessions = [...sessions].sort((a, b) => 
      new Date(b.session_start).getTime() - new Date(a.session_start).getTime()
    );

    let streak = 0;
    let currentDate = new Date();
    currentDate.setHours(0, 0, 0, 0);

    const sessionsByDate = new Map<string, ReviewSession[]>();
    sortedSessions.forEach(session => {
      const sessionDate = new Date(session.session_start);
      sessionDate.setHours(0, 0, 0, 0);
      const dateKey = sessionDate.toISOString().split('T')[0];
      
      if (!sessionsByDate.has(dateKey)) {
        sessionsByDate.set(dateKey, []);
      }
      sessionsByDate.get(dateKey)!.push(session);
    });

    while (true) {
      const dateKey = currentDate.toISOString().split('T')[0];
      
      if (sessionsByDate.has(dateKey)) {
        streak++;
        currentDate.setDate(currentDate.getDate() - 1);
      } else {
        if (streak === 0 && currentDate.toDateString() === new Date().toDateString()) {
          currentDate.setDate(currentDate.getDate() - 1);
          continue;
        }
        break;
      }
    }

    return streak;
  }

  calculateThisWeekStats(sessions: ReviewSession[]): {
    sessionsCount: number;
    studyTime: number;
    flashcardSessions: number;
    testSessions: number;
  } {
    if (!sessions || sessions.length === 0) {
      return { 
        sessionsCount: 0, 
        studyTime: 0, 
        flashcardSessions: 0, 
        testSessions: 0 
      };
    }

    const now = new Date();
    const weekStart = new Date(now);
    weekStart.setDate(now.getDate() - now.getDay()); 
    weekStart.setHours(0, 0, 0, 0);

    const thisWeekSessions = sessions.filter(session => {
      const sessionDate = new Date(session.session_start);
      return sessionDate >= weekStart;
    });

    return {
      sessionsCount: thisWeekSessions.length,
      studyTime: thisWeekSessions.reduce((total, session) => total + session.total_duration_seconds, 0),
      flashcardSessions: thisWeekSessions.filter(s => s.material_type === 'flashcard_deck').length,
      testSessions: thisWeekSessions.filter(s => s.material_type === 'test').length
    };
  }

  prepareStudyTimeChartData(sessions: ReviewSession[]): StudyTimeDataPoint[] {
    console.log('Preparing study time chart data from', sessions.length, 'sessions');
    
    if (!sessions || sessions.length === 0) {
      console.log('No sessions provided');
      return [];
    }

    sessions.forEach(session => {
      console.log('Session date:', session.session_start, 'Duration:', session.total_duration_seconds);
    });

    const sessionDates = sessions.map(s => new Date(s.session_start));
    const maxSessionDate = new Date(Math.max(...sessionDates.map(d => d.getTime())));
    const today = new Date();
    
    const endDate = maxSessionDate > today ? maxSessionDate : today;
    const startDate = new Date(endDate);
    startDate.setDate(endDate.getDate() - 29);

    console.log('Chart date range:', startDate.toISOString().split('T')[0], 'to', endDate.toISOString().split('T')[0]);

    const dayMap = new Map<string, number>();
    for (let d = new Date(startDate); d <= endDate; d.setDate(d.getDate() + 1)) {
      const dateKey = d.toISOString().split('T')[0];
      dayMap.set(dateKey, 0);
    }

    sessions.forEach(session => {
      const sessionDate = new Date(session.session_start);
      const dateKey = sessionDate.toISOString().split('T')[0];
      
      if (dayMap.has(dateKey)) {
        const currentTime = dayMap.get(dateKey)!;
        const newTime = currentTime + session.total_duration_seconds;
        dayMap.set(dateKey, newTime);
        console.log('Added', session.total_duration_seconds, 'seconds to', dateKey, '(total:', newTime, ')');
      } else {
        console.log('Session date', dateKey, 'not in chart range');
      }
    });

    const chartData = Array.from(dayMap.entries()).map(([date, studyTime]) => {
      const dateObj = new Date(date);
      return {
        date,
        dateLabel: dateObj.toLocaleDateString('en-US', { 
          month: 'short', 
          day: 'numeric' 
        }),
        studyTime
      };
    });

    console.log('Final chart data points:', chartData.length);
    console.log('Non-zero days:', chartData.filter(d => d.studyTime > 0).length);
    
    return chartData;
  }

  prepareAccuracyChartData(sessions: ReviewSession[]): AccuracyDataPoint[] {
    if (!sessions || sessions.length === 0) return [];

    // Sort sessions by date and filter out sessions with no accuracy data
    const sortedSessions = [...sessions]
      .filter(session => {
        const accuracy = this.getAccuracyRate(session);
        return accuracy >= 0 && session.completed;
      })
      .sort((a, b) => 
        new Date(a.session_start).getTime() - new Date(b.session_start).getTime()
      )
      .slice(-20); // Last 20 sessions

    return sortedSessions.map(session => {
      const sessionDate = new Date(session.session_start);
      return {
        date: session.session_start,
        dateLabel: sessionDate.toLocaleDateString('en-US', { 
          month: 'short', 
          day: 'numeric' 
        }),
        accuracy: this.getAccuracyRate(session)
      };
    });
  }

  calculatePeriodStats(sessions: ReviewSession[], days: number): {
    sessionsCount: number;
    totalStudyTime: number;
    totalQuestions: number;
    averageAccuracy: number;
    averageSessionLength: number;
    flashcardSessions: number;
    testSessions: number;
  } {
    if (!sessions || sessions.length === 0) {
      return {
        sessionsCount: 0,
        totalStudyTime: 0,
        totalQuestions: 0,
        averageAccuracy: 0,
        averageSessionLength: 0,
        flashcardSessions: 0,
        testSessions: 0
      };
    }

    const cutoffDate = new Date();
    cutoffDate.setDate(cutoffDate.getDate() - days);

    const periodSessions = sessions.filter(session => 
      new Date(session.session_start) >= cutoffDate
    );

    if (periodSessions.length === 0) {
      return {
        sessionsCount: 0,
        totalStudyTime: 0,
        totalQuestions: 0,
        averageAccuracy: 0,
        averageSessionLength: 0,
        flashcardSessions: 0,
        testSessions: 0
      };
    }

    const totalStudyTime = periodSessions.reduce((sum, session) => sum + session.total_duration_seconds, 0);
    const totalQuestions = periodSessions.reduce((sum, session) => sum + this.getItemsStudied(session), 0);
    const completedSessions = periodSessions.filter(session => {
      const accuracy = this.getAccuracyRate(session);
      return session.completed && accuracy >= 0;
    });
    const averageAccuracy = completedSessions.length > 0 
      ? completedSessions.reduce((sum, session) => sum + this.getAccuracyRate(session), 0) / completedSessions.length
      : 0;

    return {
      sessionsCount: periodSessions.length,
      totalStudyTime,
      totalQuestions,
      averageAccuracy,
      averageSessionLength: totalStudyTime / periodSessions.length,
      flashcardSessions: periodSessions.filter(s => s.material_type === 'flashcard_deck').length,
      testSessions: periodSessions.filter(s => s.material_type === 'test').length
    };
  }

  getMaterialPerformance(sessions: ReviewSession[]): {
    bestMaterials: Array<{materialId: string, materialName: string, averageAccuracy: number, sessionsCount: number}>;
    worstMaterials: Array<{materialId: string, materialName: string, averageAccuracy: number, sessionsCount: number}>;
  } {
    if (!sessions || sessions.length === 0) {
      return { bestMaterials: [], worstMaterials: [] };
    }

    // Group sessions by material
    const materialStats = new Map<string, {
      materialId: string;
      materialName: string;
      totalAccuracy: number;
      sessionsCount: number;
    }>();

    sessions
      .filter(session => {
        const accuracy = this.getAccuracyRate(session);
        return session.completed && accuracy >= 0;
      })
      .forEach(session => {
        const key = session.material_id;
        const existing = materialStats.get(key);
        const accuracy = this.getAccuracyRate(session);
        
        if (existing) {
          existing.totalAccuracy += accuracy;
          existing.sessionsCount += 1;
        } else {
          materialStats.set(key, {
            materialId: session.material_id,
            materialName: session.material_name || 'Unknown Material',
            totalAccuracy: accuracy,
            sessionsCount: 1
          });
        }
      });

    // Calculate averages and sort
    const materials = Array.from(materialStats.values())
      .map(material => ({
        materialId: material.materialId,
        materialName: material.materialName,
        averageAccuracy: material.totalAccuracy / material.sessionsCount,
        sessionsCount: material.sessionsCount
      }))
      .filter(material => material.sessionsCount >= 2); // Only include materials with multiple sessions

    const sortedByAccuracy = [...materials].sort((a, b) => b.averageAccuracy - a.averageAccuracy);

    return {
      bestMaterials: sortedByAccuracy.slice(0, 5),
      worstMaterials: sortedByAccuracy.slice(-5).reverse()
    };
  }
}