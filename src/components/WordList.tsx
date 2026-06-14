import { Component, createSignal, onMount, For, Show } from 'solid-js';
import { useStore } from '@nanostores/solid';
import { $wordList } from '../stores/appStore';
import { listWords, deleteWord } from '../stores/api';
import type { Word, WordStatus } from '../types';

const WordList: Component = () => {
  const wordList = useStore($wordList);
  const [isLoading, setIsLoading] = createSignal(false);
  const [filterStatus, setFilterStatus] = createSignal<WordStatus | null>(null);
  const [offset, setOffset] = createSignal(0);
  const limit = 20;

  const loadWords = async (reset = true) => {
    setIsLoading(true);
    if (reset) setOffset(0);

    try {
      const words = await listWords(filterStatus() || undefined, reset ? 0 : offset(), limit);
      if (reset) {
        $wordList.set(words);
      } else {
        $wordList.set([...$wordList.get(), ...words]);
      }
    } catch (e) {
      console.error(e);
    } finally {
      setIsLoading(false);
    }
  };

  onMount(() => {
    loadWords();
  });

  const handleDelete = async (wordId: string) => {
    if (!confirm('确定要删除这个单词吗？')) return;

    try {
      await deleteWord(wordId);
      loadWords();
    } catch (e) {
      console.error(e);
    }
  };

  const getStatusClass = (status: WordStatus) => {
    const classes: Record<WordStatus, string> = {
      new: 'status-new',
      learning: 'status-learning',
      mastered: 'status-mastered',
      skipped: 'status-skipped',
    };
    return classes[status] || '';
  };

  const getStatusText = (status: WordStatus) => {
    const texts: Record<WordStatus, string> = {
      new: '新',
      learning: '学习中',
      mastered: '已掌握',
      skipped: '已跳过',
    };
    return texts[status] || status;
  };

  return (
    <div class="word-list-panel">
      <div class="word-list-header">
        <h3>单词列表</h3>
        <div class="filter-buttons">
          <button
            class={`filter-btn ${filterStatus() === null ? 'active' : ''}`}
            onClick={() => { setFilterStatus(null); loadWords(); }}
          >
            全部
          </button>
          <button
            class={`filter-btn ${filterStatus() === 'new' ? 'active' : ''}`}
            onClick={() => { setFilterStatus('new'); loadWords(); }}
          >
            新
          </button>
          <button
            class={`filter-btn ${filterStatus() === 'learning' ? 'active' : ''}`}
            onClick={() => { setFilterStatus('learning'); loadWords(); }}
          >
            学习
          </button>
          <button
            class={`filter-btn ${filterStatus() === 'mastered' ? 'active' : ''}`}
            onClick={() => { setFilterStatus('mastered'); loadWords(); }}
          >
            掌握
          </button>
          <button
            class={`filter-btn ${filterStatus() === 'skipped' ? 'active' : ''}`}
            onClick={() => { setFilterStatus('skipped'); loadWords(); }}
          >
            跳过
          </button>
        </div>
      </div>

      <Show when={isLoading()}>
        <div class="loading">加载中...</div>
      </Show>

      <div class="word-list">
        <For each={wordList()}>
          {(word) => (
            <div class="word-item">
              <div class="word-info">
                <div class="word-main">
                  <span class="word-text">{word.word}</span>
                  <Show when={word.phonetic}>
                    <span class="word-phonetic">{word.phonetic}</span>
                  </Show>
                </div>
                <div class="word-defs">
                  {word.definitions.map(d => d.definition).join(', ')}
                </div>
              </div>
              <div class="word-actions">
                <span class={`status-badge ${getStatusClass(word.status)}`}>
                  {getStatusText(word.status)}
                </span>
                <button class="delete-btn" onClick={() => handleDelete(word.id)}>
                  删除
                </button>
              </div>
            </div>
          )}
        </For>
      </div>

      <Show when={!isLoading() && wordList().length === 0}>
        <div class="empty-state">
          <p>暂无单词</p>
        </div>
      </Show>

      <Show when={wordList().length > 0 && wordList().length % limit === 0}>
        <button class="btn btn-secondary load-more" onClick={() => { setOffset(offset() + limit); loadWords(false); }}>
          加载更多
        </button>
      </Show>
    </div>
  );
};

export default WordList;