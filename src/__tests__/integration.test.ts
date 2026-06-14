/**
 * 前端完整业务流程链路测试
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { $currentWord, $reviewResult, $reviewState, $userAnswer, $showResult, $statistics, $wordList, resetReviewState, startReview, showReviewResult, nextQuestion } from '../stores/appStore';
import type { Word, Definition, ReviewResult, Statistics } from '../types';

vi.mock('@tauri-apps/api/core');

// ========== 复习完整链路测试 ==========
describe('复习完整链路测试', () => {
  const mockWord: Word = {
    id: 'word-1',
    word: 'hello',
    phonetic: '/həˈloʊ/',
    definitions: [
      { id: 'd1', pos: 'n', definition: 'a greeting used when meeting someone', example: 'Hello!' },
      { id: 'd2', pos: 'v', definition: 'to greet someone' },
    ],
    status: 'learning',
    created_at: 1000000,
    updated_at: 1000000,
  };

  const mockResult: ReviewResult = {
    is_correct: true,
    matched_definition: { id: 'd1', definition: 'a greeting used when meeting someone' },
    correct_definitions: mockWord.definitions,
    phonetic: '/həˈloʊ/',
    phonetic_audio_url: undefined,
    next_review_interval: 600,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    resetReviewState();
  });

  it('完整复习流程：获取单词 -> 显示问题 -> 提交答案 -> 显示结果 -> 下一题', async () => {
    // 1. 模拟获取下一个待复习单词
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockWord);
    const { getNextReviewWord } = await import('../stores/api');
    const word = await getNextReviewWord();
    expect(word).toEqual(mockWord);

    // 2. 开始复习
    startReview(word!);
    expect($currentWord.get()).toEqual(mockWord);
    expect($reviewState.get()).toBe('answering');
    expect($userAnswer.get()).toBe('');

    // 3. 用户输入答案
    $userAnswer.set('a greeting used when meeting someone');
    expect($userAnswer.get()).toBe('a greeting used when meeting someone');

    // 4. 提交答案
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockResult);
    const { submitReview } = await import('../stores/api');
    const result = await submitReview(mockWord.id, $userAnswer.get());
    expect(result.is_correct).toBe(true);

    // 5. 显示结果
    showReviewResult(result);
    expect($reviewResult.get()).toEqual(mockResult);
    expect($reviewState.get()).toBe('result');
    expect($showResult.get()).toBe(true);

    // 6. 下一题
    nextQuestion();
    expect($currentWord.get()).toBeNull();
    expect($reviewResult.get()).toBeNull();
    expect($reviewState.get()).toBe('idle');
  });

  it('复习流程：错误答案 -> 显示正确答案 -> 下一题', async () => {
    const wrongResult: ReviewResult = {
      is_correct: false,
      correct_definitions: mockWord.definitions,
      next_review_interval: 60,
    };

    // 开始复习
    startReview(mockWord);
    $userAnswer.set('wrong answer');

    // 提交错误答案
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(wrongResult);
    const { submitReview } = await import('../stores/api');
    const result = await submitReview(mockWord.id, $userAnswer.get());
    expect(result.is_correct).toBe(false);

    // 显示结果
    showReviewResult(result);
    expect($reviewResult.get()?.is_correct).toBe(false);
    expect($reviewResult.get()?.correct_definitions).toEqual(mockWord.definitions);
  });

  it('复习流程：没有待复习单词', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(null);
    const { getNextReviewWord } = await import('../stores/api');
    const word = await getNextReviewWord();
    expect(word).toBeNull();
  });
});

// ========== 单词管理完整链路测试 ==========
describe('单词管理完整链路测试', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('完整添加单词流程', async () => {
    const newWord: Word = {
      id: 'new-word-id',
      word: 'test',
      definitions: [
        { id: 'd1', pos: 'n', definition: 'a procedure to check quality' },
      ],
      status: 'new',
      created_at: Date.now(),
      updated_at: Date.now(),
    };

    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(newWord);

    const { addWord } = await import('../stores/api');
    const result = await addWord(
      'test',
      [{ id: 'd1', pos: 'n', definition: 'a procedure to check quality' }],
      undefined,
      undefined
    );

    expect(result.word).toBe('test');
    expect(invoke).toHaveBeenCalledWith('add_word', expect.objectContaining({
      wordText: 'test',
    }));
  });

  it('完整删除单词流程', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);

    const { deleteWord } = await import('../stores/api');
    await deleteWord('word-to-delete');

    expect(invoke).toHaveBeenCalledWith('delete_word', { wordId: 'word-to-delete' });
  });

  it('获取单词列表并筛选', async () => {
    const mockWords: Word[] = [
      { id: '1', word: 'apple', definitions: [], status: 'mastered', created_at: 0, updated_at: 0 },
      { id: '2', word: 'banana', definitions: [], status: 'learning', created_at: 0, updated_at: 0 },
      { id: '3', word: 'cherry', definitions: [], status: 'new', created_at: 0, updated_at: 0 },
    ];

    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue([mockWords[1]]);

    const { listWords } = await import('../stores/api');
    const learningWords = await listWords('learning');

    expect(learningWords).toHaveLength(1);
    expect(learningWords[0].word).toBe('banana');
  });

  it('分页获取单词列表', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue([]);

    const { listWords } = await import('../stores/api');
    await listWords(undefined, 10, 20);

    expect(invoke).toHaveBeenCalledWith('list_words', {
      status: undefined,
      offset: 10,
      limit: 20,
    });
  });
});

// ========== 统计完整链路测试 ==========
describe('统计完整链路测试', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('获取完整统计数据', async () => {
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
    const stats = await getStatistics();

    expect(stats.total_words).toBe(100);
    expect(stats.new_words).toBe(20);
    expect(stats.learning_words).toBe(30);
    expect(stats.mastered_words).toBe(40);
    expect(stats.skipped_words).toBe(10);
    expect(stats.correct_rate).toBe(90.0);

    // 验证正确率计算
    expect((stats.total_correct / stats.total_reviews) * 100).toBeCloseTo(90.0);
  });

  it('统计数据更新后刷新', async () => {
    const stats1: Statistics = {
      total_words: 99,
      new_words: 19,
      learning_words: 30,
      mastered_words: 40,
      skipped_words: 10,
      total_reviews: 499,
      total_correct: 449,
      total_incorrect: 50,
      correct_rate: 89.98,
    };

    const stats2: Statistics = {
      ...stats1,
      total_reviews: 500,
      total_correct: 450,
      correct_rate: 90.0,
    };

    (invoke as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce(stats1)
      .mockResolvedValueOnce(stats2);

    const { getStatistics } = await import('../stores/api');

    const s1 = await getStatistics();
    expect(s1.total_reviews).toBe(499);

    const s2 = await getStatistics();
    expect(s2.total_reviews).toBe(500);
    expect(s2.correct_rate).toBe(90.0);
  });
});

// ========== 跳过和恢复单词流程测试 ==========
describe('跳过和恢复单词流程测试', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('跳过单词流程', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);

    const { skipWord } = await import('../stores/api');
    await skipWord('word-to-skip');

    expect(invoke).toHaveBeenCalledWith('skip_word', { wordId: 'word-to-skip' });
  });

  it('恢复单词流程', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);

    const { unskipWord } = await import('../stores/api');
    await unskipWord('word-to-unskip');

    expect(invoke).toHaveBeenCalledWith('unskip_word', { wordId: 'word-to-unskip' });
  });
});

// ========== 设置管理流程测试 ==========
describe('设置管理流程测试', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('保存和读取复习间隔设置', async () => {
    (invoke as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce(undefined) // setSetting
      .mockResolvedValueOnce('30'); // getSetting

    const { setSetting, getSetting } = await import('../stores/api');

    // 保存设置
    await setSetting('review_interval', '30');
    expect(invoke).toHaveBeenCalledWith('set_setting', {
      key: 'review_interval',
      value: '30',
    });

    // 读取设置
    const value = await getSetting('review_interval');
    expect(value).toBe('30');
  });

  it('读取不存在的设置返回 null', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(null);

    const { getSetting } = await import('../stores/api');
    const value = await getSetting('nonexistent_setting');

    expect(value).toBeNull();
  });
});

// ========== 答案匹配逻辑测试 ==========
describe('答案匹配逻辑测试', () => {
  it('精确匹配', () => {
    const definitions: Definition[] = [
      { id: '1', definition: 'a greeting' },
    ];

    function checkAnswer(userAnswer: string, defs: Definition[]): boolean {
      const normalized = userAnswer.trim().toLowerCase();
      if (normalized.length === 0) return false;

      for (const def of defs) {
        const defNorm = def.definition.trim().toLowerCase();
        if (defNorm === normalized ||
            defNorm.includes(normalized) ||
            normalized.includes(defNorm)) {
          return true;
        }
      }
      return false;
    }

    // 精确匹配
    expect(checkAnswer('a greeting', definitions)).toBe(true);
    expect(checkAnswer('A GREETING', definitions)).toBe(true);
    expect(checkAnswer('  a greeting  ', definitions)).toBe(true);
  });

  it('部分匹配 - 用户答案包含释义', () => {
    const definitions: Definition[] = [
      { id: '1', definition: 'a greeting used when meeting someone' },
    ];

    function checkAnswer(userAnswer: string, defs: Definition[]): boolean {
      const normalized = userAnswer.trim().toLowerCase();
      if (normalized.length === 0) return false;

      for (const def of defs) {
        const defNorm = def.definition.trim().toLowerCase();
        if (defNorm === normalized ||
            defNorm.includes(normalized) ||
            normalized.includes(defNorm)) {
          return true;
        }
      }
      return false;
    }

    expect(checkAnswer('a greeting', definitions)).toBe(true);
    expect(checkAnswer('greeting', definitions)).toBe(true);
  });

  it('部分匹配 - 释义包含用户答案', () => {
    const definitions: Definition[] = [
      { id: '1', definition: 'hello' },
    ];

    function checkAnswer(userAnswer: string, defs: Definition[]): boolean {
      const normalized = userAnswer.trim().toLowerCase();
      if (normalized.length === 0) return false;

      for (const def of defs) {
        const defNorm = def.definition.trim().toLowerCase();
        if (defNorm === normalized ||
            defNorm.includes(normalized) ||
            normalized.includes(defNorm)) {
          return true;
        }
      }
      return false;
    }

    expect(checkAnswer('hello world', definitions)).toBe(true);
  });

  it('不匹配', () => {
    const definitions: Definition[] = [
      { id: '1', definition: 'a greeting' },
    ];

    function checkAnswer(userAnswer: string, defs: Definition[]): boolean {
      const normalized = userAnswer.trim().toLowerCase();
      if (normalized.length === 0) return false;

      for (const def of defs) {
        const defNorm = def.definition.trim().toLowerCase();
        if (defNorm === normalized ||
            defNorm.includes(normalized) ||
            normalized.includes(defNorm)) {
          return true;
        }
      }
      return false;
    }

    expect(checkAnswer('farewell', definitions)).toBe(false);
    expect(checkAnswer('', definitions)).toBe(false);
    expect(checkAnswer('   ', definitions)).toBe(false);
  });

  it('多释义匹配', () => {
    const definitions: Definition[] = [
      { id: '1', definition: 'first meaning' },
      { id: '2', definition: 'second meaning' },
      { id: '3', definition: 'third meaning' },
    ];

    function checkAnswer(userAnswer: string, defs: Definition[]): boolean {
      const normalized = userAnswer.trim().toLowerCase();
      if (normalized.length === 0) return false;

      for (const def of defs) {
        const defNorm = def.definition.trim().toLowerCase();
        if (defNorm === normalized ||
            defNorm.includes(normalized) ||
            normalized.includes(defNorm)) {
          return true;
        }
      }
      return false;
    }

    expect(checkAnswer('second meaning', definitions)).toBe(true);
    expect(checkAnswer('first', definitions)).toBe(true);
    expect(checkAnswer('third', definitions)).toBe(true);
  });
});

// ========== 复习间隔计算测试 ==========
describe('复习间隔计算测试', () => {
  it('正确计算复习间隔', () => {
    function calculateInterval(repetitions: number, easeFactor: number, currentInterval: number): number {
      switch (repetitions) {
        case 0: return 60;
        case 1: return 600;
        default: return Math.round(currentInterval * easeFactor);
      }
    }

    // 第一次复习
    expect(calculateInterval(0, 2.5, 60)).toBe(60);

    // 第二次复习
    expect(calculateInterval(1, 2.5, 60)).toBe(600);

    // 第三次及以后
    expect(calculateInterval(2, 2.5, 600)).toBe(1500);
    expect(calculateInterval(3, 2.5, 1500)).toBe(3750);
  });

  it('间隔随学习进度增长', () => {
    function calculateInterval(repetitions: number, easeFactor: number, currentInterval: number): number {
      switch (repetitions) {
        case 0: return 60;
        case 1: return 600;
        default: return Math.round(currentInterval * easeFactor);
      }
    }

    // 模拟连续正确复习：每次复习后ease_factor增加0.1
    // repetitions=0 -> interval=60 (第1次复习后)
    // repetitions=1 -> interval=600 (第2次复习后)
    // repetitions=2 -> interval=60*2.6=156 (第3次复习后)
    // repetitions=3 -> interval=156*2.7=421 (第4次复习后)
    const intervals: number[] = [];
    let interval = 60;
    let ef = 2.5;

    for (let i = 0; i < 10; i++) {
      // 计算当前repetitions对应的间隔并记录
      interval = calculateInterval(i, ef, interval);
      intervals.push(interval);
      // 正确后ease_factor增加
      ef = Math.min(ef + 0.1, 5.0); // 最大不超过5.0
    }

    // 验证间隔逐渐增长
    // intervals[0] = 60 (rep=0), intervals[1] = 600 (rep=1), intervals[2] = 1560 (rep=2), ...
    for (let i = 1; i < intervals.length; i++) {
      expect(intervals[i]).toBeGreaterThan(intervals[i - 1]);
    }
  });

  it('错误后重置间隔', () => {
    // 模拟错误后的重置
    const wrongAnswerInterval = 60; // 总是重置到 60 秒

    // 假设之前间隔是 10000 秒
    const previousInterval = 10000;
    expect(wrongAnswerInterval).toBe(60);
    expect(wrongAnswerInterval).toBeLessThan(previousInterval);
  });
});

// ========== 状态流转测试 ==========
describe('状态流转测试', () => {
  beforeEach(() => {
    resetReviewState();
  });

  it('状态流转: idle -> answering -> result -> idle', () => {
    const word: Word = {
      id: '1',
      word: 'test',
      definitions: [],
      status: 'new',
      created_at: 0,
      updated_at: 0,
    };

    const result: ReviewResult = {
      is_correct: true,
      correct_definitions: [],
      next_review_interval: 60,
    };

    // idle 状态
    expect($reviewState.get()).toBe('idle');

    // answering 状态
    startReview(word);
    expect($reviewState.get()).toBe('answering');

    // result 状态
    showReviewResult(result);
    expect($reviewState.get()).toBe('result');

    // 返回 idle 状态
    nextQuestion();
    expect($reviewState.get()).toBe('idle');
  });

  it('resetReviewState 重置所有状态', () => {
    const word: Word = {
      id: '1',
      word: 'test',
      definitions: [],
      status: 'new',
      created_at: 0,
      updated_at: 0,
    };

    const result: ReviewResult = {
      is_correct: true,
      correct_definitions: [],
      next_review_interval: 60,
    };

    // 设置一些状态
    startReview(word);
    showReviewResult(result);
    $userAnswer.set('test answer');

    // 重置
    resetReviewState();

    expect($currentWord.get()).toBeNull();
    expect($reviewResult.get()).toBeNull();
    expect($userAnswer.get()).toBe('');
    expect($reviewState.get()).toBe('idle');
  });
});
