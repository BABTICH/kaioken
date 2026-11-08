import React from 'react';

export function HeaderBanner({ title }) {
    return (
        <header className="banner">
            <h1>{title}</h1>
        </header>
    );
}

export const Footer = () => (
    <footer>
        <p>&copy; 2026 Kaioken</p>
    </footer>
);
