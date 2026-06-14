import { Component } from 'solid-js';
import { getCurrentWindow } from '@tauri-apps/api/window';

const TitleBar: Component = () => {
  const appWindow = getCurrentWindow();

  const handleMinimize = () => appWindow.minimize();
  const handleMaximize = async () => {
    if (await appWindow.isMaximized()) {
      appWindow.unmaximize();
    } else {
      appWindow.maximize();
    }
  };
  const handleClose = () => appWindow.close();

  return (
    <div class="title-bar" data-tauri-drag-region>
      <div class="title-bar-title" data-tauri-drag-region>
        📚 单词学习助手
      </div>
      <div class="title-bar-buttons">
        <button class="title-btn minimize" onClick={handleMinimize} title="最小化">
          ─
        </button>
        <button class="title-btn maximize" onClick={handleMaximize} title="最大化">
          □
        </button>
        <button class="title-btn close" onClick={handleClose} title="关闭">
          ✕
        </button>
      </div>
    </div>
  );
};

export default TitleBar;
