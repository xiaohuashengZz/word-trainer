import { Component, createSignal, onMount, For } from 'solid-js';
import { useStore } from '@nanostores/solid';
import { $reviewInterval } from '../stores/appStore';
import { getSetting, setSetting } from '../stores/api';

// 预设的时间间隔选项（分钟）
const INTERVAL_OPTIONS = [
  { value: 5, label: '5 分钟' },
  { value: 10, label: '10 分钟' },
  { value: 15, label: '15 分钟' },
  { value: 30, label: '30 分钟' },
  { value: 60, label: '1 小时' },
  { value: 120, label: '2 小时' },
  { value: 180, label: '3 小时' },
];

const Settings: Component = () => {
  const reviewInterval = useStore($reviewInterval);
  const [isSaving, setIsSaving] = createSignal(false);
  const [message, setMessage] = createSignal<{ type: 'success' | 'error'; text: string } | null>(null);
  const [reminderEnabled, setReminderEnabled] = createSignal(true);
  const [customInterval, setCustomInterval] = createSignal('');

  // 加载设置
  onMount(async () => {
    try {
      const savedInterval = await getSetting('review_interval');
      if (savedInterval) {
        $reviewInterval.set(parseInt(savedInterval, 10));
      }
      const savedEnabled = await getSetting('reminder_enabled');
      if (savedEnabled !== null) {
        setReminderEnabled(savedEnabled === 'true');
      }
    } catch (e) {
      console.error('加载设置失败:', e);
    }
  });

  // 切换提醒开关
  const handleToggleReminder = async () => {
    const newValue = !reminderEnabled();
    setReminderEnabled(newValue);
    try {
      await setSetting('reminder_enabled', newValue.toString());
      setMessage({ type: 'success', text: newValue ? '复习提醒已开启' : '复习提醒已关闭' });
      setTimeout(() => setMessage(null), 2000);
    } catch (e) {
      setReminderEnabled(!newValue);
      setMessage({ type: 'error', text: '保存失败' });
    }
  };

  // 保存间隔设置
  const handleIntervalChange = async (minutes: number) => {
    if (minutes < 1 || minutes > 1440) {
      setMessage({ type: 'error', text: '请输入 1-1440 之间的数字' });
      return;
    }
    setIsSaving(true);
    setMessage(null);
    try {
      $reviewInterval.set(minutes);
      await setSetting('review_interval', minutes.toString());
      setMessage({ type: 'success', text: `已设置为 ${minutes} 分钟` });
      setTimeout(() => setMessage(null), 2000);
    } catch (e) {
      setMessage({ type: 'error', text: '保存失败' });
    } finally {
      setIsSaving(false);
    }
  };

  // 处理自定义输入
  const handleCustomInput = (e: Event) => {
    const value = (e.target as HTMLInputElement).value.replace(/[^\d]/g, '');
    setCustomInterval(value);
  };

  // 提交自定义输入
  const handleCustomSubmit = () => {
    const value = parseInt(customInterval(), 10);
    if (!isNaN(value) && value > 0) {
      handleIntervalChange(value);
      setCustomInterval('');
    }
  };

  return (
    <div class="settings-container">
      <h2>⚙️ 设置</h2>

      <div class="settings-section">
        <h3>🔔 复习提醒</h3>
        <p class="settings-description">
          开启后，应用会定时弹出复习提醒窗口
        </p>

        <div class="toggle-row">
          <span class="toggle-label">启用复习提醒</span>
          <button
            class={`toggle-btn ${reminderEnabled() ? 'active' : ''}`}
            onClick={handleToggleReminder}
          >
            <span class="toggle-slider" />
          </button>
        </div>
      </div>

      <div class="settings-section">
        <h3>📝 复习提醒间隔</h3>
        <p class="settings-description">
          设置每隔多长时间弹出一次复习提醒弹窗
        </p>

        <div class="interval-options">
          <For each={INTERVAL_OPTIONS}>
            {(option) => (
              <button
                class={`interval-btn ${reviewInterval() === option.value ? 'active' : ''}`}
                onClick={() => handleIntervalChange(option.value)}
                disabled={isSaving()}
              >
                {option.label}
              </button>
            )}
          </For>
        </div>

        <div class="custom-interval-input">
          <input
            type="text"
            placeholder="自定义（1-1440分钟）"
            value={customInterval()}
            onInput={handleCustomInput}
            onKeyPress={(e) => e.key === 'Enter' && handleCustomSubmit()}
          />
          <button
            class="btn-primary"
            onClick={handleCustomSubmit}
            disabled={!customInterval() || isSaving()}
          >
            确定
          </button>
        </div>

        <div class="current-interval">
          当前设置：每 <strong>{reviewInterval()}</strong> 分钟弹出一次复习提醒
        </div>

        {message() && (
          <div class={`message ${message()!.type}`}>
            {message()!.text}
          </div>
        )}
      </div>

      <div class="settings-section">
        <h3>💡 使用说明</h3>
        <ul class="settings-tips">
          <li>设置复习提醒间隔后，应用会定时弹出复习窗口</li>
          <li>弹窗会在后台自动打开，让你随时随地复习单词</li>
          <li>建议将间隔设置在 15-60 分钟之间，复习效果最佳</li>
        </ul>
      </div>
    </div>
  );
};

export default Settings;
