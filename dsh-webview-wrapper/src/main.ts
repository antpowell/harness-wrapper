import { invoke } from '@tauri-apps/api/core';
import { open as openPath } from '@tauri-apps/plugin-shell';

const loadingStatus = document.getElementById('loadingStatus') as HTMLDivElement;
const errorMessage = document.getElementById('errorMessage') as HTMLDivElement;
const configPanel = document.getElementById('configPanel') as HTMLDivElement;
const endpointInput = document.getElementById('endpointInput') as HTMLInputElement;
const connectButton = document.getElementById('connectButton') as HTMLButtonElement;
const openLogButton = document.getElementById('openLogButton') as HTMLButtonElement;

function showStatus(message: string) {
  loadingStatus.textContent = message;
  errorMessage.style.display = 'none';
}

function showConfigPanel(message?: string) {
  configPanel.classList.add('active');
  if (message) {
    showError(message);
  }
  endpointInput.focus();
}

function showError(message: string) {
  errorMessage.textContent = message;
  errorMessage.style.display = 'block';
  openLogButton.style.display = 'block';
  loadingStatus.textContent = 'The harness could not be opened automatically.';
}

async function openEndpoint(url: string) {
  await invoke('set_auth_url', { url });
  await invoke('open_endpoint', { url });
}

async function init() {
  showStatus('Starting DSH and waiting for its authenticated URL...');

  try {
    const url = await invoke<string>('start_dsh_and_get_url');
    showStatus('Authenticated endpoint found. Loading DSH...');
    await openEndpoint(url);
  } catch (error) {
    console.error('Failed to start DSH automatically', error);
    showConfigPanel(String(error));
  }
}

async function openHarnessLog() {
  openLogButton.disabled = true;
  try {
    const path = await invoke<string>('get_harness_log_path');
    await openPath(path);
  } catch (error) {
    errorMessage.textContent = `Could not open the harness log: ${String(error)}`;
  } finally {
    openLogButton.disabled = false;
  }
}

async function connectToEndpoint() {
  const url = endpointInput.value.trim();
  if (!url) {
    showError('Enter the complete harness URL, including its token.');
    return;
  }

  connectButton.disabled = true;
  showStatus('Checking the harness endpoint...');

  try {
    const available = await invoke<boolean>('check_endpoint', { url });
    if (!available) {
      throw new Error('The endpoint did not respond successfully. Check the URL and token.');
    }
    await openEndpoint(url);
  } catch (error) {
    showError(String(error));
  } finally {
    connectButton.disabled = false;
  }
}

connectButton.addEventListener('click', connectToEndpoint);
openLogButton.addEventListener('click', () => {
  void openHarnessLog();
});
endpointInput.addEventListener('keydown', (event) => {
  if (event.key === 'Enter') {
    void connectToEndpoint();
  }
});

document.addEventListener('DOMContentLoaded', () => {
  void init();
});
