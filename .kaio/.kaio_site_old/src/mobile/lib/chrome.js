import { jsx as _jsx } from "react/jsx-runtime";
import * as React from "react";
const ChromeContext = React.createContext({
    title: null,
    setTitle: () => { },
});
export function ChromeProvider({ children }) {
    const [title, setTitle] = React.useState(null);
    const value = React.useMemo(() => ({ title, setTitle }), [title]);
    return _jsx(ChromeContext.Provider, { value: value, children: children });
}
/** Publish this screen's title while it is mounted. */
export function useScreenTitle(title) {
    const { setTitle } = React.useContext(ChromeContext);
    React.useEffect(() => {
        setTitle(title);
        return () => setTitle(null);
    }, [title, setTitle]);
}
export function useChromeTitle() {
    return React.useContext(ChromeContext).title;
}
/** Where a detail screen's back button goes. */
export function parentPath(pathname) {
    if (/^\/docs\/.+/.test(pathname))
        return "/docs";
    if (/^\/preview\/.+/.test(pathname))
        return "/preview";
    return null;
}
