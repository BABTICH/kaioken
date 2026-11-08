import * as React from "react";
export const PHONE_QUERY = "(max-width: 767px)";
const KEY = "kaioken:layout";
const EVENT = "kaioken:layout-change";
function readOverride() {
    try {
        const v = localStorage.getItem(KEY);
        return v === "mobile" || v === "desktop" ? v : null;
    }
    catch {
        return null;
    }
}
function subscribeOverride(onChange) {
    window.addEventListener(EVENT, onChange);
    window.addEventListener("storage", onChange);
    return () => {
        window.removeEventListener(EVENT, onChange);
        window.removeEventListener("storage", onChange);
    };
}
export function setLayoutOverride(layout) {
    try {
        if (layout)
            localStorage.setItem(KEY, layout);
        else
            localStorage.removeItem(KEY);
    }
    catch {
        // ignore in private mode
    }
    window.dispatchEvent(new Event(EVENT));
}
export function useLayoutOverride() {
    return React.useSyncExternalStore(subscribeOverride, readOverride, () => null);
}
export function useMediaQuery(query) {
    const [subscribe, getSnapshot] = React.useMemo(() => {
        if (typeof window === "undefined") {
            return [() => () => { }, () => false];
        }
        const mq = window.matchMedia(query);
        return [
            (onChange) => {
                mq.addEventListener("change", onChange);
                return () => mq.removeEventListener("change", onChange);
            },
            () => mq.matches,
        ];
    }, [query]);
    return React.useSyncExternalStore(subscribe, getSnapshot, () => false);
}
export function useLayout() {
    const override = useLayoutOverride();
    const isPhone = useMediaQuery(PHONE_QUERY);
    return override ?? (isPhone ? "mobile" : "desktop");
}
