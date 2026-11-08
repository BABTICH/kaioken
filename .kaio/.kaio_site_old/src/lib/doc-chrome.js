import * as React from "react";
export const DocChromeContext = React.createContext({ bare: false });
export function useDocChrome() {
    return React.useContext(DocChromeContext);
}
