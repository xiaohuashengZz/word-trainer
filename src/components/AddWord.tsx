import { Component, createSignal, Show } from 'solid-js';
import { useStore } from '@nanostores/solid';
import { $statistics } from '../stores/appStore';
import { getStatistics } from '../stores/api';
import { onMount } from 'solid-js';

const Statistics: Component = () => {
  const stats = useStore($statistics);
  const [isLoading, setIsLoading] = createSignal(true);

  const loadStats = async () => {
    setIsLoading(true);
    try {
      const data = await getStatistics();
      $statistics.set(data);
    } catch (e) {
      console.error(e);
    } finally {
      setIsLoading(false);
    }
  };

  onMount(() => {
    loadStats();
  });

  return (
    <div class="statistics-panel">
      <h3>学习统计</h3>

      <Show when={isLoading()}>
        <div class="loading">加载中...</div>
      </Show>

      <Show when={!isLoading() && stats()}>
        <div class="stats-grid">
          <div class="stat-card">
            <div class="stat-value">{stats()?.total_words || 0}</div>
            <div class="stat-label">单词总数</div>
          </div>

          <div class="stat-card new">
            <div class="stat-value">{stats()?.new_words || 0}</div>
            <div class="stat-label">新单词</div>
          </div>

          <div class="stat-card learning">
            <div class="stat-value">{stats()?.learning_words || 0}</div>
            <div class="stat-label">学习中</div>
          </div>

          <div class="stat-card mastered">
            <div class="stat-value">{stats()?.mastered_words || 0}</div>
            <div class="stat-label">已掌握</div>
          </div>

          <div class="stat-card skipped">
            <div class="stat-value">{stats()?.skipped_words || 0}</div>
            <div class="stat-label">已跳过</div>
          </div>
        </div>

        <div class="review-stats">
          <h4>复习统计</h4>
          <div class="review-stats-grid">
            <div class="review-stat">
              <span class="review-stat-value">{stats()?.total_reviews || 0}</span>
              <span class="review-stat-label">总复习次数</span>
            </div>
            <div class="review-stat correct">
              <span class="review-stat-value">{stats()?.total_correct || 0}</span>
              <span class="review-stat-label">正确</span>
            </div>
            <div class="review-stat incorrect">
              <span class="review-stat-value">{stats()?.total_incorrect || 0}</span>
              <span class="review-stat-label">错误</span>
            </div>
          </div>
          <div class="correct-rate">
            正确率: <strong>{stats()?.correct_rate.toFixed(1) || 0}%</strong>
          </div>
        </div>

        <button class="btn btn-secondary" onClick={loadStats}>
          刷新统计
        </button>
      </Show>
    </div>
  );
};

export default Statistics;