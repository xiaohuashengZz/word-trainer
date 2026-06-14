import { Component, createSignal, For } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';

interface WordBook {
  id: string;
  name: string;
  level: string;
  description: string;
  wordCount: number;
}

const WORD_BOOKS: WordBook[] = [
  { id: 'primary1', name: '小学一年级', level: '小学', description: '小学一年级必备词汇', wordCount: 200 },
  { id: 'primary2', name: '小学二年级', level: '小学', description: '小学二年级必备词汇', wordCount: 250 },
  { id: 'primary3', name: '小学三年级', level: '小学', description: '小学三年级必备词汇', wordCount: 300 },
  { id: 'primary4', name: '小学四年级', level: '小学', description: '小学四年级必备词汇', wordCount: 350 },
  { id: 'primary5', name: '小学五年级', level: '小学', description: '小学五年级必备词汇', wordCount: 400 },
  { id: 'primary6', name: '小学六年级', level: '小学', description: '小学六年级必备词汇', wordCount: 450 },
  { id: 'junior1', name: '初一', level: '初中', description: '初中一年级必备词汇', wordCount: 500 },
  { id: 'junior2', name: '初二', level: '初中', description: '初中二年级必备词汇', wordCount: 600 },
  { id: 'junior3', name: '初三', level: '初中', description: '初中三年级必备词汇', wordCount: 700 },
  { id: 'senior1', name: '高一', level: '高中', description: '高中一年级必备词汇', wordCount: 800 },
  { id: 'senior2', name: '高二', level: '高中', description: '高中二年级必备词汇', wordCount: 900 },
  { id: 'senior3', name: '高三', level: '高中', description: '高中三年级必备词汇', wordCount: 1000 },
  { id: 'cet4', name: '大学英语四级', level: '四六级', description: 'CET-4 必备词汇', wordCount: 2500 },
  { id: 'cet6', name: '大学英语六级', level: '四六级', description: 'CET-6 必备词汇', wordCount: 2500 },
  { id: 'ielts', name: '雅思', level: '留学', description: '雅思必备词汇', wordCount: 5000 },
  { id: 'toefl', name: '托福', level: '留学', description: '托福必备词汇', wordCount: 8000 },
];

const LEVEL_COLORS: Record<string, string> = {
  '小学': '#22c55e',
  '初中': '#3b82f6',
  '高中': '#f59e0b',
  '四六级': '#8b5cf6',
  '留学': '#ec4899',
};

const WordBookDownload: Component = () => {
  const [selectedLevel, setSelectedLevel] = createSignal<string | null>(null);
  const [downloading, setDownloading] = createSignal<string | null>(null);
  const [message, setMessage] = createSignal<{ type: 'success' | 'error'; text: string } | null>(null);

  const levels = [...new Set(WORD_BOOKS.map(b => b.level))];

  const filteredBooks = () => {
    if (!selectedLevel()) return WORD_BOOKS;
    return WORD_BOOKS.filter(b => b.level === selectedLevel());
  };

  const handleDownload = async (book: WordBook) => {
    setDownloading(book.id);
    setMessage(null);
    try {
      // 调用后端命令下载单词本
      await invoke('download_word_book', { bookId: book.id, bookName: book.name });
      setMessage({ type: 'success', text: `${book.name} 下载成功！` });
    } catch (e) {
      setMessage({ type: 'error', text: `下载失败: ${e}` });
    } finally {
      setDownloading(null);
    }
  };

  return (
    <div class="wordbook-download">
      <h3>📚 下载词库</h3>
      <p class="description">选择适合的词库开始学习</p>

      <div class="level-filters">
        <button
          class={`level-btn ${selectedLevel() === null ? 'active' : ''}`}
          onClick={() => setSelectedLevel(null)}
        >
          全部
        </button>
        <For each={levels}>
          {(level) => (
            <button
              class={`level-btn ${selectedLevel() === level ? 'active' : ''}`}
              style={{ '--level-color': LEVEL_COLORS[level] }}
              onClick={() => setSelectedLevel(level)}
            >
              {level}
            </button>
          )}
        </For>
      </div>

      <div class="wordbook-grid">
        <For each={filteredBooks()}>
          {(book) => (
            <div class="wordbook-card">
              <div class="wordbook-header">
                <span class="wordbook-level" style={{ background: LEVEL_COLORS[book.level] }}>
                  {book.level}
                </span>
                <span class="wordbook-count">{book.wordCount}词</span>
              </div>
              <h4 class="wordbook-name">{book.name}</h4>
              <p class="wordbook-desc">{book.description}</p>
              <button
                class={`btn-download ${downloading() === book.id ? 'loading' : ''}`}
                onClick={() => handleDownload(book)}
                disabled={downloading() !== null}
              >
                {downloading() === book.id ? '下载中...' : '下载'}
              </button>
            </div>
          )}
        </For>
      </div>

      {message() && (
        <div class={`message ${message()!.type}`}>
          {message()!.text}
        </div>
      )}
    </div>
  );
};

export default WordBookDownload;
