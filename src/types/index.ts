// 类型定义

export interface Definition {
  id: string;
  pos?: string;
  definition: string;
  example?: string;
}

export type WordStatus = 'new' | 'learning' | 'mastered' | 'skipped';

export interface Word {
  id: string;
  word: string;
  phonetic?: string;
  phonetic_audio_url?: string;
  definitions: Definition[];
  status: WordStatus;
  created_at: number;
  updated_at: number;
}

export interface ReviewResult {
  is_correct: boolean;
  matched_definition?: Definition;
  correct_definitions: Definition[];
  phonetic?: string;
  phonetic_audio_url?: string;
  next_review_interval: number;
}

export interface Statistics {
  total_words: number;
  new_words: number;
  learning_words: number;
  mastered_words: number;
  skipped_words: number;
  total_reviews: number;
  total_correct: number;
  total_incorrect: number;
  correct_rate: number;
}