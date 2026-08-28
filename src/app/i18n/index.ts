import { fallbackMessages, type MessageKey, zhCNMessages } from "./messages";
export type { MessageKey } from "./messages";

export const DEFAULT_LOCALE = "zh-CN" as const;
export const FALLBACK_LOCALE = "en" as const;
export type MessageParams = Record<string, string | number>;
const TOKEN_PATTERN = /\{([A-Za-z][A-Za-z0-9_]*)\}/g;

function interpolate(template: string, params?: MessageParams): string {
  if (!params) return template;
  return template.replace(TOKEN_PATTERN, (token, name: string) => Object.prototype.hasOwnProperty.call(params, name) ? String(params[name]) : token);
}

export function t(key: MessageKey, params?: MessageParams): string {
  const message = zhCNMessages[key] ?? fallbackMessages[key] ?? fallbackMessages["errors.unknown"];
  return interpolate(message, params);
}

export function validateCatalogs(): string[] {
  const errors: string[] = [];
  for (const key of Object.keys(fallbackMessages) as MessageKey[]) {
    const fallbackTokens = [...fallbackMessages[key].matchAll(TOKEN_PATTERN)].map((match) => match[1]).sort();
    const zhTokens = [...zhCNMessages[key].matchAll(TOKEN_PATTERN)].map((match) => match[1]).sort();
    if (fallbackTokens.join("|") !== zhTokens.join("|")) errors.push(`placeholder mismatch: ${key}`);
  }
  return errors;
}
