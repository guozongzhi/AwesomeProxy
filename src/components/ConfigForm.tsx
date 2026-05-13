import type { AppConfig, ModelConfig } from '../types';

type ConfigFormProps = {
  config: AppConfig;
  saving: boolean;
  onChange: (config: AppConfig) => void;
  onSubmit: () => void;
};

const emptyModel: ModelConfig = {
  model_name: '',
  litellm_params: {
    model: '',
    api_key: '',
    api_base: '',
  },
};

export function ConfigForm({ config, saving, onChange, onSubmit }: ConfigFormProps) {
  const model = config.model_list[0] ?? emptyModel;

  const updateModel = (patch: Partial<ModelConfig>) => {
    const nextModel = { ...model, ...patch };
    onChange({
      ...config,
      model_list: [nextModel, ...config.model_list.slice(1)],
    });
  };

  const updateLiteLlmParams = (patch: Partial<ModelConfig['litellm_params']>) => {
    updateModel({
      litellm_params: {
        ...model.litellm_params,
        ...patch,
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
          <p className="section-copy">
            AwesomeProxy stores app settings and LiteLLM model routes in ~/.awesomeproxy/config.yaml.
          </p>
        </div>
        <div className="field-grid two-columns">
          <label>
            Host
            <input
              value={config.app_settings.host}
              onChange={(event) =>
                onChange({
                  ...config,
                  app_settings: { ...config.app_settings, host: event.target.value },
                })
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
              value={config.app_settings.port}
              onChange={(event) =>
                onChange({
                  ...config,
                  app_settings: { ...config.app_settings, port: Number(event.target.value) || 4000 },
                })
              }
            />
          </label>
        </div>
      </section>

      <section className="form-section">
        <div>
          <p className="eyebrow">LiteLLM</p>
          <h2>Default Model Route</h2>
          <p className="section-copy">
            This writes the LiteLLM-compatible model_list entry used by the sidecar.
          </p>
        </div>
        <div className="field-grid">
          <label>
            Public Model Name
            <input value={model.model_name} onChange={(event) => updateModel({ model_name: event.target.value })} />
          </label>
          <label>
            Provider Model
            <input
              value={model.litellm_params.model}
              onChange={(event) => updateLiteLlmParams({ model: event.target.value })}
              placeholder="deepseek/deepseek-chat"
            />
          </label>
          <label>
            API Key
            <input
              type="password"
              value={model.litellm_params.api_key}
              onChange={(event) => updateLiteLlmParams({ api_key: event.target.value })}
              placeholder="sk-..."
            />
          </label>
          <label>
            API Base URL
            <input
              value={model.litellm_params.api_base}
              onChange={(event) => updateLiteLlmParams({ api_base: event.target.value })}
              placeholder="https://api.deepseek.com"
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
