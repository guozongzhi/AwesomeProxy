export type AppConfig = {
  proxy: {
    host: string;
    port: number;
  };
  providers: ProviderConfig[];
  routes: {
    default_provider: string;
    default_model: string;
  };
};

export type ProviderConfig = {
  name: string;
  api_key: string;
  base_url: string;
  model: string;
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
