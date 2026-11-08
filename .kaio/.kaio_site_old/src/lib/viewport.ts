import * as React from "react"

export const PHONE_QUERY = "(max-width: 767px)"

export type Layout = "mobile" | "desktop"

const KEY = "kaioken:layout"
const EVENT = "kaioken:layout-change"

function readOverride(): Layout | null {
  try {
    const v = localStorage.getItem(KEY)
    return v === "mobile" || v === "desktop" ? v : null
  } catch {
    return null
  }
}

function subscribeOverride(onChange: () => void) {
  window.addEventListener(EVENT, onChange)
  window.addEventListener("storage", onChange)
  return () => {
    window.removeEventListener(EVENT, onChange)
    window.removeEventListener("storage", onChange)
  }
}

export function setLayoutOverride(layout: Layout | null) {
  try {
    if (layout) localStorage.setItem(KEY, layout)
    else localStorage.removeItem(KEY)
  } catch {
    // ignore in private mode
  }
  window.dispatchEvent(new Event(EVENT))
}

export function useLayoutOverride(): Layout | null {
  return React.useSyncExternalStore(subscribeOverride, readOverride, () => null)
}

export function useMediaQuery(query: string): boolean {
  const [subscribe, getSnapshot] = React.useMemo(() => {
    if (typeof window === "undefined") {
      return [() => () => {}, () => false] as const
    }
    const mq = window.matchMedia(query)
    return [
      (onChange: () => void) => {
        mq.addEventListener("change", onChange)
        return () => mq.removeEventListener("change", onChange)
      },
      () => mq.matches,
    ] as const
  }, [query])

  return React.useSyncExternalStore(subscribe, getSnapshot, () => false)
}

export function useLayout(): Layout {
  const override = useLayoutOverride()
  const isPhone = useMediaQuery(PHONE_QUERY)
  return override ?? (isPhone ? "mobile" : "desktop")
}
