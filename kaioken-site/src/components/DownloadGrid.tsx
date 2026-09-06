import React from 'react';
import { OS_DOWNLOADS } from '../data/content.ts';
import { Terminal, Download } from 'lucide-react';

export const DownloadGrid: React.FC = () => {
  return (
    <section className="kw-downloads-section" id="downloads">
      {/* Section Header */}
      <div className="kw-section-header">
        <p className="kw-section-eyebrow">NATIVE DESKTOP APP &amp; CLI</p>
        <h2 className="kw-section-title">
          KAIOKEN DESKTOP FOR MACOS, WINDOWS &amp; LINUX
        </h2>
      </div>

      {/* 3-Column Grid */}
      <div className="kw-downloads-grid">
        {OS_DOWNLOADS.map((item) => (
          <div key={item.os} className="kw-download-card">
            {/* Background Etching Art */}
            <img
              src={item.art}
              alt={`${item.os} classical woodcut art`}
              className="kw-card-bg-art"
              loading="lazy"
            />

            {/* Card Content */}
            <div className="kw-card-content">
              <p className="kw-card-version">{item.version}</p>
              <h3 className="kw-card-title">{item.os}</h3>

              <a
                href={item.href}
                className="kw-btn-primary"
                style={{
                  padding: '14px 26px',
                  fontSize: '0.82rem',
                }}
              >
                {item.icon === 'linux' ? (
                  <Terminal size={18} />
                ) : (
                  <Download size={18} />
                )}
                {item.buttonText}
              </a>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
};
