import React from 'react';
import { Navbar } from './components/Navbar.tsx';
import { Hero } from './components/Hero.tsx';
import { PreviewSection } from './components/PreviewSection.tsx';
import { DownloadGrid } from './components/DownloadGrid.tsx';
import { FeatureGrid } from './components/FeatureGrid.tsx';
import { Footer } from './components/Footer.tsx';

export const App: React.FC = () => {
  return (
    <>
      {/* Fixed Viewport Hairline Frame */}
      <div className="kw-outer-frame" aria-hidden="true" />

      {/* Main Page Layout Container */}
      <div className="kw-app">
        <Navbar />
        <Hero />
        <PreviewSection />
        <DownloadGrid />
        <FeatureGrid />
        <Footer />
      </div>
    </>
  );
};

export default App;
