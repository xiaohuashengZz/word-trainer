import { describe, it, expect, beforeEach, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import type { Word, Definition, ReviewResult, Statistics } from '../types';

vi.mock('@tauri-apps/api/core');

// ========== 复习流程测试 ==========
describe('复习流程测试', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  const mockWord: Word = {
    id: 'word-1',
    word: 'hello',
    phonetic: '/həˈloʊ/',
    definitions: [
      { id: 'd1', pos: 'n', definition: 'a greeting', example: 'Hello!' },
      { id: 'd2', pos: 'v', definition: 'to greet someone' },
    ],
    status: 'learning',
    created_at: 1000000,
    updated_at: 1000000,
  };

  it('应该能获取下一个复习单词', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockWord);

    const { getNextReviewWord } = await import('../stores/api');
    const result = await getNextReviewWord();

    expect(invoke).toHaveBeenCalledWith('get_next_review_word');
    expect(result).toEqual(mockWord);
    expect(result?.word).toBe('hello');
  });

  it('应该返回 null 当没有待复习单词', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(null);

    const { getNextReviewWord } = await import('../stores/api');
    const result = await getNextReviewWord();

    expect(result).toBeNull();
  });

  it('应该能提交正确答案', async () => {
    const mockResult: ReviewResult = {
      is_correct: true,
      matched_definition: { id: 'd1', definition: 'a greeting' },
      correct_definitions: mockWord.definitions,
      next_review_interval: 600,
    };
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockResult);

    const { submitReview } = await import('../stores/api');
    const result = await submitReview('word-1', 'a greeting');

    expect(invoke).toHaveBeenCalledWith('submit_review', {
      wordId: 'word-1',
      userAnswer: 'a greeting',
    });
    expect(result.is_correct).toBe(true);
  });

  it('应该能提交错误答案', async () => {
    const mockResult: ReviewResult = {
      is_correct: false,
      correct_definitions: mockWord.definitions,
      next_review_interval: 60,
    };
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockResult);

    const { submitReview } = await import('../stores/api');
    const result = await submitReview('word-1', 'wrong answer');

    expect(result.is_correct).toBe(false);
  });

  it('应该能跳过单词', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);

    const { skipWord } = await import('../stores/api');
    await skipWord('word-1');

    expect(invoke).toHaveBeenCalledWith('skip_word', { wordId: 'word-1' });
  });

  it('应该能恢复跳过的单词', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);

    const { unskipWord } = await import('../stores/api');
    await unskipWord('word-1');

    expect(invoke).toHaveBeenCalledWith('unskip_word', { wordId: 'word-1' });
  });
});

// ========== 单词管理测试 ==========
describe('单词管理测试', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('应该能添加新单词', async () => {
    const definitions: Definition[] = [
      { id: 'd1', pos: 'n', definition: 'a test' },
    ];
    const mockWord: Word = {
      id: 'new-id',
      word: 'test',
      definitions,
      status: 'new',
      created_at: Date.now(),
      updated_at: Date.now(),
    };
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockWord);

    const { addWord } = await import('../stores/api');
    const result = await addWord('test', definitions);

    expect(invoke).toHaveBeenCalledWith('add_word', {
      wordText: 'test',
      definitions,
      phonetic: undefined,
      audioUrl: undefined,
    });
    expect(result.word).toBe('test');
  });

  it('应该能删除单词', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);

    const { deleteWord } = await import('../stores/api');
    await deleteWord('word-1');

    expect(invoke).toHaveBeenCalledWith('delete_word', { wordId: 'word-1' });
  });

  it('应该能获取单词列表', async () => {
    const mockWords: Word[] = [
      { id: '1', word: 'hello', definitions: [], status: 'new', created_at: 0, updated_at: 0 },
      { id: '2', word: 'world', definitions: [], status: 'learning', created_at: 0, updated_at: 0 },
    ];
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockWords);

    const { listWords } = await import('../stores/api');
    const result = await listWords('learning', 0, 10);

    expect(invoke).toHaveBeenCalledWith('list_words', {
      status: 'learning',
      offset: 0,
      limit: 10,
    });
    expect(result).toHaveLength(2);
  });

  it('应该能获取所有单词列表', async () => {
    const mockWords: Word[] = [];
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockWords);

    const { listWords } = await import('../stores/api');
    await listWords();

    expect(invoke).toHaveBeenCalledWith('list_words', {
      status: undefined,
      offset: 0,
      limit: 50,
    });
  });

  it('应该能获取单词数量', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(42);

    const { getWordCount } = await import('../stores/api');
    const result = await getWordCount();

    expect(invoke).toHaveBeenCalledWith('get_word_count');
    expect(result).toBe(42);
  });
});

// ========== 统计测试 ==========
describe('统计测试', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('应该能获取统计信息', async () => {
    const mockStats: Statistics = {
      total_words: 100,
      new_words: 20,
      learning_words: 30,
      mastered_words: 40,
      skipped_words: 10,
      total_reviews: 500,
      total_correct: 450,
      total_incorrect: 50,
      correct_rate: 90.0,
    };
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockStats);

    const { getStatistics } = await import('../stores/api');
    const result = await getStatistics();

    expect(result.total_words).toBe(100);
    expect(result.new_words).toBe(20);
    expect(result.correct_rate).toBe(90.0);
  });

  it('应该正确计算正确率', async () => {
    const mockStats: Statistics = {
      total_words: 10,
      new_words: 0,
      learning_words: 5,
      mastered_words: 5,
      skipped_words: 0,
      total_reviews: 100,
      total_correct: 85,
      total_incorrect: 15,
      correct_rate: 85.0,
    };
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockStats);

    const { getStatistics } = await import('../stores/api');
    const result = await getStatistics();

    expect(result.correct_rate).toBe(85.0);
    expect(result.total_correct / result.total_reviews).toBeCloseTo(0.85);
  });
});

// ========== 设置测试 ==========
describe('设置测试', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('应该能获取设置', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue('30');

    const { getSetting } = await import('../stores/api');
    const result = await getSetting('review_interval');

    expect(invoke).toHaveBeenCalledWith('get_setting', { key: 'review_interval' });
    expect(result).toBe('30');
  });

  it('应该能获取不存在的设置返回 null', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(null);

    const { getSetting } = await import('../stores/api');
    const result = await getSetting('nonexistent');

    expect(result).toBeNull();
  });

  it('应该能设置设置', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);

    const { setSetting } = await import('../stores/api');
    await setSetting('review_interval', '60');

    expect(invoke).toHaveBeenCalledWith('set_setting', {
      key: 'review_interval',
      value: '60',
    });
  });
});
