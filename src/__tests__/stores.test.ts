import { describe, it, expect, beforeEach } from 'vitest';
import { $currentWord, $reviewResult, $userAnswer, $showResult, $reviewState, resetReviewState, startReview, showReviewResult, nextQuestion, $theme } from '../stores/appStore';
import type { Word, ReviewResult } from '../types';

// ========== Store 状态测试 ==========
describe('AppStore 测试', () => {
  const mockWord: Word = {
    id: 'word-1',
    word: 'hello',
    definitions: [{ id: 'd1', definition: 'greeting' }],
    status: 'new',
    created_at: Date.now(),
    updated_at: Date.now(),
  };

  const mockResult: ReviewResult = {
    is_correct: true,
    matched_definition: { id: 'd1', definition: 'greeting' },
    correct_definitions: [{ id: 'd1', definition: 'greeting' }],
    next_review_interval: 60,
  };

  beforeEach(() => {
    // 每个测试前重置状态
    resetReviewState();
  });

  it('resetReviewState 应该重置所有状态', () => {
    startReview(mockWord);
    expect($currentWord.get()).toEqual(mockWord);

    resetReviewState();
    expect($currentWord.get()).toBeNull();
    expect($reviewResult.get()).toBeNull();
    expect($userAnswer.get()).toBe('');
    expect($showResult.get()).toBe(false);
    expect($reviewState.get()).toBe('idle');
  });

  it('startReview 应该设置当前单词并进入答题状态', () => {
    startReview(mockWord);

    expect($currentWord.get()).toEqual(mockWord);
    expect($reviewState.get()).toBe('answering');
    expect($userAnswer.get()).toBe('');
    expect($showResult.get()).toBe(false);
  });

  it('showReviewResult 应该显示复习结果', () => {
    startReview(mockWord);
    showReviewResult(mockResult);

    expect($reviewResult.get()).toEqual(mockResult);
    expect($showResult.get()).toBe(true);
    expect($reviewState.get()).toBe('result');
  });

  it('nextQuestion 应该重置状态', () => {
    startReview(mockWord);
    showReviewResult(mockResult);

    nextQuestion();

    expect($currentWord.get()).toBeNull();
    expect($reviewResult.get()).toBeNull();
    expect($reviewState.get()).toBe('idle');
  });
});

// ========== 用户输入状态测试 ==========
describe('用户输入状态测试', () => {
  beforeEach(() => {
    $userAnswer.set('');
  });

  it('应该能设置和获取用户答案', () => {
    $userAnswer.set('hello');
    expect($userAnswer.get()).toBe('hello');
  });

  it('应该能清空用户答案', () => {
    $userAnswer.set('hello');
    $userAnswer.set('');
    expect($userAnswer.get()).toBe('');
  });

  it('应该能更新用户答案', () => {
    $userAnswer.set('hel');
    $userAnswer.set('hello');
    expect($userAnswer.get()).toBe('hello');
  });
});

// ========== 主题切换测试 ==========
describe('主题切换测试', () => {
  it('默认主题应该是 light', () => {
    expect($theme.get()).toBe('light');
  });
});
