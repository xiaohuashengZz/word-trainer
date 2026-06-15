import { Component, createSignal, Show } from 'solid-js';
import { useStore } from '@nanostores/solid';
import { $theme, toggleTheme } from './stores/appStore';
import ReviewPopup from './components/ReviewPopup';
import Statistics from './components/AddWord';
import AddWordForm from './components/AddWordForm';
import WordList from './components/WordList';
import Settings from './components/Settings';
import TitleBar from './components/TitleBar';
import WordBookDownload from './components/WordBookDownload';
import WordSearchDownload from './components/WordSearchDownload';
import './App.css';

type Tab = 'review' | 'words' | 'add' | 'stats' | 'settings' | 'download';

const App: Component = () => {
  const theme = useStore($theme);
  const [activeTab, setActiveTab] = createSignal<Tab>('review');

  return (
    <div class={`app ${theme()}`}>
      <TitleBar />

      <nav class="tab-nav">
        <button class={`tab-btn ${activeTab() === 'review' ? 'active' : ''}`} onClick={() => setActiveTab('review')}>复习</button>
        <button class={`tab-btn ${activeTab() === 'words' ? 'active' : ''}`} onClick={() => setActiveTab('words')}>单词</button>
        <button class={`tab-btn ${activeTab() === 'add' ? 'active' : ''}`} onClick={() => setActiveTab('add')}>添加</button>
        <button class={`tab-btn ${activeTab() === 'stats' ? 'active' : ''}`} onClick={() => setActiveTab('stats')}>统计</button>
        <button class={`tab-btn ${activeTab() === 'download' ? 'active' : ''}`} onClick={() => setActiveTab('download')}>词库</button>
        <button class={`tab-btn ${activeTab() === 'settings' ? 'active' : ''}`} onClick={() => setActiveTab('settings')}>⚙️</button>
      </nav>

      <main class="app-content">
        <Show when={activeTab() === 'review'}><ReviewPopup /></Show>
        <Show when={activeTab() === 'words'}><WordList /></Show>
        <Show when={activeTab() === 'add'}><AddWordForm /></Show>
        <Show when={activeTab() === 'stats'}><Statistics /></Show>
        <Show when={activeTab() === 'download'}>
          <WordSearchDownload />
          <WordBookDownload />
        </Show>
        <Show when={activeTab() === 'settings'}><Settings /></Show>
      </main>
    </div>
  );
};

export default App;
