import { invoke } from '@tauri-apps/api/core';
import type { Word, Definition, ReviewResult, Statistics } from '../types';

// 获取下一个待复习单词
export async function getNextReviewWord(): Promise<Word | null> {
  return await invoke<Word | null>('get_next_review_word');
}

// 提交复习答案
export async function submitReview(wordId: string, userAnswer: string): Promise<ReviewResult> {
  return await invoke<ReviewResult>('submit_review', { wordId, userAnswer });
}

// 跳过单词
export async function skipWord(wordId: string): Promise<void> {
  return await invoke<void>('skip_word', { wordId });
}

// 恢复单词
export async function unskipWord(wordId: string): Promise<void> {
  return await invoke<void>('unskip_word', { wordId });
}

// 获取单词列表
export async function listWords(status?: string, offset = 0, limit = 50): Promise<Word[]> {
  return await invoke<Word[]>('list_words', { status, offset, limit });
}

// 添加单词
export async function addWord(
  wordText: string,
  definitions: Definition[],
  phonetic?: string,
  audioUrl?: string
): Promise<Word> {
  return await invoke<Word>('add_word', { wordText, definitions, phonetic, audioUrl });
}

// 删除单词
export async function deleteWord(wordId: string): Promise<void> {
  return await invoke<void>('delete_word', { wordId });
}

// 获取统计信息
export async function getStatistics(): Promise<Statistics> {
  return await invoke<Statistics>('get_statistics');
}

// 获取设置
export async function getSetting(key: string): Promise<string | null> {
  return await invoke<string | null>('get_setting', { key });
}

// 设置设置
export async function setSetting(key: string, value: string): Promise<void> {
  return await invoke<void>('set_setting', { key, value });
}

// 获取单词数量
export async function getWordCount(): Promise<number> {
  return await invoke<number>('get_word_count');
}