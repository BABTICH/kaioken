import { useEffect, useRef, useState } from 'react';

/**
 * Tracks whether an element sits inside the viewport, so the caller can animate
 * both an "open" reveal on entry and a "shut down" exit on leave (unlike a
 * once-only reveal, this keeps tracking after the first entrance).
 */
export function useScrollReveal<T extends HTMLElement>(threshold = 0.15) {
  const ref = useRef<T | null>(null);
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    const node = ref.current;
    if (!node) return;

    const observer = new IntersectionObserver(
      ([entry]) => setIsVisible(entry.isIntersecting),
      { threshold, rootMargin: '0px 0px -8% 0px' }
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, [threshold]);

  return { ref, isVisible };
}
