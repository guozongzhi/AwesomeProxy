export type AppConfig = {
  app_settings: {
    host: string;
    port: number;
  };
  model_list: ModelConfig[];
};

export type ModelConfig = {
  model_name: string;
  litellm_params: LiteLlmParams;
};

export type LiteLlmParams = {
  model: string;
  api_key: string;
  api_base: string;
};

export type SaveConfigResponse = {
  config_path: string;
  reloaded: boolean;
  message: string;
};

export type SidecarStatus = {
  running: boolean;
  pid?: number | null;
  message: string;
};
