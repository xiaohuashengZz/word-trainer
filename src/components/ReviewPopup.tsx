import { Component, createSignal, Show, onMount, onCleanup } from 'solid-js';
import { useStore } from '@nanostores/solid';
import { listen } from '@tauri-apps/api/event';
import { $currentWord, $userAnswer, $showResult, $reviewResult, $reviewState, startReview, showReviewResult, resetReviewState } from '../stores/appStore';
import { getNextReviewWord, submitReview, skipWord } from '../stores/api';
import type { Word, ReviewResult } from '../types';

const ReviewPopup: Component = () => {
  const currentWord = useStore($currentWord);
  const userAnswer = useStore($userAnswer);
  const showResult = useStore($showResult);
  const reviewResult = useStore($reviewResult);
  const reviewState = useStore($reviewState);

  const [isLoading, setIsLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [showReminder, setShowReminder] = createSignal(false);

  // 监听提醒事件
  onMount(async () => {
    // 监听复习提醒事件
    const unlistenReminder = await listen('review-reminder', () => {
      console.log('收到复习提醒');
      setShowReminder(true);
      loadNextWord();
    });

    // 监听开始复习事件（从托盘菜单触发）
    const unlistenStartReview = await listen('start-review', () => {
      console.log('收到开始复习事件');
      loadNextWord();
    });

    onCleanup(() => {
      unlistenReminder();
      unlistenStartReview();
    });
  });

  // 关闭提醒弹窗
  const dismissReminder = () => {
    setShowReminder(false);
  };

  // 加载下一个单词
  const loadNextWord = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const word = await getNextReviewWord();
      if (word) {
        startReview(word);
      } else {
        resetReviewState();
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  };

  // 提交答案
  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    const word = currentWord();
    if (!word) return;

    setIsLoading(true);
    setError(null);
    try {
      const result = await submitReview(word.id, userAnswer());
      showReviewResult(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  };

  // 跳过单词
  const handleSkip = async () => {
    const word = currentWord();
    if (!word) return;

    try {
      await skipWord(word.id);
      await loadNextWord();
    } catch (e) {
      setError(String(e));
    }
  };

  // 播放发音
  const playPronunciation = (url: string | undefined) => {
    if (url) {
      const audio = new Audio(url);
      audio.play().catch(console.error);
    }
  };

  // 初始化加载
  if (reviewState() === 'idle' && !currentWord()) {
    loadNextWord();
  }

  return (
    <div class="review-popup">
      <Show when={isLoading()}>
        <div class="loading-overlay">
          <div class="spinner" />
        </div>
      </Show>

      <Show when={error()}>
        <div class="error-message">{error()}</div>
      </Show>

      <Show when={!currentWord() && !isLoading()}>
        <div class="empty-state">
          <div class="empty-icon">📚</div>
          <h3>暂无待复习单词</h3>
          <p>请先添加单词或下载词库</p>
          <button class="btn btn-primary" onClick={loadNextWord}>
            刷新
          </button>
        </div>
      </Show>

      <Show when={currentWord()}>
        <div class="word-card">
          <div class="word-header">
            <h2 class="word-text">{currentWord()?.word}</h2>
            <Show when={currentWord()?.phonetic}>
              <span class="phonetic">{currentWord()?.phonetic}</span>
            </Show>
          </div>

          <Show when={currentWord()?.phonetic_audio_url}>
            <button
              class="audio-btn"
              onClick={() => playPronunciation(currentWord()?.phonetic_audio_url)}
            >
              🔊 播放发音
            </button>
          </Show>

          <Show when={showResult() && reviewResult()}>
            <div class={`result-box ${reviewResult()?.is_correct ? 'correct' : 'incorrect'}`}>
              <Show when={reviewResult()?.is_correct}>
                <div class="result-icon">✓</div>
                <div class="result-text">回答正确！</div>
              </Show>
              <Show when={!reviewResult()?.is_correct}>
                <div class="result-icon">✕</div>
                <div class="result-text">回答错误</div>
              </Show>
            </div>

            <div class="definitions-box">
              <div class="definitions-title">释义：</div>
              <div class="definitions-list">
                {reviewResult()?.correct_definitions.map((def, i) => (
                  <div class="definition-item">
                    <span class="pos">{def.pos || ''}</span>
                    <span class="def-text">{def.definition}</span>
                  </div>
                ))}
              </div>
            </div>
          </Show>

          <Show when={!showResult()}>
            <form onSubmit={handleSubmit}>
              <div class="input-group">
                <label for="answer">请输入中文意思</label>
                <input
                  id="answer"
                  type="text"
                  class="answer-input"
                  value={userAnswer()}
                  onInput={(e) => $userAnswer.set(e.currentTarget.value)}
                  placeholder="输入答案后按回车提交"
                  autofocus
                />
              </div>
              <div class="button-group">
                <button type="submit" class="btn btn-primary" disabled={isLoading()}>
                  提交
                </button>
                <button type="button" class="btn btn-skip" onClick={handleSkip}>
                  跳过
                </button>
              </div>
            </form>
          </Show>

          <Show when={showResult()}>
            <div class="button-group">
              <button class="btn btn-primary" onClick={loadNextWord}>
                下一题
              </button>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
};

export default ReviewPopup;