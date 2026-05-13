import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ConfigForm } from './components/ConfigForm';
import type { AppConfig, SaveConfigResponse, SidecarStatus } from './types';

function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [status, setStatus] = useState<SidecarStatus | null>(null);
  const [message, setMessage] = useState('Loading local YAML configuration…');
  const [saving, setSaving] = useState(false);

  const refreshStatus = async () => {
    const sidecarStatus = await invoke<SidecarStatus>('get_sidecar_status');
    setStatus(sidecarStatus);
  };

  useEffect(() => {
    const load = async () => {
      try {
        const loadedConfig = await invoke<AppConfig>('read_config');
        setConfig(loadedConfig);
        await refreshStatus();
        setMessage('Configuration loaded from ~/.awesomeproxy/config.yaml');
      } catch (error) {
        setMessage(`Failed to load configuration: ${String(error)}`);
      }
    };

    void load();
  }, []);

  const saveConfig = async () => {
    if (!config) {
      return;
    }

    setSaving(true);
    try {
      const response = await invoke<SaveConfigResponse>('save_config', { config });
      await refreshStatus();
      setMessage(`${response.message} (${response.config_path})`);
    } catch (error) {
      setMessage(`Failed to save configuration: ${String(error)}`);
    } finally {
      setSaving(false);
    }
  };

  return (
    <main className="app-shell">
      <section className="hero-panel">
        <div>
          <p className="eyebrow">AwesomeProxy</p>
          <h1>Local-first AI gateway</h1>
          <p className="hero-copy">
            Manage the YAML source of truth and keep the LiteLLM sidecar aligned with Codex and Claude routes.
          </p>
        </div>
        <div className={`status-pill ${status?.running ? 'online' : 'offline'}`}>
          <span />
          {status?.running ? `Sidecar running · PID ${status.pid}` : 'Sidecar offline'}
        </div>
      </section>

      <section className="message-panel">
        <strong>Status</strong>
        <span>{status?.message ?? message}</span>
      </section>

      {config ? (
        <ConfigForm config={config} saving={saving} onChange={setConfig} onSubmit={saveConfig} />
      ) : (
        <section className="config-card skeleton">{message}</section>
      )}
    </main>
  );
}

export default App;
