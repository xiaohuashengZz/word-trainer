import { Component, createSignal, Show } from 'solid-js';
import { useStore } from '@nanostores/solid';
import { $theme, toggleTheme } from './stores/appStore';
import ReviewPopup from './components/ReviewPopup';
import Statistics from './components/AddWord';
import AddWordForm from './components/AddWordForm';
import WordList from './components/WordList';
import Settings from './components/Settings';
import './App.css';

type Tab = 'review' | 'words' | 'add' | 'stats' | 'settings';

const App: Component = () => {
  const theme = useStore($theme);
  const [activeTab, setActiveTab] = createSignal<Tab>('review');

  return (
    <div class={`app ${theme()}`}>
      <header class="app-header">
        <h1>📚 单词学习助手</h1>
        <div class="header-actions">
          <button class="settings-btn" onClick={() => setActiveTab('settings')} title="设置">
            ⚙️
          </button>
          <button class="theme-btn" onClick={toggleTheme} title="切换主题">
            {theme() === 'light' ? '🌙' : '☀️'}
          </button>
        </div>
      </header>

      <nav class="tab-nav">
        <button
          class={`tab-btn ${activeTab() === 'review' ? 'active' : ''}`}
          onClick={() => setActiveTab('review')}
        >
          复习
        </button>
        <button
          class={`tab-btn ${activeTab() === 'words' ? 'active' : ''}`}
          onClick={() => setActiveTab('words')}
        >
          单词
        </button>
        <button
          class={`tab-btn ${activeTab() === 'add' ? 'active' : ''}`}
          onClick={() => setActiveTab('add')}
        >
          添加
        </button>
        <button
          class={`tab-btn ${activeTab() === 'stats' ? 'active' : ''}`}
          onClick={() => setActiveTab('stats')}
        >
          统计
        </button>
      </nav>

      <main class="app-content">
        <Show when={activeTab() === 'review'}>
          <ReviewPopup />
        </Show>
        <Show when={activeTab() === 'words'}>
          <WordList />
        </Show>
        <Show when={activeTab() === 'add'}>
          <AddWordForm />
        </Show>
        <Show when={activeTab() === 'stats'}>
          <Statistics />
        </Show>
        <Show when={activeTab() === 'settings'}>
          <Settings />
        </Show>
      </main>
    </div>
  );
};

export default App;