export type HttpMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS";

export interface Header {
  key: string;
  value: string;
}

export interface SslConfig {
  verifySsl: boolean;
  certPath: string;
  keyPath: string;
  caPath: string;
}

export interface AuthConfig {
  type: "none" | "basic" | "bearer" | "api-key";
  // Basic auth
  username: string;
  password: string;
  // Bearer token
  token: string;
  // API Key
  apiKeyName: string;
  apiKeyValue: string;
  apiKeyIn: "header" | "query";
}

export interface SavedRequest {
  id: string;
  name: string;
  method: HttpMethod;
  url: string;
  headers: Header[];
  body: string;
  sslConfig?: SslConfig;
  authConfig?: AuthConfig;
  createdAt: number;
}

export interface ResponseData {
  status_code: number;
  status_text: string;
  headers: Record<string, string>;
  body: string;
  time_ms: number;
}