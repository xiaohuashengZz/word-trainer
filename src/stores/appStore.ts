import { atom, map } from 'nanostores';
import type { Word, ReviewResult, Statistics } from '../types';

// 当前复习的单词
export const $currentWord = atom<Word | null>(null);

// 复习结果
export const $reviewResult = atom<ReviewResult | null>(null);

// 加载状态
export const $isLoading = atom<boolean>(false);

// 用户输入的答案
export const $userAnswer = atom<string>('');

// 是否显示结果
export const $showResult = atom<boolean>(false);

// 统计数据
export const $statistics = atom<Statistics | null>(null);

// 单词列表
export const $wordList = atom<Word[]>([]);

// 复习状态
export type ReviewState = 'idle' | 'answering' | 'result';
export const $reviewState = atom<ReviewState>('idle');

// 设置相关
export const $reviewInterval = atom<number>(30); // 分钟

// 主题
export type Theme = 'light' | 'dark';
export const $theme = atom<Theme>('light');

// 重置复习状态
export function resetReviewState() {
  $currentWord.set(null);
  $reviewResult.set(null);
  $userAnswer.set('');
  $showResult.set(false);
  $reviewState.set('idle');
}

// 设置当前单词并进入答题状态
export function startReview(word: Word) {
  $currentWord.set(word);
  $reviewResult.set(null);
  $userAnswer.set('');
  $showResult.set(false);
  $reviewState.set('answering');
}

// 显示结果
export function showReviewResult(result: ReviewResult) {
  $reviewResult.set(result);
  $showResult.set(true);
  $reviewState.set('result');
}

// 下一题
export function nextQuestion() {
  resetReviewState();
}

// 切换主题
export function toggleTheme() {
  const current = $theme.get();
  $theme.set(current === 'light' ? 'dark' : 'light');
}