import { Component, createSignal, For, Show } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';

interface SearchResult {
  word: string;
  phonetic?: string;
  translation?: string;
  definition?: string;
  selected: boolean;
}

const WordSearchDownload: Component = () => {
  const [searchText, setSearchText] = createSignal('');
  const [searchResults, setSearchResults] = createSignal<SearchResult[]>([]);
  const [isSearching, setIsSearching] = createSignal(false);
  const [isDownloading, setIsDownloading] = createSignal(false);
  const [message, setMessage] = createSignal<{ type: 'success' | 'error'; text: string } | null>(null);
  const [downloadedWords, setDownloadedWords] = createSignal<string[]>([]);

  // 搜索单词
  const handleSearch = async () => {
    const text = searchText().trim();
    if (!text) return;

    setIsSearching(true);
    setMessage(null);
    setDownloadedWords([]);

    try {
      // 支持批量搜索，每行一个单词
      const words = text.split(/[\n,，]/).map(w => w.trim()).filter(w => w.length > 0);

      // 对每个单词进行API查询
      const results: SearchResult[] = [];
      for (const word of words) {
        try {
          const response = await invoke<any>('download_word_from_api', { wordText: word });
          results.push({
            word: response.word,
            phonetic: response.phonetic,
            definition: response.definitions?.[0]?.definition || '',
            selected: false,
          });
        } catch (e) {
          // 如果API未配置或出错，添加基本信息
          results.push({
            word: word,
            phonetic: undefined,
            definition: '',
            selected: false,
          });
        }
      }

      setSearchResults(results);
      if (results.length > 0) {
        setMessage({ type: 'success', text: `找到 ${results.length} 个单词` });
      }
    } catch (e) {
      setMessage({ type: 'error', text: `搜索失败: ${e}` });
    } finally {
      setIsSearching(false);
    }
  };

  // 切换选择状态
  const toggleSelect = (index: number) => {
    const results = [...searchResults()];
    results[index].selected = !results[index].selected;
    setSearchResults(results);
  };

  // 全选/取消全选
  const toggleSelectAll = () => {
    const allSelected = searchResults().every(r => r.selected);
    setSearchResults(searchResults().map(r => ({ ...r, selected: !allSelected })));
  };

  // 下载选中的单词
  const handleDownloadSelected = async () => {
    const selected = searchResults().filter(r => r.selected);
    if (selected.length === 0) {
      setMessage({ type: 'error', text: '请先选择要下载的单词' });
      return;
    }

    setIsDownloading(true);
    setMessage(null);

    try {
      const words = selected.map(r => r.word);
      const result = await invoke<any>('download_words_from_api', { words });

      const downloaded = selected
        .filter((_, i) => i < result.success)
        .map(r => r.word);
      setDownloadedWords(downloaded);

      let msg = `成功下载 ${result.success} 个单词`;
      if (result.skipped > 0) msg += `，跳过 ${result.skipped} 个（已存在）`;
      if (result.failed > 0) msg += `，失败 ${result.failed} 个`;

      setMessage({
        type: result.failed > 0 ? 'error' : 'success',
        text: msg
      });

      // 清除已成功下载的
      setSearchResults(searchResults().filter(r => !downloaded.includes(r.word)));
    } catch (e) {
      setMessage({ type: 'error', text: `下载失败: ${e}` });
    } finally {
      setIsDownloading(false);
    }
  };

  // 下载单个单词
  const handleDownloadSingle = async (word: string) => {
    setIsDownloading(true);
    try {
      await invoke('download_words_from_api', { words: [word] });
      setDownloadedWords([...downloadedWords(), word]);
      setMessage({ type: 'success', text: `"${word}" 下载成功` });
      // 从列表中移除
      setSearchResults(searchResults().filter(r => r.word !== word));
    } catch (e) {
      setMessage({ type: 'error', text: `下载失败: ${e}` });
    } finally {
      setIsDownloading(false);
    }
  };

  // 清除结果
  const handleClear = () => {
    setSearchResults([]);
    setSearchText('');
    setMessage(null);
    setDownloadedWords([]);
  };

  const selectedCount = () => searchResults().filter(r => r.selected).length;

  return (
    <div class="word-search-download">
      <h3>🔍 搜索下载单词</h3>
      <p class="description">
        输入单词（每行一个或用逗号分隔），从有道API获取释义并下载
      </p>

      <div class="search-box">
        <textarea
          class="search-input"
          placeholder="输入单词，例如：hello, world, apple&#10;或者每行一个单词"
          value={searchText()}
          onInput={(e) => setSearchText(e.currentTarget.value)}
          rows={4}
        />
        <div class="search-actions">
          <button
            class="btn-secondary"
            onClick={handleClear}
            disabled={searchResults().length === 0}
          >
            清除
          </button>
          <button
            class="btn-primary"
            onClick={handleSearch}
            disabled={isSearching() || !searchText().trim()}
          >
            {isSearching() ? '搜索中...' : '搜索'}
          </button>
        </div>
      </div>

      <Show when={searchResults().length > 0}>
        <div class="results-header">
          <div class="results-count">
            找到 {searchResults().length} 个单词
            <Show when={downloadedWords().length > 0}>
              <span class="downloaded-count">
                （已下载 {downloadedWords().length} 个）
              </span>
            </Show>
          </div>
          <div class="results-actions">
            <button class="btn-small" onClick={toggleSelectAll}>
              {searchResults().every(r => r.selected) ? '取消全选' : '全选'}
            </button>
            <button
              class="btn-primary"
              onClick={handleDownloadSelected}
              disabled={selectedCount() === 0 || isDownloading()}
            >
              {isDownloading() ? '下载中...' : `下载选中 (${selectedCount()})`}
            </button>
          </div>
        </div>

        <div class="results-list">
          <For each={searchResults()}>
            {(result, index) => (
              <div class={`result-item ${result.selected ? 'selected' : ''}`}>
                <label class="checkbox-label">
                  <input
                    type="checkbox"
                    checked={result.selected}
                    onChange={() => toggleSelect(index())}
                  />
                  <span class="word-text">{result.word}</span>
                  <Show when={result.phonetic}>
                    <span class="word-phonetic">{result.phonetic}</span>
                  </Show>
                </label>
                <Show when={result.definition}>
                  <div class="word-definition">{result.definition}</div>
                </Show>
                <button
                  class="btn-download-single"
                  onClick={() => handleDownloadSingle(result.word)}
                  disabled={isDownloading()}
                >
                  下载
                </button>
              </div>
            )}
          </For>
        </div>
      </Show>

      <Show when={searchResults().length === 0 && searchText().trim() && !isSearching()}>
        <div class="empty-results">
          <p>没有找到匹配的单词</p>
          <p class="hint">请检查输入是否正确，或确保已配置有道API</p>
        </div>
      </Show>

      {message() && (
        <div class={`message ${message()!.type}`}>
          {message()!.text}
        </div>
      )}
    </div>
  );
};

export default WordSearchDownload;