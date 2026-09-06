import React from 'react';
import { ArrowUpRight } from 'lucide-react';
import { KaiokenPowerCore } from './KaiokenPowerCore.tsx';

export const Footer: React.FC = () => {
  return (
    <footer className="kw-footer">
      {/* Ghost Watermark */}
      <div className="kw-footer-ghost" aria-hidden="true">
        KAIOKEN STUDIO
      </div>

      {/* Top Header & Plans Callout */}
      <div className="kw-footer-top">
        <p className="kw-footer-eyebrow">
          OFFLINE-FIRST • CORE FREE • PRO CLOUD • ENTERPRISE
        </p>

        <h2 className="kw-footer-title">KAIOKEN STUDIO</h2>

        <p className="kw-footer-desc">
          Deterministic repository intelligence, verified technical wikis, and high-velocity
          agentic workflows. Zero documentation rot. Zero hallucinated symbols.
        </p>

        <a
          href="https://github.com/babtix/kaioken"
          target="_blank"
          rel="noopener noreferrer"
          className="kw-btn-primary"
          style={{ marginTop: '16px' }}
        >
          VIEW ON GITHUB <ArrowUpRight size={18} />
        </a>
      </div>

      {/* Authentic Kaioken Power Core & AST Reactor (Replaces the Hermes girl image) */}
      <KaiokenPowerCore />

      {/* Bottom Metadata Bar */}
      <div className="kw-footer-bottom">
        <div>
          <p style={{ color: 'var(--kai-white-muted)' }}>KAIOKEN ENGINE v2.0.0-PROD</p>
        </div>

        <div className="kw-footer-bottom-right">
          <img
            src="/assets/kaioken-logo.png"
            alt="Kaioken Official Logo"
            className="kw-footer-bottom-logo"
          />
          <p style={{ color: 'var(--kai-white)' }}>
            ENGINEERED BY BABTIX &amp; KAIOKEN CONTRIBUTORS
          </p>
          <p style={{ color: 'var(--kai-orange)', opacity: 0.85 }}>
            MIT &amp; LICENSE ZERO · 2026
          </p>
        </div>
      </div>
    </footer>
  );
};
