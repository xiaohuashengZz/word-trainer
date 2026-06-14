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
  const [message, setMessage] = createSignal<string | null>(null);
  const [reminderEnabled, setReminderEnabled] = createSignal(true);

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
      console.error('Failed to load settings:', e);
    }
  });

  // 切换提醒开关
  const handleToggleReminder = async () => {
    const newValue = !reminderEnabled();
    setReminderEnabled(newValue);
    try {
      await setSetting('reminder_enabled', newValue.toString());
      setMessage(newValue ? '复习提醒已开启' : '复习提醒已关闭');
      setTimeout(() => setMessage(null), 2000);
    } catch (e) {
      setReminderEnabled(!newValue); // 回滚
      setMessage('保存失败');
    }
  };

  // 保存间隔设置
  const handleIntervalChange = async (minutes: number) => {
    setIsSaving(true);
    setMessage(null);
    try {
      $reviewInterval.set(minutes);
      await setSetting('review_interval', minutes.toString());
      setMessage('设置已保存');
      setTimeout(() => setMessage(null), 2000);
    } catch (e) {
      setMessage('保存失败');
    } finally {
      setIsSaving(false);
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

        <div class="current-interval">
          当前设置：每 <strong>{reviewInterval()}</strong> 分钟弹出一次复习提醒
        </div>

        {message() && (
          <div class={`message ${message().includes('失败') ? 'error' : 'success'}`}>
            {message()}
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
