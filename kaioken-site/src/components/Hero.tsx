import React, { useState, useEffect } from 'react';
import { TERMINAL_COMMANDS } from '../data/content.ts';
import { Check, Copy, Monitor } from 'lucide-react';
import AeroShards from './AeroShards.tsx';
import { useScrollReveal } from '../hooks/useScrollReveal.ts';

export const Hero: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'mac' | 'win'>('mac');
  const [copied, setCopied] = useState(false);
  const [detectedOS, setDetectedOS] = useState<'Windows' | 'macOS' | 'Linux'>('Windows');
  const [auraFailed, setAuraFailed] = useState(false);
  const { ref: artRef, isVisible: artOpen } = useScrollReveal<HTMLDivElement>(0.15);

  useEffect(() => {
    const userAgent = window.navigator.userAgent.toLowerCase();
    if (userAgent.includes('mac')) {
      setDetectedOS('macOS');
      setActiveTab('mac');
    } else if (userAgent.includes('linux')) {
      setDetectedOS('Linux');
      setActiveTab('mac');
    } else {
      setDetectedOS('Windows');
      setActiveTab('win');
    }
  }, []);

  const handleCopy = () => {
    const textToCopy =
      activeTab === 'win' ? TERMINAL_COMMANDS.windows : TERMINAL_COMMANDS.macLinux;
    navigator.clipboard.writeText(textToCopy);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <header className="kw-hero">
      {/* Kaioken Aura Glow — WebGPU shard field across the hero background,
          weighted to the right so the headline column stays clear */}
      {!auraFailed && (
        <div className="kw-hero-bg" aria-hidden="true">
          <AeroShards
            backgroundColor="#08080a"
            shardColor="#ff8700"
            accentColor="#ff1e00"
            placement="right"
            flow="vortex"
            material="pearl"
            detail="balanced"
            scale={2.2}
            spread={1.1}
            depth={1.15}
            speed={0.18}
            spin={1}
            interaction="repel"
            density={1.2}
            shardSize={1}
            glow={1.6}
            bloom={1.4}
            grain={0}
            chromaticAberration={0.004}
            transitionDuration={1.4}
            interactionRadius={0.9}
            interactionStrength={0.6}
            rippleIntensity={1.2}
            holdToGather={false}
            onError={() => setAuraFailed(true)}
          />
        </div>
      )}

      {/* Hero Left Content */}
      <div className="kw-hero-content">
        <p className="kw-hero-eyebrow">
          OPEN SOURCE • MIT &amp; L0 LICENSE • DETERMINISTIC KNOWLEDGE ENGINE
        </p>

        <h1 className="kw-hero-title">
          <span>THE REPO</span>
          <span>THAT KNOWS</span>
          <span className="highlight-red">YOUR CODE</span>
        </h1>

        <div className="kw-hero-actions">
          {/* Download App Section */}
          <section style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            <p className="kw-hero-eyebrow" style={{ opacity: 0.85 }}>
              DOWNLOAD KAIOKEN DESKTOP APP
            </p>
            <a
              href="#downloads"
              className="kw-btn-primary"
              id="hero-download-cta"
            >
              <Monitor size={20} />
              DOWNLOAD FOR {detectedOS.toUpperCase()}
            </a>
          </section>

          {/* Install via Terminal Widget */}
          <section id="install" style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            <p className="kw-hero-eyebrow" style={{ opacity: 0.85 }}>
              INSTALL VIA TERMINAL
            </p>

            <div className="kw-terminal-box">
              {/* Terminal Tabs */}
              <div className="kw-terminal-tabs" role="tablist">
                <button
                  type="button"
                  role="tab"
                  className={`kw-terminal-tab ${activeTab === 'mac' ? 'active' : ''}`}
                  onClick={() => setActiveTab('mac')}
                >
                  macOS / Linux
                </button>
                <button
                  type="button"
                  role="tab"
                  className={`kw-terminal-tab ${activeTab === 'win' ? 'active' : ''}`}
                  onClick={() => setActiveTab('win')}
                >
                  Windows (PowerShell)
                </button>
              </div>

              {/* Terminal Code Row */}
              <div className="kw-terminal-content">
                <div className="kw-terminal-code">
                  {activeTab === 'mac' ? (
                    <>
                      <span className="dim">curl </span>
                      <span className="highlight">-fsSL https://kaioken.dev/install.sh</span>
                      <span className="dim"> | bash</span>
                    </>
                  ) : (
                    <>
                      <span className="dim">irm </span>
                      <span className="highlight">https://kaioken.dev/install.ps1</span>
                      <span className="dim"> | iex</span>
                    </>
                  )}
                </div>

                <button
                  type="button"
                  className="kw-copy-btn"
                  onClick={handleCopy}
                  title="Copy command"
                  aria-label="Copy install command"
                >
                  {copied ? <Check size={18} color="#ff8700" /> : <Copy size={18} />}
                </button>
              </div>
            </div>
          </section>
        </div>
      </div>

      {/* Hero Right Engraving Art with Kaioken Aura Glow */}
      <div ref={artRef} className={`kw-hero-art kw-reveal ${artOpen ? 'is-open' : ''}`} aria-hidden="true">
        <img
          src="/assets/hero-engraving.jpg"
          alt="Classical celestial warrior channeling power aura"
          loading="eager"
        />
      </div>
    </header>
  );
};
