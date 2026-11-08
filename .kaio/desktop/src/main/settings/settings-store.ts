// src/main/settings/settings-store.ts
import { app } from "electron";
import { promises as fs } from "node:fs";
import path from "node:path";

export interface PersistedProvider {
  id: string;
  name: string;
  baseUrl?: string;
  defaultModelId?: string;
  apiKeyEnc?: string;
  builtin?: boolean;
  updatedAt: number;
}

export interface PersistedSettings {
  version: 1;
  providers: Record<string, PersistedProvider>;
}

class SettingsStore {
  private data: PersistedSettings | null = null;

  private filePath: string | null = null;

  private loading: Promise<void> | null = null;

  async ensureLoaded(): Promise<void> {
    if (this.data) {
      return;
    }

    if (!this.loading) {
      this.loading = this.load();
    }

    await this.loading;
  }

  async getProviders(): Promise<PersistedProvider[]> {
    await this.ensureLoaded();

    return Object.values(this.data!.providers).sort((a, b) =>
      a.id.localeCompare(b.id)
    );
  }

  async getProvider(
    id: string
  ): Promise<PersistedProvider | undefined> {
    await this.ensureLoaded();

    return this.data!.providers[id];
  }

  async upsertProvider(provider: PersistedProvider): Promise<void> {
    await this.ensureLoaded();

    this.data!.providers[provider.id] = provider;

    await this.save();
  }

  async removeProvider(id: string): Promise<void> {
    await this.ensureLoaded();

    const provider = this.data!.providers[id];

    if (!provider) {
      return;
    }

    if (provider.builtin) {
      throw new Error("BUILTIN_PROVIDER_CANNOT_BE_REMOVED");
    }

    delete this.data!.providers[id];

    await this.save();
  }

  private async load(): Promise<void> {
    this.filePath = path.join(
      app.getPath("userData"),
      "kaioken-settings.json"
    );

    try {
      const raw = await fs.readFile(this.filePath, "utf8");
      const parsed = JSON.parse(raw) as PersistedSettings;

      if (
        parsed &&
        parsed.version === 1 &&
        typeof parsed.providers === "object" &&
        parsed.providers !== null
      ) {
        this.data = parsed;
      } else {
        this.data = this.defaults();
      }
    } catch {
      this.data = this.defaults();
    }

    await this.save();
  }

  private async save(): Promise<void> {
    if (!this.filePath || !this.data) {
      return;
    }

    await fs.mkdir(path.dirname(this.filePath), {
      recursive: true
    });

    const tmpPath = `${this.filePath}.tmp`;

    await fs.writeFile(
      tmpPath,
      JSON.stringify(this.data, null, 2),
      "utf8"
    );

    await fs.rename(tmpPath, this.filePath);
  }

  private defaults(): PersistedSettings {
    const now = Date.now();

    return {
      version: 1,
      providers: {
        mock: {
          id: "mock",
          name: "Mock Provider",
          defaultModelId: "mock-stream",
          builtin: true,
          updatedAt: now
        },
        openai: {
          id: "openai",
          name: "OpenAI",
          baseUrl: "https://api.openai.com/v1",
          defaultModelId: "gpt-4.1",
          builtin: true,
          updatedAt: now
        },
        anthropic: {
          id: "anthropic",
          name: "Anthropic",
          baseUrl: "https://api.anthropic.com",
          defaultModelId: "claude-sonnet-4-5",
          builtin: true,
          updatedAt: now
        },
        ollama: {
          id: "ollama",
          name: "Ollama",
          baseUrl: "http://localhost:11434",
          defaultModelId: "llama3",
          builtin: true,
          updatedAt: now
        }
      }
    };
  }
}

export const settingsStore = new SettingsStore();
