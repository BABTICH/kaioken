import React from 'react';
import { NAV_LINKS, SOCIAL_LINKS } from '../data/content.ts';
import { ArrowRight } from 'lucide-react';

export const Navbar: React.FC = () => {
  return (
    <nav className="kw-nav" aria-label="Main Navigation">
      {/* Left Links */}
      <div className="kw-nav-left">
        {NAV_LINKS.map((link) => (
          <a
            key={link.label}
            href={link.href}
            className="kw-nav-link"
            target={link.href.startsWith('http') ? '_blank' : '_self'}
            rel="noopener noreferrer"
          >
            {link.label}
          </a>
        ))}
      </div>

      {/* Center Brand Identity with Official Logo */}
      <div className="kw-nav-center">
        <a href="#" className="kw-nav-logo-link" aria-label="Kaioken Home">
          <img
            src="/assets/kaioken-logo.png"
            alt="KAIOKEN Official Logo"
            className="kw-nav-logo-img"
          />
        </a>

        {/* Social Icons Row */}
        <div className="kw-nav-social-icons">
          {/* Discord */}
          <a
            href={SOCIAL_LINKS.discord}
            target="_blank"
            rel="noopener noreferrer"
            className="kw-nav-social-link"
            aria-label="Discord Community"
          >
            <svg width="18" height="15" viewBox="0 0 28 22" fill="currentColor">
              <path d="M23.0569 1.76461C21.295 0.940249 19.4111 0.341139 17.4415 0C17.1996 0.437328 16.917 1.02554 16.7221 1.49347C14.6284 1.17859 12.5539 1.17859 10.4987 1.49347C10.3039 1.02554 10.0149 0.437328 9.77083 0C7.7991 0.341139 5.91301 0.94245 4.15109 1.76897C0.597274 7.13925 -0.366106 12.3761 0.115585 17.5387C2.47266 19.2989 4.75695 20.3681 7.00269 21.0678C7.55717 20.3047 8.0517 19.4935 8.47772 18.6385C7.66635 18.3302 6.88923 17.9498 6.15494 17.5081C6.34974 17.3637 6.54029 17.2129 6.72438 17.0576C11.203 19.1524 16.0692 19.1524 20.4943 17.0576C20.6806 17.2129 20.8711 17.3637 21.0638 17.5081C20.3273 17.9519 19.5481 18.3324 18.7367 18.6407C19.1627 19.4935 19.6551 20.3069 20.2117 21.07C22.4596 20.3703 24.746 19.3011 27.1031 17.5387C27.6683 11.554 26.1376 6.36518 23.0569 1.76461ZM9.08788 14.3638C7.74343 14.3638 6.64088 13.1086 6.64088 11.5802C6.64088 10.0518 7.71989 8.7945 9.08788 8.7945C10.4559 8.7945 11.5584 10.0496 11.5349 11.5802C11.537 13.1086 10.4559 14.3638 9.08788 14.3638ZM18.1308 14.3638C16.7864 14.3638 15.6838 13.1086 15.6838 11.5802C15.6838 10.0518 16.7628 8.7945 18.1308 8.7945C19.4988 8.7945 20.6014 10.0496 20.5778 11.5802C20.5778 13.1086 19.4988 14.3638 18.1308 14.3638Z" />
            </svg>
          </a>
          {/* X */}
          <a
            href={SOCIAL_LINKS.x}
            target="_blank"
            rel="noopener noreferrer"
            className="kw-nav-social-link"
            aria-label="X / Twitter"
          >
            <svg width="15" height="15" viewBox="0 0 24 24" fill="currentColor">
              <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24h-6.66l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231Zm-1.161 17.52h1.833L7.084 4.126H5.117Z" />
            </svg>
          </a>
          {/* GitHub */}
          <a
            href={SOCIAL_LINKS.github}
            target="_blank"
            rel="noopener noreferrer"
            className="kw-nav-social-link"
            aria-label="GitHub Repository"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 .5C5.37.5 0 5.87 0 12.5c0 5.3 3.44 9.8 8.21 11.39.6.11.82-.26.82-.58v-2.03c-3.34.73-4.04-1.61-4.04-1.61-.55-1.39-1.34-1.76-1.34-1.76-1.09-.75.08-.73.08-.73 1.2.08 1.84 1.24 1.84 1.24 1.07 1.83 2.81 1.3 3.5.99.11-.78.42-1.3.76-1.6-2.67-.3-5.47-1.34-5.47-5.95 0-1.31.47-2.39 1.24-3.23-.12-.31-.54-1.53.12-3.18 0 0 1.01-.32 3.3 1.23a11.5 11.5 0 0 1 6 0c2.29-1.55 3.3-1.23 3.3-1.23.66 1.65.24 2.87.12 3.18.77.84 1.24 1.92 1.24 3.23 0 4.62-2.81 5.64-5.49 5.94.43.37.81 1.1.81 2.22v3.29c0 .32.22.7.83.58A12 12 0 0 0 24 12.5C24 5.87 18.63.5 12 .5Z" />
            </svg>
          </a>
        </div>
      </div>

      {/* Right Tools & Install Action */}
      <div className="kw-nav-right">
        <span className="kw-multiplier-badge">
          ⚡ MULTIPLIER ×10
        </span>

        <a href="#install" className="kw-install-btn">
          INSTALL <ArrowRight size={14} />
        </a>
      </div>
    </nav>
  );
};
