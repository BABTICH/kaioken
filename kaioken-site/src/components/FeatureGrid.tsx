import React from 'react';
import { FEATURES } from '../data/content.ts';

export const FeatureGrid: React.FC = () => {
  return (
    <section className="kw-paper-section" id="features">
      {/* Sticky Logo Badge in Left Margin */}
      <div className="kw-paper-badge" title="Kaioken Verified Engine">
        <img src="/assets/kaioken-logo.png" alt="Kaioken Official Logo" />
      </div>

      {/* Feature Preview Tag Top-Right */}
      <div className="kw-paper-tag-pill">
        <span className="kw-paper-tag-item">FEATURE</span>
        <span className="kw-paper-tag-item">PREVIEW</span>
      </div>

      {/* 6-Item Feature Grid */}
      <div className="kw-feature-grid">
        {FEATURES.map((feature) => (
          <article key={feature.id} className="kw-feature-article">
            {/* Header with Number & Title */}
            <div className="kw-feature-header">
              <p className="kw-feature-num">{feature.num}</p>
              <h2 className="kw-feature-heading">{feature.title}</h2>
            </div>

            {/* Visual Media Box */}
            <div className="kw-feature-media-box">
              <img
                src={feature.image}
                alt={feature.title}
                className="kw-feature-img"
                loading="lazy"
              />
            </div>

            {/* Description Paragraph */}
            <p className="kw-feature-desc">{feature.description}</p>
          </article>
        ))}
      </div>

      {/* Screen-Spanning Colossal KAIOKEN Wordmark in Kaioken Red */}
      <div className="kw-wordmark-container" aria-hidden="true">
        <div className="kw-wordmark-text">KAIOKEN</div>
      </div>
    </section>
  );
};
