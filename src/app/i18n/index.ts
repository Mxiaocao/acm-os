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

// Guard application-owned zh-CN strings against the mojibake signatures that
// previously leaked into the active catalog. This intentionally scopes the
// check to our catalog rather than external/user-provided content.
const MOJIBAKE_SIGNATURES = /(?:閲|璐︽|鍚敤|鎾ら攢|涓汉|淇濆瓨|缂栬緫|姝ｅ湪|鏄剧ず|璇疯緭|閲戝竵|宸插厬鎹)/;

export function validateChineseCatalogMojibake(): string[] {
  return (Object.keys(zhCNMessages) as MessageKey[])
    .filter((key) => MOJIBAKE_SIGNATURES.test(zhCNMessages[key]))
    .map((key) => `mojibake: ${key}`);
}
