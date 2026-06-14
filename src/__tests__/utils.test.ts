import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import type { Word, Definition } from '../types';

// Mock the invoke function
vi.mock('@tauri-apps/api/core');

// ========== API 测试 ==========
describe('API 测试', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('getNextReviewWord 应该调用正确的命令', async () => {
    const mockWord: Word = {
      id: 'word-1',
      word: 'test',
      definitions: [],
      status: 'new',
      created_at: Date.now(),
      updated_at: Date.now(),
    };
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockWord);

    const { getNextReviewWord } = await import('../stores/api');
    const result = await getNextReviewWord();

    expect(invoke).toHaveBeenCalledWith('get_next_review_word');
    expect(result).toEqual(mockWord);
  });

  it('submitReview 应该传递正确的参数', async () => {
    const mockResult = { is_correct: true, correct_definitions: [], next_review_interval: 60 };
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue(mockResult);

    const { submitReview } = await import('../stores/api');
    const result = await submitReview('word-1', 'hello');

    expect(invoke).toHaveBeenCalledWith('submit_review', {
      wordId: 'word-1',
      userAnswer: 'hello',
    });
    expect(result).toEqual(mockResult);
  });

  it('listWords 应该支持分页参数', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValue([]);

    const { listWords } = await import('../stores/api');
    await listWords('learning', 10, 20);

    expect(invoke).toHaveBeenCalledWith('list_words', {
      status: 'learning',
      offset: 10,
      limit: 20,
    });
  });
});

// ========== 答案匹配逻辑测试 ==========
describe('答案匹配逻辑测试', () => {
  // 模拟 Rust 后端的 normalize 和 check_answer 逻辑
  function normalize(text: string): string {
    return text.trim().toLowerCase()
      .split('')
      .filter(c => /[a-z0-9\s]/.test(c))
      .join('')
      .trim();
  }

  function checkAnswer(userAnswer: string, definitions: Definition[]): boolean {
    const normalized = normalize(userAnswer);
    // 空答案不匹配任何东西
    if (normalized.length === 0) return false;

    for (const def of definitions) {
      const defNormalized = normalize(def.definition);
      if (defNormalized === normalized ||
          defNormalized.includes(normalized) ||
          normalized.includes(defNormalized)) {
        return true;
      }
    }
    return false;
  }

  it('normalize 应该转小写并去除特殊字符', () => {
    expect(normalize('Hello, World!')).toBe('hello world');
    expect(normalize('  TEST  ')).toBe('test');
    expect(normalize('word123')).toBe('word123');
  });

  it('normalize 应该处理空字符串', () => {
    expect(normalize('')).toBe('');
    expect(normalize('   ')).toBe('');
  });

  it('checkAnswer 应该正确匹配完全相同的答案', () => {
    const defs: Definition[] = [{ id: '1', definition: 'a greeting' }];
    expect(checkAnswer('a greeting', defs)).toBe(true);
    expect(checkAnswer('A GREETING', defs)).toBe(true);
  });

  it('checkAnswer 应该支持部分匹配（用户答案包含释义）', () => {
    const defs: Definition[] = [{ id: '1', definition: 'a greeting used when meeting' }];
    expect(checkAnswer('a greeting', defs)).toBe(true);
  });

  it('checkAnswer 应该支持反向部分匹配（释义包含用户答案）', () => {
    const defs: Definition[] = [{ id: '1', definition: 'hello' }];
    expect(checkAnswer('hello world', defs)).toBe(true);
  });

  it('checkAnswer 应该正确处理不匹配的答案', () => {
    const defs: Definition[] = [{ id: '1', definition: 'a greeting' }];
    expect(checkAnswer('farewell', defs)).toBe(false);
    expect(checkAnswer('', defs)).toBe(false);
  });
});

// ========== 复习间隔计算测试 ==========
describe('复习间隔计算测试', () => {
  // 模拟 SM-2 算法的间隔计算
  function calculateNextInterval(repetitions: number, easeFactor: number, currentInterval: number): number {
    switch (repetitions) {
      case 0: return 60;   // 第一次复习：1分钟
      case 1: return 600;   // 第二次复习：10分钟
      default: return Math.round(currentInterval * easeFactor);
    }
  }

  it('第一次复习间隔应该是60秒', () => {
    expect(calculateNextInterval(0, 2.5, 60)).toBe(60);
  });

  it('第二次复习间隔应该是600秒', () => {
    expect(calculateNextInterval(1, 2.5, 60)).toBe(600);
  });

  it('第三次复习间隔应该是当前间隔乘以难度因子', () => {
    expect(calculateNextInterval(2, 2.5, 600)).toBe(1500);
  });

  it('多次复习后间隔应该指数增长', () => {
    let interval = 60;
    let ef = 2.5;
    for (let i = 0; i < 5; i++) {
      interval = calculateNextInterval(i, ef, interval);
    }
    expect(interval).toBeGreaterThan(600);
  });
});
