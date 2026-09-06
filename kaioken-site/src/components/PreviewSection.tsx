import React, { useState } from 'react';
import { useScrollReveal } from '../hooks/useScrollReveal.ts';

interface CommandExample {
  label: string;
  cmd: string;
  output: string[];
}

export const PreviewSection: React.FC = () => {
  const COMMANDS: CommandExample[] = [
    {
      label: '/verify',
      cmd: '/verify --strict',
      output: [
        '● Running AST grounding check across 267 files (38,490 symbols)...',
        '✔ SymbolOracle: All 103 markdown docs match active code ASTs',
        '✔ Native verification gate: 412 test suites passing (0 token hallucination)',
        'Status: 100% fresh · Repository intelligence confirmed.',
      ],
    },
    {
      label: '/wiki',
      cmd: '/wiki --plan',
      output: [
        '● Synthesizing deterministic repository wiki dossier...',
        '✔ Ingested AST skeletons across TypeScript, Go, Python, and Rust',
        '✔ Generated 28 architectural chapters into .kaioken/wiki/',
        'Status: Saved to local vault · SHA-256 provenance lattice locked.',
      ],
    },
    {
      label: '/multiplier',
      cmd: '/multiplier x10',
      output: [
        '🔥 KAIOKEN MULTIPLIER ×10 ACTIVATED',
        '⚡ Parallel search breadth: 20 queries · Verification passes: 10',
        '✔ 4 isolated sandboxed subagents executing in parallel',
        'Status: Maximum compute intensity engaged with total token transparency.',
      ],
    },
  ];

  const [activeCmdIndex, setActiveCmdIndex] = useState(0);
  const { ref: frameRef, isVisible: frameOpen } = useScrollReveal<HTMLDivElement>(0.15);

  return (
    <section className="kw-preview-section" id="preview">
      {/* Top Section Header */}
      <div className="kw-section-header">
        <p className="kw-section-eyebrow">NATIVE TERMINAL INTERFACE</p>
        <h2 className="kw-section-title">THE ENGINE IN ACTION</h2>
      </div>

      {/* Showcase Terminal Frame — Compact & Authentic to kaioken_v2 */}
      <div ref={frameRef} className={`kw-showcase-frame kw-reveal ${frameOpen ? 'is-open' : ''}`}>
        {/* Terminal Window Chrome Titlebar */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '10px 16px',
            backgroundColor: '#0d0d12',
            borderBottom: '1px solid rgba(255, 30, 0, 0.25)',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span style={{ width: 10, height: 10, borderRadius: '50%', backgroundColor: '#ff1e00' }} />
            <span style={{ width: 10, height: 10, borderRadius: '50%', backgroundColor: '#ff8700' }} />
            <span style={{ width: 10, height: 10, borderRadius: '50%', backgroundColor: '#ffffff' }} />
            <span
              style={{
                fontFamily: 'var(--font-mono)',
                fontSize: '0.74rem',
                color: 'rgba(255,255,255,0.6)',
                marginLeft: 10,
              }}
            >
              kaioken@kaioken_v2 — 2.0.0
            </span>
          </div>

          {/* Quick Command Selector */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            {COMMANDS.map((item, i) => (
              <button
                key={item.label}
                type="button"
                onClick={() => setActiveCmdIndex(i)}
                style={{
                  background: activeCmdIndex === i ? 'rgba(255, 135, 0, 0.18)' : 'transparent',
                  border: '1px solid',
                  borderColor: activeCmdIndex === i ? '#ff8700' : 'rgba(255,255,255,0.15)',
                  color: activeCmdIndex === i ? '#ff8700' : 'rgba(255,255,255,0.6)',
                  fontFamily: 'var(--font-mono)',
                  fontSize: '0.68rem',
                  padding: '3px 8px',
                  borderRadius: 2,
                  cursor: 'pointer',
                  fontWeight: activeCmdIndex === i ? 700 : 500,
                }}
              >
                {item.label}
              </button>
            ))}
          </div>
        </div>

        {/* Terminal Body */}
        <div
          style={{
            padding: '20px 24px',
            backgroundColor: '#08080a',
            fontFamily: 'var(--font-mono)',
            fontSize: 'max(0.78rem, 13px)',
            lineHeight: 1.5,
            color: 'rgba(255, 255, 255, 0.88)',
          }}
        >
          {/* Top TUI Banner (Left: Logo + Keys, Right: System Specs) */}
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: '1fr 1fr',
              gap: '24px',
              alignItems: 'start',
            }}
          >
            {/* Left: Pixel Logo & Shortcuts */}
            <div>
              <img
                src="/assets/kaioken-logo.png"
                alt="KAIOKEN"
                style={{
                  height: 38,
                  width: 'auto',
                  objectFit: 'contain',
                  display: 'block',
                  marginBottom: 14,
                }}
              />
              <div
                style={{
                  display: 'flex',
                  flexDirection: 'column',
                  gap: 3,
                  fontSize: '0.72rem',
                  lineHeight: 1.45,
                }}
              >
                <div>
                  <span style={{ color: '#ff8700', fontWeight: 600 }}>enter</span>{' '}
                  <span style={{ color: 'rgba(255,255,255,0.5)' }}>send</span>
                  <span style={{ color: 'rgba(255,255,255,0.25)', margin: '0 6px' }}>·</span>
                  <span style={{ color: '#ff8700', fontWeight: 600 }}>alt+enter</span>{' '}
                  <span style={{ color: 'rgba(255,255,255,0.5)' }}>newline</span>
                  <span style={{ color: 'rgba(255,255,255,0.25)', margin: '0 6px' }}>·</span>
                  <span style={{ color: '#ff8700', fontWeight: 600 }}>/</span>{' '}
                  <span style={{ color: 'rgba(255,255,255,0.5)' }}>commands</span>
                </div>
                <div>
                  <span style={{ color: '#ff8700', fontWeight: 600 }}>tab</span>{' '}
                  <span style={{ color: 'rgba(255,255,255,0.5)' }}>complete</span>
                  <span style={{ color: 'rgba(255,255,255,0.25)', margin: '0 6px' }}>·</span>
                  <span style={{ color: '#ff8700', fontWeight: 600 }}>↑↓</span>{' '}
                  <span style={{ color: 'rgba(255,255,255,0.5)' }}>history</span>
                  <span style={{ color: 'rgba(255,255,255,0.25)', margin: '0 6px' }}>·</span>
                  <span style={{ color: '#ff8700', fontWeight: 600 }}>pgup/pgdn</span>{' '}
                  <span style={{ color: 'rgba(255,255,255,0.5)' }}>scroll</span>
                </div>
                <div>
                  <span style={{ color: '#ff8700', fontWeight: 600 }}>ctrl+shift+f</span>{' '}
                  <span style={{ color: 'rgba(255,255,255,0.5)' }}>search</span>
                  <span style={{ color: 'rgba(255,255,255,0.25)', margin: '0 6px' }}>·</span>
                  <span style={{ color: '#ff8700', fontWeight: 600 }}>ctrl+c</span>{' '}
                  <span style={{ color: 'rgba(255,255,255,0.5)' }}>twice to quit</span>
                </div>
              </div>
            </div>

            {/* Right: Spec Info */}
            <div style={{ fontSize: '0.74rem', lineHeight: 1.5 }}>
              <div
                style={{
                  color: '#ff8700',
                  fontWeight: 700,
                  borderBottom: '1px solid rgba(255, 135, 0, 0.4)',
                  display: 'inline-block',
                  marginBottom: 6,
                }}
              >
                kaioken@kaioken_v2
              </div>
              <div style={{ display: 'grid', gridTemplateColumns: '84px 1fr', gap: '2px 8px' }}>
                <span style={{ color: '#ff8700', fontWeight: 600 }}>Version:</span>
                <span style={{ color: '#ffffff' }}>2.0.0</span>

                <span style={{ color: '#ff8700', fontWeight: 600 }}>Repo:</span>
                <span style={{ color: '#ffffff' }}>~/workspace/kaioken-core</span>

                <span style={{ color: '#ff8700', fontWeight: 600 }}>Branch:</span>
                <span style={{ color: '#38bdf8' }}>master</span>

                <span style={{ color: '#ff8700', fontWeight: 600 }}>Model:</span>
                <span style={{ color: '#ffffff' }}>kaioken-neural-v2</span>

                <span style={{ color: '#ff8700', fontWeight: 600 }}>Provider:</span>
                <span style={{ color: '#ffffff' }}>kaioken-cloud</span>

                <span style={{ color: '#ff8700', fontWeight: 600 }}>API Key:</span>
                <span style={{ color: '#10b981', fontWeight: 600 }}>saved ✓</span>

                <span style={{ color: '#ff8700', fontWeight: 600 }}>Knowledge:</span>
                <span>
                  <span style={{ color: '#ffffff' }}>267 files · 103 docs · </span>
                  <span style={{ color: '#10b981', fontWeight: 600 }}>100% fresh</span>
                </span>
              </div>
            </div>
          </div>

          {/* Full Width Divider */}
          <div
            style={{
              borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
              margin: '14px 0 8px 0',
            }}
          />

          {/* Tutorial / Subtitle prompt */}
          <div
            style={{
              color: 'rgba(255, 255, 255, 0.45)',
              fontSize: '0.72rem',
              marginBottom: 16,
            }}
          >
            type to chat <span style={{ color: 'rgba(255,255,255,0.2)' }}>·</span> press{' '}
            <span style={{ color: '#ff8700' }}>/</span> for commands{' '}
            <span style={{ color: 'rgba(255,255,255,0.2)' }}>·</span>{' '}
            <span style={{ color: 'rgba(255,255,255,0.7)' }}>/tutorial</span> to learn them
          </div>

          {/* Active Command Execution Box */}
          <div
            style={{
              backgroundColor: 'rgba(255, 255, 255, 0.02)',
              border: '1px solid rgba(255, 135, 0, 0.2)',
              borderRadius: 3,
              padding: '12px 14px',
              marginBottom: 14,
            }}
          >
            <div style={{ color: '#ff8700', fontWeight: 700, marginBottom: 6 }}>
              › {COMMANDS[activeCmdIndex].cmd}
            </div>
            {COMMANDS[activeCmdIndex].output.map((line, idx) => (
              <div
                key={idx}
                style={{
                  color: line.startsWith('✔')
                    ? '#ffffff'
                    : line.startsWith('🔥') || line.startsWith('⚡')
                    ? '#ff8700'
                    : line.startsWith('Status:')
                    ? '#10b981'
                    : 'rgba(255,255,255,0.72)',
                  fontSize: '0.76rem',
                  lineHeight: 1.5,
                  fontWeight: line.startsWith('Status:') || line.startsWith('🔥') ? 600 : 400,
                }}
              >
                {line}
              </div>
            ))}
          </div>

          {/* Active Prompt Line */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              color: '#38bdf8',
              fontSize: '0.78rem',
              paddingTop: 4,
            }}
          >
            <span style={{ marginRight: 6 }}>›</span>
            <span style={{ color: 'rgba(255,255,255,0.7)' }}>chat with the model, or /help for commands</span>
            <span
              style={{
                display: 'inline-block',
                width: 7,
                height: 14,
                backgroundColor: '#ff8700',
                marginLeft: 6,
              }}
            />
          </div>

          {/* Bottom Bar Status Line */}
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              paddingTop: 14,
              marginTop: 10,
              borderTop: '1px solid rgba(255, 255, 255, 0.08)',
              fontSize: '0.68rem',
              color: 'rgba(255, 255, 255, 0.45)',
            }}
          >
            <div>
              <span style={{ color: 'rgba(255,255,255,0.6)' }}>/ commands</span>{' '}
              <span style={{ color: 'rgba(255,255,255,0.2)' }}>·</span> alt+enter newline{' '}
              <span style={{ color: 'rgba(255,255,255,0.2)' }}>·</span> ctrl+d quit
            </div>
            <div style={{ color: 'rgba(255,255,255,0.7)' }}>kaioken-neural-v2</div>
          </div>
        </div>
      </div>
    </section>
  );
};
