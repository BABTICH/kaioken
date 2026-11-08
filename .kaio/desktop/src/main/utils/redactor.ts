// src/main/utils/redactor.ts

const SECRET_PATTERNS: RegExp[] = [
  // OpenAI / Anthropic / generic Bearer tokens
  /\b(?:sk-|ant-)[a-zA-Z0-9]{20,}\b/g,
  /\bBearer\s+[a-zA-Z0-9\-._~+/]+=*/gi,

  // GitHub / GitLab / AWS / GCP
  /\bghp_[a-zA-Z0-9]{36,}\b/g,
  /\bglpat-[a-zA-Z0-9\-]{20,}\b/g,
  /\bAKIA[0-9A-Z]{16}\b/g,
  /\bAIza[a-zA-Z0-9\-_]{35}\b/g,

  // Generic high-entropy hex/base64 strings prefixed with "key" or "secret"
  /(?:api_?key|secret|password|token)\s*[:=]\s*['"]?([a-zA-Z0-9+/=_\-]{24,})['"]?/gi
];

const REDACTED = "[REDACTED]";

export function redactSecrets(input: string): string {
  if (!input) return input;

  let output = input;

  for (const pattern of SECRET_PATTERNS) {
    // Reset regex state for global matches
    pattern.lastIndex = 0;
    output = output.replace(pattern, (match) => {
      // If the match includes a prefix like "key=", keep the prefix
      const eqIndex = match.indexOf("=");
      const colonIndex = match.indexOf(":");

      if (eqIndex > 0) {
        return match.slice(0, eqIndex + 1) + REDACTED;
      }
      if (colonIndex > 0) {
        return match.slice(0, colonIndex + 1) + REDACTED;
      }

      return REDACTED;
    });
  }

  return output;
}
