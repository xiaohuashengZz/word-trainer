import { Component, createSignal, Show } from 'solid-js';
import { addWord } from '../stores/api';
import type { Definition } from '../types';

const AddWordForm: Component = () => {
  const [wordText, setWordText] = createSignal('');
  const [definitionText, setDefinitionText] = createSignal('');
  const [pos, setPos] = createSignal('n.');
  const [isLoading, setIsLoading] = createSignal(false);
  const [message, setMessage] = createSignal<{ type: 'success' | 'error'; text: string } | null>(null);

  const handleSubmit = async (e: Event) => {
    e.preventDefault();

    if (!wordText().trim() || !definitionText().trim()) {
      setMessage({ type: 'error', text: '请填写单词和释义' });
      return;
    }

    setIsLoading(true);
    setMessage(null);

    try {
      const definitions: Definition[] = [
        {
          id: crypto.randomUUID(),
          pos: pos(),
          definition: definitionText().trim(),
        },
      ];

      await addWord(wordText().trim(), definitions);
      setMessage({ type: 'success', text: '单词添加成功！' });
      setWordText('');
      setDefinitionText('');
    } catch (e) {
      setMessage({ type: 'error', text: String(e) });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div class="add-word-form">
      <h3>添加单词</h3>

      <Show when={message()}>
        <div class={`message ${message()?.type}`}>{message()?.text}</div>
      </Show>

      <form onSubmit={handleSubmit}>
        <div class="form-group">
          <label for="word">单词</label>
          <input
            id="word"
            type="text"
            value={wordText()}
            onInput={(e) => setWordText(e.currentTarget.value)}
            placeholder="输入英语单词"
          />
        </div>

        <div class="form-group">
          <label for="pos">词性</label>
          <select id="pos" value={pos()} onChange={(e) => setPos(e.currentTarget.value)}>
            <option value="n.">名词 n.</option>
            <option value="v.">动词 v.</option>
            <option value="adj.">形容词 adj.</option>
            <option value="adv.">副词 adv.</option>
            <option value="prep.">介词 prep.</option>
            <option value="conj.">连词 conj.</option>
            <option value="pron.">代词 pron.</option>
            <option value="num.">数词 num.</option>
          </select>
        </div>

        <div class="form-group">
          <label for="definition">释义</label>
          <input
            id="definition"
            type="text"
            value={definitionText()}
            onInput={(e) => setDefinitionText(e.currentTarget.value)}
            placeholder="输入中文释义"
          />
        </div>

        <button type="submit" class="btn btn-primary" disabled={isLoading()}>
          {isLoading() ? '添加中...' : '添加单词'}
        </button>
      </form>
    </div>
  );
};

export default AddWordForm;