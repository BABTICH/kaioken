// src/renderer/components/settings/SettingsSurface.tsx
import { useEffect, useState } from "react";
import { useStore } from "@nanostores/react";
import {
  loadProviders,
  removeProviderById,
  saveProviderInput,
  settingsStore
} from "@renderer/stores/settings";
import type { ProviderPublicConfig } from "@shared/types/settings";
import { SecurityPosture } from "./SecurityPosture";
import { UpdateSettings } from "./UpdateSettings";
import { CertificationPanel } from "./CertificationPanel";

interface ProviderFormState {
  id: string;
  name: string;
  baseUrl: string;
  defaultModelId: string;
  apiKey: string;
}

const emptyForm: ProviderFormState = {
  id: "",
  name: "",
  baseUrl: "",
  defaultModelId: "",
  apiKey: ""
};

export function SettingsSurface() {
  const settings = useStore(settingsStore);

  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [form, setForm] = useState<ProviderFormState>(emptyForm);

  const selectedProvider = settings.providers.find(
    (provider) => provider.id === selectedId
  );

  useEffect(() => {
    void loadProviders();
  }, []);

  const selectProvider = (provider: ProviderPublicConfig) => {
    setSelectedId(provider.id);

    setForm({
      id: provider.id,
      name: provider.name,
      baseUrl: provider.baseUrl ?? "",
      defaultModelId: provider.defaultModelId ?? "",
      apiKey: ""
    });
  };

  const newProvider = () => {
    setSelectedId(null);
    setForm(emptyForm);
  };

  const save = async () => {
    await saveProviderInput({
      id: form.id,
      name: form.name || undefined,
      baseUrl: form.baseUrl || undefined,
      defaultModelId: form.defaultModelId || undefined,
      apiKey: form.apiKey || undefined
    });

    setForm((current) => ({
      ...current,
      apiKey: ""
    }));
  };

  const clearSecret = async () => {
    if (!selectedProvider) {
      return;
    }

    await saveProviderInput({
      id: selectedProvider.id,
      apiKey: null
    });
  };

  const removeProvider = async () => {
    if (!selectedProvider) {
      return;
    }

    await removeProviderById(selectedProvider.id);
    newProvider();
  };

  return (
    <div className="hud-scrollbar h-full overflow-auto p-6">
      <div className="mx-auto max-w-5xl">
        <div className="surface-card">
          <div className="head">
            <span className="head-num">07</span>
            <span className="head-title">Provider Vault</span>
            <span className="head-note">PHASE 2.4</span>
          </div>

          <div className="space-y-5 p-5">
            <p className="lead">
              Provider secrets are encrypted in the Electron main process.
              The renderer only receives public metadata and a boolean
              indicating whether a secret exists.
            </p>

            {settings.error && (
              <div className="badge badge--bad">
                {settings.error}
              </div>
            )}

            <div className="grid gap-5 lg:grid-cols-[1.25fr_1fr]">
              <div>
                <div className="label label--accent mb-2">
                  Providers
                </div>

                <div className="tw">
                  <table className="tbl">
                    <thead>
                      <tr>
                        <th>ID</th>
                        <th>Name</th>
                        <th>Base URL</th>
                        <th>Default Model</th>
                        <th>Secret</th>
                      </tr>
                    </thead>

                    <tbody>
                      {settings.providers.map((provider) => {
                        const isSelected =
                          provider.id === selectedId;

                        return (
                          <tr
                            key={provider.id}
                            onClick={() => selectProvider(provider)}
                            style={{
                              cursor: "pointer",
                              background: isSelected
                                ? "var(--bg-raise)"
                                : undefined
                            }}
                          >
                            <td className="mono">{provider.id}</td>
                            <td>{provider.name}</td>
                            <td className="mono">
                              {provider.baseUrl ?? "—"}
                            </td>
                            <td className="mono">
                              {provider.defaultModelId ?? "—"}
                            </td>
                            <td>
                              {provider.hasApiKey ? (
                                <span className="badge badge--good">
                                  secret
                                </span>
                              ) : (
                                <span className="badge">
                                  no secret
                                </span>
                              )}

                              {provider.builtin && (
                                <span
                                  className="badge"
                                  style={{ marginLeft: 6 }}
                                >
                                  builtin
                                </span>
                              )}
                            </td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>

                <div className="mt-3 flex gap-2">
                  <button
                    type="button"
                    className="btn btn--sm"
                    onClick={newProvider}
                  >
                    New provider
                  </button>

                  <button
                    type="button"
                    className="btn btn--sm"
                    onClick={() => {
                      void loadProviders();
                    }}
                  >
                    Reload
                  </button>
                </div>
              </div>

              <div>
                <div className="label label--accent mb-2">
                  Provider Configuration
                </div>

                <div className="space-y-3">
                  <div className="field">
                    <label className="label">Provider ID</label>

                    <input
                      className="input"
                      value={form.id}
                      disabled={Boolean(selectedProvider)}
                      placeholder="openai"
                      onChange={(event) => {
                        setForm((current) => ({
                          ...current,
                          id: event.target.value
                        }));
                      }}
                    />
                  </div>

                  <div className="field">
                    <label className="label">Display Name</label>

                    <input
                      className="input"
                      value={form.name}
                      placeholder="OpenAI"
                      onChange={(event) => {
                        setForm((current) => ({
                          ...current,
                          name: event.target.value
                        }));
                      }}
                    />
                  </div>

                  <div className="field">
                    <label className="label">Base URL</label>

                    <input
                      className="input"
                      value={form.baseUrl}
                      placeholder="https://api.openai.com/v1"
                      onChange={(event) => {
                        setForm((current) => ({
                          ...current,
                          baseUrl: event.target.value
                        }));
                      }}
                    />
                  </div>

                  <div className="field">
                    <label className="label">Default Model</label>

                    <input
                      className="input"
                      value={form.defaultModelId}
                      placeholder="gpt-4.1"
                      onChange={(event) => {
                        setForm((current) => ({
                          ...current,
                          defaultModelId: event.target.value
                        }));
                      }}
                    />
                  </div>

                  <div className="field">
                    <label className="label">API Key</label>

                    <input
                      type="password"
                      className="input"
                      value={form.apiKey}
                      placeholder={
                        selectedProvider?.hasApiKey
                          ? "Stored — enter a new value to replace"
                          : "Enter API key"
                      }
                      onChange={(event) => {
                        setForm((current) => ({
                          ...current,
                          apiKey: event.target.value
                        }));
                      }}
                    />
                  </div>

                  <div className="flex flex-wrap gap-2">
                    <button
                      type="button"
                      className="btn btn--primary"
                      disabled={settings.saving || form.id.length === 0}
                      onClick={() => {
                        void save();
                      }}
                    >
                      Save
                    </button>

                    <button
                      type="button"
                      className="btn"
                      disabled={
                        !selectedProvider ||
                        !selectedProvider.hasApiKey ||
                        settings.saving
                      }
                      onClick={() => {
                        void clearSecret();
                      }}
                    >
                      Clear secret
                    </button>

                    <button
                      type="button"
                      className="btn"
                      disabled={
                        !selectedProvider ||
                        selectedProvider.builtin ||
                        settings.saving
                      }
                      onClick={() => {
                        void removeProvider();
                      }}
                    >
                      Remove
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <SecurityPosture />
        <UpdateSettings />
        <CertificationPanel />
      </div>
    </div>
  );
}
