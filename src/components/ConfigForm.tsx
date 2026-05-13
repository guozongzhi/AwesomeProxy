import type { AppConfig, ProviderConfig } from '../types';

type ConfigFormProps = {
  config: AppConfig;
  saving: boolean;
  onChange: (config: AppConfig) => void;
  onSubmit: () => void;
};

export function ConfigForm({ config, saving, onChange, onSubmit }: ConfigFormProps) {
  const provider = config.providers[0] ?? {
    name: '',
    api_key: '',
    base_url: '',
    model: '',
  };

  const updateProvider = (patch: Partial<ProviderConfig>) => {
    const nextProvider = { ...provider, ...patch };
    onChange({
      ...config,
      providers: [nextProvider, ...config.providers.slice(1)],
      routes: {
        ...config.routes,
        default_provider: patch.name ?? config.routes.default_provider,
        default_model: patch.model ?? config.routes.default_model,
      },
    });
  };

  return (
    <form
      className="config-card"
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit();
      }}
    >
      <section className="form-section">
        <div>
          <p className="eyebrow">Proxy</p>
          <h2>Local LiteLLM Gateway</h2>
          <p className="section-copy">AwesomeProxy writes this configuration to ~/.awesomeproxy/config.yaml.</p>
        </div>
        <div className="field-grid two-columns">
          <label>
            Host
            <input
              value={config.proxy.host}
              onChange={(event) =>
                onChange({ ...config, proxy: { ...config.proxy, host: event.target.value } })
              }
              placeholder="127.0.0.1"
            />
          </label>
          <label>
            Port
            <input
              type="number"
              min="1"
              max="65535"
              value={config.proxy.port}
              onChange={(event) =>
                onChange({
                  ...config,
                  proxy: { ...config.proxy, port: Number(event.target.value) || 4000 },
                })
              }
            />
          </label>
        </div>
      </section>

      <section className="form-section">
        <div>
          <p className="eyebrow">Provider</p>
          <h2>Default Model Route</h2>
          <p className="section-copy">Edit the first provider route. More dynamic providers can be added later.</p>
        </div>
        <div className="field-grid">
          <label>
            Provider Name
            <input value={provider.name} onChange={(event) => updateProvider({ name: event.target.value })} />
          </label>
          <label>
            API Key
            <input
              type="password"
              value={provider.api_key}
              onChange={(event) => updateProvider({ api_key: event.target.value })}
              placeholder="sk-..."
            />
          </label>
          <label>
            Base URL
            <input
              value={provider.base_url}
              onChange={(event) => updateProvider({ base_url: event.target.value })}
              placeholder="https://api.deepseek.com"
            />
          </label>
          <label>
            Model
            <input
              value={provider.model}
              onChange={(event) => updateProvider({ model: event.target.value })}
              placeholder="deepseek-chat"
            />
          </label>
        </div>
      </section>

      <button className="primary-button" type="submit" disabled={saving}>
        {saving ? 'Saving…' : 'Save YAML & Reload Sidecar'}
      </button>
    </form>
  );
}
