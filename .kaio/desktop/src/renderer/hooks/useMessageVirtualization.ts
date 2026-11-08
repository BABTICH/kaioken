// src/renderer/hooks/useMessageVirtualization.ts
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState
} from "react";

interface VirtualizationOptions {
  estimatedHeight?: number;
  overscan?: number;
}

interface VirtualizationResult {
  containerRef: (node: HTMLDivElement | null) => void;
  visibleIds: string[];
  topSpacerHeight: number;
  bottomSpacerHeight: number;
  totalHeight: number;
  setHeight: (id: string, height: number) => void;
}

interface PositionEntry {
  id: string;
  top: number;
  height: number;
}

function findStartIndex(
  positions: PositionEntry[],
  target: number
): number {
  let low = 0;
  let high = positions.length - 1;

  while (low <= high) {
    const mid = Math.floor((low + high) / 2);
    const entry = positions[mid];

    if (entry.top + entry.height < target) {
      low = mid + 1;
    } else {
      high = mid - 1;
    }
  }

  return Math.max(0, low);
}

export function useMessageVirtualization(
  ids: string[],
  options?: VirtualizationOptions
): VirtualizationResult {
  const estimatedHeight = options?.estimatedHeight ?? 180;
  const overscan = options?.overscan ?? 700;

  const [containerEl, setContainerEl] = useState<HTMLDivElement | null>(
    null
  );

  const containerRef = useCallback(
    (node: HTMLDivElement | null) => {
      setContainerEl(node);
    },
    []
  );

  const heightsRef = useRef(new Map<string, number>());
  const [heightsVersion, setHeightsVersion] = useState(0);

  const [scrollTop, setScrollTop] = useState(0);
  const [viewportHeight, setViewportHeight] = useState(0);

  const stickToBottomRef = useRef(true);

  useEffect(() => {
    const validIds = new Set(ids);

    for (const key of [...heightsRef.current.keys()]) {
      if (!validIds.has(key)) {
        heightsRef.current.delete(key);
      }
    }
  }, [ids]);

  useEffect(() => {
    if (!containerEl) {
      return;
    }

    const update = () => {
      setScrollTop(containerEl.scrollTop);
      setViewportHeight(containerEl.clientHeight);

      stickToBottomRef.current =
        containerEl.scrollHeight -
          containerEl.scrollTop -
          containerEl.clientHeight <
        90;
    };

    update();

    containerEl.addEventListener("scroll", update, {
      passive: true
    });

    const observer = new ResizeObserver(update);
    observer.observe(containerEl);

    return () => {
      containerEl.removeEventListener("scroll", update);
      observer.disconnect();
    };
  }, [containerEl]);

  const setHeight = useCallback(
    (id: string, height: number) => {
      if (height <= 0) {
        return;
      }

      const previous = heightsRef.current.get(id);

      if (previous !== height) {
        heightsRef.current.set(id, height);
        setHeightsVersion((version) => version + 1);
      }
    },
    []
  );

  const positions = useMemo(() => {
    let top = 0;

    return ids.map((id) => {
      const height = heightsRef.current.get(id) ?? estimatedHeight;

      const entry: PositionEntry = {
        id,
        top,
        height
      };

      top += height;

      return entry;
    });
  }, [ids, heightsVersion, estimatedHeight]);

  const totalHeight =
    positions.length > 0
      ? positions[positions.length - 1].top +
        positions[positions.length - 1].height
      : 0;

  const windowed = useMemo(() => {
    if (positions.length === 0) {
      return {
        visibleIds: [] as string[],
        topSpacerHeight: 0,
        bottomSpacerHeight: 0
      };
    }

    if (viewportHeight === 0) {
      return {
        visibleIds: positions.map((position) => position.id),
        topSpacerHeight: 0,
        bottomSpacerHeight: 0
      };
    }

    const start = findStartIndex(
      positions,
      Math.max(0, scrollTop - overscan)
    );

    let end = start;

    while (
      end < positions.length &&
      positions[end].top < scrollTop + viewportHeight + overscan
    ) {
      end++;
    }

    const topSpacerHeight = positions[start]?.top ?? 0;

    const bottomSpacerHeight =
      end < positions.length
        ? totalHeight - positions[end].top
        : 0;

    return {
      visibleIds: positions.slice(start, end).map((entry) => entry.id),
      topSpacerHeight,
      bottomSpacerHeight
    };
  }, [
    positions,
    scrollTop,
    viewportHeight,
    overscan,
    totalHeight
  ]);

  useEffect(() => {
    if (containerEl && stickToBottomRef.current) {
      containerEl.scrollTop = containerEl.scrollHeight;
    }
  }, [totalHeight, ids.length, containerEl]);

  return {
    containerRef,
    visibleIds: windowed.visibleIds,
    topSpacerHeight: windowed.topSpacerHeight,
    bottomSpacerHeight: windowed.bottomSpacerHeight,
    totalHeight,
    setHeight
  };
}
