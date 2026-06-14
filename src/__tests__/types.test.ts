import { describe, it, expect } from 'vitest';
import type { Word, Definition, WordStatus, ReviewResult, Statistics } from '../types';

// ========== 类型测试 ==========
describe('类型定义测试', () => {
  it('应该正确定义 Word 类型', () => {
    const word: Word = {
      id: '123',
      word: 'hello',
      definitions: [],
      status: 'new',
      created_at: Date.now(),
      updated_at: Date.now(),
    };
    expect(word.word).toBe('hello');
    expect(word.status).toBe('new');
  });

  it('应该正确定义 Definition 类型', () => {
    const def: Definition = {
      id: '1',
      definition: 'a greeting',
    };
    expect(def.definition).toBe('a greeting');
    expect(def.pos).toBeUndefined();
  });

  it('应该正确定义带可选字段的 Definition', () => {
    const def: Definition = {
      id: '1',
      pos: 'n',
      definition: 'a greeting',
      example: 'Hello, world!',
    };
    expect(def.pos).toBe('n');
    expect(def.example).toBe('Hello, world!');
  });

  it('应该正确定义 WordStatus 类型', () => {
    const statuses: WordStatus[] = ['new', 'learning', 'mastered', 'skipped'];
    expect(statuses).toHaveLength(4);
  });

  it('应该正确定义 ReviewResult 类型', () => {
    const result: ReviewResult = {
      is_correct: true,
      correct_definitions: [{ id: '1', definition: 'greeting' }],
      next_review_interval: 60,
    };
    expect(result.is_correct).toBe(true);
  });

  it('应该正确定义 Statistics 类型', () => {
    const stats: Statistics = {
      total_words: 100,
      new_words: 20,
      learning_words: 30,
      mastered_words: 40,
      skipped_words: 10,
      total_reviews: 500,
      total_correct: 450,
      total_incorrect: 50,
      correct_rate: 90,
    };
    expect(stats.total_words).toBe(100);
    expect(stats.correct_rate).toBe(90);
  });
});
